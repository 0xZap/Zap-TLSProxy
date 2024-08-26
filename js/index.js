const net = require("net");
const tls = require("tls");
const http = require("http");
const crypto = require("crypto");

const HTTP_PORT = 8080;

class ZapProxy {
  constructor(proxyHost, proxyPort, targetHost, targetPort) {
    this.proxyHost = proxyHost;
    this.proxyPort = proxyPort;
    this.targetHost = targetHost;
    this.targetPort = targetPort;
    this.accumulatedData = [];
    this.proofId = crypto.randomUUID();
  }

  connect() {
    return new Promise((resolve, reject) => {
      this.proxySocket = net.createConnection(
        { host: this.proxyHost, port: this.proxyPort },
        () => {
          this.proxySocket.write(
            `CONNECT ${this.targetHost}:${this.targetPort} HTTP/1.1\r\nHost: ${this.targetHost}\r\n\r\n`
          );
        }
      );

      this.proxySocket.on("data", (data) => {
        if (data.toString().includes("200 Connection established")) {
          this.establishTlsConnection(resolve, reject);
        } else {
          reject(
            new Error("Failed to establish a connection through the proxy")
          );
          this.proxySocket.end();
        }
      });

      this.proxySocket.on("error", (err) => {
        reject(new Error("Proxy socket error: " + err.message));
      });
    });
  }

  establishTlsConnection(resolve, reject) {
    this.tlsSocket = tls.connect(
      {
        socket: this.proxySocket,
        servername: this.targetHost,
        rejectUnauthorized: false,
      },
      () => {
        resolve();
      }
    );

    this.tlsSocket.on("data", (tlsData) => {
      const serverResponse = tlsData.toString();
      this.accumulatedData.push(serverResponse);
    });

    this.tlsSocket.on("end", () => {
      resolve();
    });

    this.tlsSocket.on("error", (err) => {
      reject(new Error("TLS socket error: " + err.message));
    });
  }

  createHttpRequest({ method, url, headers = {}, body = "" }) {
    const fullPath = url.startsWith("http") ? new URL(url).pathname : url;
    let request = `${method} ${fullPath} HTTP/1.1\r\nHost: ${this.targetHost}\r\n`;

    for (const [key, value] of Object.entries(headers)) {
      request += `${key}: ${value}\r\n`;
    }

    if (body && method !== "GET") {
      request += `Content-Length: ${Buffer.byteLength(body)}\r\n\r\n`;
      request += body;
    } else {
      request += `\r\n`;
    }

    return request;
  }

  prove(schemaJson) {
    return new Promise((resolve, reject) => {
      if (this.tlsSocket) {
        const request = this.createHttpRequest(schemaJson);
        this.tlsSocket.write(request);

        this.tlsSocket.on("end", () => {
          resolve(this.sendProof(schemaJson.id));
        });

        this.tlsSocket.on("error", (err) => {
          reject(new Error("Error during TLS communication: " + err.message));
        });
      } else {
        reject(new Error("TLS connection is not established"));
      }
    });
  }

  sendProof(schemaId) {
    return new Promise((resolve, reject) => {
      const accumulatedDataStr = this.accumulatedData.join("");

      const bodyStartIndex = accumulatedDataStr.indexOf("\r\n\r\n") + 4;
      const responseBody = accumulatedDataStr.slice(bodyStartIndex);

      let filteredData;

      // Schema Filter
      switch (schemaId) {
        case "node_guardians":
          try {
            const parsedData = JSON.parse(responseBody);
            filteredData = parsedData.data?.trees;
            if (!filteredData) {
              return reject(
                new Error("Could not find 'trees' data in the response")
              );
            }
          } catch (err) {
            return reject(
              new Error(
                `Failed to parse 'node_guardians' response: ${err.message}`
              )
            );
          }
          break;

        default:
          filteredData = responseBody;
      }

      const jsonResponse = JSON.stringify({
        proofId: this.proofId,
        schemaId: schemaId,
        data: filteredData,
      });

      const options = {
        hostname: this.proxyHost,
        port: HTTP_PORT,
        path: "/proof",
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          "Content-Length": Buffer.byteLength(jsonResponse),
        },
      };

      const req = http.request(options, (res) => {
        let responseData = "";

        res.on("data", (chunk) => {
          responseData += chunk;
        });

        res.on("end", () => {
          try {
            const jsonData = JSON.parse(responseData);

            if (res.statusCode === 200) {
              resolve(jsonData);
            } else {
              reject(
                new Error(
                  `Failed to send proof: ${jsonData.message || "Unknown error"}`
                )
              );
            }
          } catch (err) {
            reject(new Error(`Failed to parse response: ${err.message}`));
          }
        });
      });

      req.on("error", (e) => {
        reject(new Error(`Problem with proof request: ${e.message}`));
      });

      req.write(jsonResponse);
      req.end();
    });
  }
}

module.exports = ZapProxy;
