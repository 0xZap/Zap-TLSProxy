const net = require("net");

// Configuration for the target server
const TARGET_HOST = "api.x.com"; // Replace with the actual target host
const TARGET_PORT = 443; // Replace with the actual target port

// Configuration for the passthrough server
const LISTEN_PORT = 55688; // Port where this passthrough server will listen

// Create a server that listens for incoming connections
const server = net.createServer((clientSocket) => {
  console.log(
    "Client connected:",
    clientSocket.remoteAddress,
    clientSocket.remotePort
  );

  // Create a socket to connect to the target server
  const targetSocket = net.createConnection({
    host: TARGET_HOST,
    port: TARGET_PORT,
  });

  // When the client sends data, forward it to the target server
  clientSocket.on("data", (data) => {
    console.log("Received from client:", data.toString());
    targetSocket.write(data);
  });

  // When the target server sends data, forward it to the client
  targetSocket.on("data", (data) => {
    console.log("Received from target:", data.toString());
    clientSocket.write(data);
  });

  // Handle client socket close event
  clientSocket.on("end", () => {
    console.log("Client disconnected");
    targetSocket.end();
  });

  // Handle target socket close event
  targetSocket.on("end", () => {
    console.log("Target server disconnected");
    clientSocket.end();
  });

  // Handle errors on client socket
  clientSocket.on("error", (err) => {
    console.error("Client socket error:", err.message);
    targetSocket.end();
  });

  // Handle errors on target socket
  targetSocket.on("error", (err) => {
    console.error("Target socket error:", err.message);
    clientSocket.end();
  });
});

// Start the server and listen on the configured port
server.listen(LISTEN_PORT, () => {
  console.log(`TCP passthrough server listening on port ${LISTEN_PORT}`);
});
