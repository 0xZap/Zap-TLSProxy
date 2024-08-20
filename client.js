const net = require("net");
const tls = require("tls");

const PROXY_HOST = "localhost";
const PROXY_PORT = 55688;

const TARGET_HOST = "www.example.com";
const TARGET_PORT = 443;

const proxySocket = net.createConnection(
  { host: PROXY_HOST, port: PROXY_PORT },
  () => {
    console.log("Connected to the proxy server");

    proxySocket.write(
      `CONNECT ${TARGET_HOST}:${TARGET_PORT} HTTP/1.1\r\nHost: ${TARGET_HOST}\r\n\r\n`
    );
  }
);

proxySocket.on("data", (data) => {
  if (data.toString().includes("200 Connection established")) {
    console.log("Proxy connected, now establishing TLS connection");

    const tlsSocket = tls.connect(
      {
        socket: proxySocket,
        servername: TARGET_HOST,
        rejectUnauthorized: false,
      },
      () => {
        console.log("TLS connection established through proxy");

        const request = `GET / HTTP/1.1\r\nHost: ${TARGET_HOST}\r\nConnection: close\r\n\r\n`;
        tlsSocket.write(request);
      }
    );

    tlsSocket.on("data", (tlsData) => {
      console.log("Received from target server:");
      console.log(tlsData.toString());
    });

    tlsSocket.on("end", () => {
      console.log("Disconnected from target server");
    });

    tlsSocket.on("error", (err) => {
      console.error("TLS socket error:", err.message);
    });
  } else {
    console.error("Failed to establish a connection through the proxy");
    proxySocket.end();
  }
});

proxySocket.on("error", (err) => {
  console.error("Proxy socket error:", err.message);
});
