module.exports = {
  exampleSchema: {
    id: "example",
    method: "GET",
    url: "/",
    headers: {
      Connection: "close",
    },
    body: "",
    proxyHost: "localhost",
    proxyPort: 55688,
    targetHost: "www.example.com",
    targetPort: 443,
  },
  nodeguardiansSchema: {
    id: "node_guardians",
    method: "GET",
    url: "/api/users/statistics?key=trees",
    headers: {
      Authorization:
        "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6ODA5NSwiaWF0IjoxNzI0NjM5MDY2LCJleHAiOjE3MjUwNzEwNjZ9.LSkDZVtNXVre_qunVGX-w3bTyTQDPyf6Yt4xub2nV9Y",
      Connection: "close",
    },
    body: "",
    proxyHost: "localhost",
    proxyPort: 55688,
    targetHost: "backend.nodeguardians.io",
    targetPort: 443,
  },
};
