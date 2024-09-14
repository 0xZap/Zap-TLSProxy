let WebSocketClass;
if (typeof window !== "undefined" && typeof window.WebSocket !== "undefined") {
  // Ambiente do navegador
  WebSocketClass = window.WebSocket;
} else {
  // Ambiente do Node.js
  WebSocketClass = require("ws");
}
const { v4: uuidv4 } = require("uuid");

class ZapProxy {
  constructor(proxyHost, proxyPort, targetHost, targetPort) {
    this.proxyHost = proxyHost;
    this.proxyPort = proxyPort;
    this.targetHost = targetHost;
    this.targetPort = targetPort;
    this.accumulatedData = [];
    this.proofId = uuidv4();
    this.HTTP_PORT = 8080; // Define HTTP_PORT here
  }

  connect() {
    return new Promise((resolve, reject) => {
      this.socket = new WebSocketClass(
        `ws://${this.proxyHost}:${this.proxyPort}`
      );

      this.socket.onopen = () => {
        this.socket.send(
          `CONNECT ${this.targetHost}:${this.targetPort} HTTP/1.1\r\nHost: ${this.targetHost}\r\n\r\n`
        );
      };

      this.socket.onmessage = (event) => {
        const data = event.data;

        // Verifica se o dado é um Blob no navegador e converte para texto
        if (data instanceof Blob) {
          data
            .text()
            .then((text) => {
              if (text.includes("200 Connection established")) {
                resolve();
              } else {
                reject(
                  new Error(
                    "Failed to establish a connection through the proxy"
                  )
                );
              }
            })
            .catch((err) =>
              reject(new Error("Error processing Blob: " + err.message))
            );
        } else {
          // Caso não seja Blob, processa diretamente
          const textData = data.toString();
          if (textData.includes("200 Connection established")) {
            resolve();
          } else {
            reject(
              new Error("Failed to establish a connection through the proxy")
            );
          }
        }
      };

      this.socket.onerror = (err) => {
        reject(new Error("WebSocket error: " + err.message));
      };
    });
  }

  createHttpRequest({ method, url, headers = {}, body = "" }) {
    const fullPath = url.startsWith("http") ? new URL(url).pathname : url;
    let request = `${method} ${fullPath} HTTP/1.1\r\nHost: ${this.targetHost}\r\n`;

    for (const [key, value] of Object.entries(headers)) {
      request += `${key}: ${value}\r\n`;
    }

    if (body && method !== "GET") {
      request += `Content-Length: ${
        new TextEncoder().encode(body).length
      }\r\n\r\n`;
      request += body;
    } else {
      request += `\r\n`;
    }

    return request;
  }

  prove(schemaJson) {
    return new Promise((resolve, reject) => {
      if (this.socket.readyState === 1) {
        const request = this.createHttpRequest(schemaJson);
        this.socket.send(request);

        this.socket.onmessage = (event) => {
          const data = event.data;

          // Verifica se o dado é um Blob no navegador
          if (data instanceof Blob) {
            data
              .text()
              .then((text) => {
                this.accumulatedData.push(text);
              })
              .catch((err) =>
                reject(new Error("Error processing Blob: " + err.message))
              );
          } else {
            this.accumulatedData.push(data.toString());
          }
        };

        this.socket.onclose = () => {
          resolve(this.sendProof(schemaJson.id));
        };

        this.socket.onerror = (err) => {
          reject(
            new Error("Error during WebSocket communication: " + err.message)
          );
        };
      } else {
        reject(new Error("WebSocket connection is not established"));
      }
    });
  }

  sendProof(schemaId) {
    return new Promise((resolve, reject) => {
      const accumulatedDataStr = this.accumulatedData.join("");

      const bodyStartIndex = accumulatedDataStr.indexOf("\r\n\r\n") + 4;
      const responseBody = accumulatedDataStr.slice(bodyStartIndex);

      let filteredData;
      let organizedData;
      let artists = [];
      let songs = [];
      let points = [];

      switch (schemaId) {
        case "spotify":
          try {
            const parsedData = JSON.parse(responseBody);

            const items = parsedData.items;

            filteredData = items?.map((item) => {
              return {
                trackId: item.track.id,
                trackName: item.track.name,
                trackImage: item.track.album.images[0].url,
                artists: item.track.artists.map((artist) => ({
                  artistId: artist.id,
                  artistName: artist.name,
                })),
                durationMs: item.track.duration_ms,
                playedAt: item.played_at,
              };
            });

            organizedData = filteredData?.reduce((acc, item) => {
              const firstArtist = item.artists[0];
              if (firstArtist) {
                const existingIndex = acc.findIndex(
                  (data) =>
                    data.trackId === item.trackId &&
                    data.artistId === firstArtist.artistId
                );

                if (existingIndex === -1) {
                  songs.push(item.trackId);
                  artists.push(firstArtist.artistId);
                  points.push(1);
                  acc.push({
                    trackId: item.trackId,
                    artistId: firstArtist.artistId,
                    artistName: firstArtist.artistName,
                    trackName: item.trackName,
                    trackImage: item.trackImage,
                    durationMs: item.durationMs,
                    playedAt: item.playedAt,
                    count: 1,
                  });
                } else {
                  acc[existingIndex].count += 1;
                  points[existingIndex] += 1;
                }
              }
              return acc;
            }, []);

            // console.log("Artists:", artists);
            // console.log("Songs:", songs);
            // console.log("Points:", points);
            // console.log("Organized data:", organizedData);

            if (!organizedData) {
              return reject(
                new Error("Could not find 'items' data in the response")
              );
            }
          } catch (err) {
            return reject(
              new Error(`Failed to parse 'spotify' response: ${err.message}`)
            );
          }
          break;

        default:
          organizedData = responseBody;
      }

      const jsonResponse = JSON.stringify({
        proofId: this.proofId,
        schemaId: schemaId,
        data: organizedData,
        message: { artists, songs, points },
      });

      // Use the HTTP_PORT defined in the constructor
      fetch(`http://${this.proxyHost}:${this.HTTP_PORT}/proof`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: jsonResponse,
      })
        .then((response) => response.json())
        .then((data) => {
          if (data) {
            resolve(data);
          } else {
            reject(new Error("Failed to send proof: Unknown error"));
          }
        })
        .catch((err) => {
          reject(new Error("Failed to send proof: " + err.message));
        });
    });
  }
}

module.exports = ZapProxy;
