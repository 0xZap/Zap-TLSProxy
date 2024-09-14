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
        "Bearer BQA2jcnotPIhSTsMoHV4p-M6zUSaQXYs7HQTAYuyIOdzDnnj_-iCpnLPR0ajdmdmHKKwBkl3ReVjVIzOU3V8jPnYrBIP1nrQkOhzTicBVpN0AiJLoT8A0oRT1-GtvO-Ix9I6KRSjcv3E7pP4RoQQIcFwCbaKU6f7PyFJXIRUzNiuA-Igb1tHZRuQH99dZArcWklS_csIb1m7zZPzXq9iplRUf892hjXFs_qg7jgWLJDkfMudTGCgNQR6p3MWqoB9BuqxGvLglYTIbyF29uNZ4wSfilynYIICFPKSoaMlylhNnCHzDIHSzgs1KuviblL5BS6eC7Z_",
      Connection: "close",
    },
    body: "",
    proxyHost: "localhost",
    proxyPort: 55688,
    targetHost: "api.spotify.com",
    targetPort: 443,
  },
};
