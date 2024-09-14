const net = require("net");
const tls = require("tls"); // Para conexões TLS
const fs = require("fs");
const crypto = require("crypto");
const http = require("http");
const WebSocket = require("ws");
const { ethers } = require("ethers");

const LISTEN_PORT = 55688;
const HTTP_PORT = 8080;
const LOG_FILE = "../utils/proxy.log";
const PRIVATE_KEY_FILE = "../utils/private-key.pem";

function logData(data) {
  fs.appendFileSync(LOG_FILE, data + "\n", "utf8");
}

function signLog(logData) {
  const privateKey = fs.readFileSync(PRIVATE_KEY_FILE, "utf8");
  const sign = crypto.createSign("SHA256");
  sign.update(logData);
  sign.end();
  const signature = sign.sign(privateKey, "hex");
  return signature;
}

async function hashAndSignMessage(artists, songs, points, privateKey) {
  const signatures = [];

  for (let i = 0; i < artists.length; i++) {
    const message = ethers.utils.solidityPack(
      ["string", "string", "uint256"],
      [artists[i], songs[i], points[i]]
    );

    const messageHash = ethers.utils.keccak256(message);

    const wallet = new ethers.Wallet(privateKey);

    const signature = await wallet.signMessage(
      ethers.utils.arrayify(messageHash)
    );

    signatures.push(signature);
  }

  return signatures;
}

// WebSocket server para conexões de proxy
const wsServer = new WebSocket.Server({ port: LISTEN_PORT });

wsServer.on("connection", (ws) => {
  console.log("WebSocket client connected");
  let targetSocket = null;

  ws.on("message", (message) => {
    const request = message.toString();
    console.log("Received message:", request);

    // Verifica se é o primeiro CONNECT
    if (!targetSocket) {
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
        logData(message.toString("hex"));

        // Verifica se é uma conexão HTTPS (porta 443)
        const useTls = targetPort === 443;

        // Conectar ao servidor de destino
        targetSocket = (useTls ? tls : net).connect(
          {
            host: targetHost,
            port: targetPort,
            rejectUnauthorized: false, // Aceitar todos os certificados (pode melhorar em produção)
          },
          () => {
            // Notifica o cliente WebSocket de que a conexão foi estabelecida
            ws.send("HTTP/1.1 200 Connection established\r\n\r\n");

            // Repassa os dados recebidos do WebSocket para o targetSocket
            ws.on("message", (data) => {
              console.log("Forwarding data from WebSocket to target server.");
              targetSocket.write(data);
            });

            // Repassa os dados recebidos do targetSocket para o WebSocket
            targetSocket.on("data", (chunk) => {
              console.log(
                "Forwarding data from target server to WebSocket client."
              );
              ws.send(chunk); // Repassa os dados para o cliente WebSocket
            });

            // Escuta quando o servidor de destino fecha a conexão
            targetSocket.on("end", () => {
              console.log("Target server closed connection.");
              ws.close();
            });
          }
        );

        // Trata erros na conexão com o servidor de destino
        targetSocket.on("error", (err) => {
          console.error("Target socket error:", err.message);
          ws.close();
        });

        // Fecha o targetSocket quando o WebSocket é desconectado
        ws.on("close", () => {
          console.log("WebSocket client disconnected");
          targetSocket.end();
        });
      } else {
        console.error("Invalid CONNECT request:", request);
        ws.close();
      }
    } else {
      // Após a conexão CONNECT, repassa as mensagens seguintes ao targetSocket
      targetSocket.write(message); // Envia a solicitação HTTP para o servidor de destino
    }
  });

  ws.on("error", (err) => {
    console.error("WebSocket error:", err.message);
  });
});

console.log(`WebSocket proxy server listening on port ${LISTEN_PORT}`);

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
  } else if (req.method === "POST" && req.url === "/proof") {
    let body = "";

    req.on("data", (chunk) => {
      body += chunk.toString();
    });

    req.on("end", async () => {
      try {
        const jsonData = JSON.parse(body);

        const proofId = jsonData.proofId;
        const schemaId = jsonData.schemaId;
        const data = jsonData.data;
        const message = jsonData.message;

        const artists = message.artists;
        const songs = message.songs;
        const points = message.points;

        logData(
          `[${new Date().toISOString()}] Received JSON proof: ${JSON.stringify({
            proofId,
            schemaId,
            message,
          })}`
        );

        const privateKey =
          "0x07da91125f80d729fad5e33bbe4d67754d0dc6ca29d60474170df75b1fd77418";
        const signatures = await hashAndSignMessage(
          artists,
          songs,
          points,
          privateKey
        );

        res.writeHead(200, { "Content-Type": "application/json" });
        res.end(
          JSON.stringify({
            signature: signatures,
            message: message,
            proofData: { proofId, schemaId, data },
          })
        );
      } catch (err) {
        res.writeHead(400, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ status: "error", message: err.message }));
      }
    });
  } else {
    res.writeHead(404, { "Content-Type": "text/plain" });
    res.end("Not Found");
  }
});

httpServer.listen(HTTP_PORT, () => {
  console.log(`HTTP server listening on port ${HTTP_PORT}`);
});
