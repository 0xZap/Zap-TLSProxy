const net = require("net");
const fs = require("fs");
const crypto = require("crypto");
const http = require("http");

const LISTEN_PORT = 55688;
const HTTP_PORT = 8080;
const LOG_FILE = "proxy.log";
const PRIVATE_KEY_FILE = "private-key.pem";

function logData(data) {
  fs.appendFileSync(LOG_FILE, data + "\n", "utf8");
}

// Function to sign the log
function signLog(logData) {
  const privateKey = fs.readFileSync(PRIVATE_KEY_FILE, "utf8");
  const sign = crypto.createSign("SHA256");
  sign.update(logData);
  sign.end();
  const signature = sign.sign(privateKey, "hex");
  return signature;
}

const proxyServer = net.createServer((clientSocket) => {
  console.log(
    "Client connected:",
    clientSocket.remoteAddress,
    clientSocket.remotePort
  );

  clientSocket.once("data", (data) => {
    const request = data.toString();
    const match = request.match(/^CONNECT\s+([^\s:]+):(\d+)\s+HTTP\/1\.1/i);

    if (match) {
      const targetHost = match[1];
      const targetPort = parseInt(match[2]);

      logData(
        `[${new Date().toISOString()}] CONNECT request to ${targetHost}:${targetPort}`
      );
      logData(
        `[${new Date().toISOString()}] Encrypted data received from client:`
      );
      logData(data.toString("hex"));

      const targetSocket = net.createConnection(
        {
          host: targetHost,
          port: targetPort,
        },
        () => {
          clientSocket.write("HTTP/1.1 200 Connection established\r\n\r\n");

          clientSocket.pipe(targetSocket);
          targetSocket.pipe(clientSocket);

          clientSocket.on("data", (chunk) => {
            logData(
              `[${new Date().toISOString()}] Encrypted data sent to server:`
            );
            logData(chunk.toString("hex"));
          });

          targetSocket.on("data", (chunk) => {
            logData(
              `[${new Date().toISOString()}] Encrypted data received from server:`
            );
            logData(chunk.toString("hex"));
          });
        }
      );

      targetSocket.on("error", (err) => {
        console.error("Target socket error:", err.message);
        clientSocket.end();
      });

      clientSocket.on("error", (err) => {
        console.error("Client socket error:", err.message);
        targetSocket.end();
      });

      clientSocket.on("end", () => {
        console.log("Client disconnected");
        targetSocket.end();
      });

      targetSocket.on("end", () => {
        console.log("Target server disconnected");
        clientSocket.end();
      });
    } else {
      console.error("Invalid CONNECT request");
      clientSocket.end();
    }
  });
});

proxyServer.listen(LISTEN_PORT, () => {
  console.log(`Proxy server listening on port ${LISTEN_PORT}`);
});

const httpServer = http.createServer((req, res) => {
  if (req.method === "GET" && req.url === "/signed-log") {
    try {
      const logData = fs.readFileSync(LOG_FILE, "utf8");
      const signature = signLog(logData);

      res.writeHead(200, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ log: logData, signature: signature }));
    } catch (err) {
      res.writeHead(500, { "Content-Type": "text/plain" });
      res.end("Error signing log: " + err.message);
    }
  } else {
    res.writeHead(404, { "Content-Type": "text/plain" });
    res.end("Not Found");
  }
});

httpServer.listen(HTTP_PORT, () => {
  console.log(`HTTP server listening on port ${HTTP_PORT}`);
});
