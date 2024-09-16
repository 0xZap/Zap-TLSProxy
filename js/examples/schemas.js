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
        "Bearer BQBJQfF6KvbwJFoGOqCdWUMOihFmeoLoeTuMKPZfL4if6jwS5GAxJrJ9pNKIDgZim3VTTyAJWrgB8ion8IsGQXLR30dcJV83TQt-D_fJ3SrV3Q9hLf1e8WFyDZJwOSsTWYXNJeCjAicJwf31M_jU8iAFJE7Mntqu98TzI3A3AknxreiDpFg_X4K5B9h9VvSy2vaeqN3VjL_Hs5PD4Fqv9JiQtf6b8dA7u0dK1CCcZuE2dmoISorUScAe5-4KnLrRdB_Be0mVX1yz-rM3U74cXuu8XKcAxKNSmJHd5aeFu7lS8qqtgs5LX6B4a9fRQYDzgJqkMFC7",
      Connection: "close",
    },
    body: "",
    proxyHost: "localhost",
    proxyPort: 55688,
    targetHost: "api.spotify.com",
    targetPort: 443,
  },
};
