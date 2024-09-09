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
  spotify: {
    id: "spotify",
    method: "GET",
    url: "/v1/me/player/recently-played?limit=50",
    headers: {
      Authorization:
        "Bearer BQA5QtbGpo2r6KKI9n9qEIbbiNEUtuS6Pe-VHb_ARCTT57te-hPeriolv6WZ9KEyIav-WpWm0rya0ZXO7Wo42gvLt-9RyvaLl0qZSaDLxWAzbtv6foog1G7nRGdi8p1jzTQmmzqtrV7f6ORR-0uNWcmZI4AVFkMOLePReUe742ChwktOrq6dh2xt-H0XzzbzfL8NiwOGygPDZrZfjciLbpCTiiumrvwSAT4niZ1ghZFCpg7metSuZV7tfNIoRwAk8_bIUODVywILl5m6phgYcNDFb49qY2bZXy2eJxbMjsHkjoe958XsnvpevaofaTjQsyKiM9Ao",
      Connection: "close",
    },
    body: "",
    proxyHost: "localhost",
    proxyPort: 55688,
    targetHost: "api.spotify.com",
    targetPort: 443,
  },
};
