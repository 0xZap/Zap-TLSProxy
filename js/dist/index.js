"use strict";

function _typeof(o) { "@babel/helpers - typeof"; return _typeof = "function" == typeof Symbol && "symbol" == typeof Symbol.iterator ? function (o) { return typeof o; } : function (o) { return o && "function" == typeof Symbol && o.constructor === Symbol && o !== Symbol.prototype ? "symbol" : typeof o; }, _typeof(o); }
function _slicedToArray(r, e) { return _arrayWithHoles(r) || _iterableToArrayLimit(r, e) || _unsupportedIterableToArray(r, e) || _nonIterableRest(); }
function _nonIterableRest() { throw new TypeError("Invalid attempt to destructure non-iterable instance.\nIn order to be iterable, non-array objects must have a [Symbol.iterator]() method."); }
function _unsupportedIterableToArray(r, a) { if (r) { if ("string" == typeof r) return _arrayLikeToArray(r, a); var t = {}.toString.call(r).slice(8, -1); return "Object" === t && r.constructor && (t = r.constructor.name), "Map" === t || "Set" === t ? Array.from(r) : "Arguments" === t || /^(?:Ui|I)nt(?:8|16|32)(?:Clamped)?Array$/.test(t) ? _arrayLikeToArray(r, a) : void 0; } }
function _arrayLikeToArray(r, a) { (null == a || a > r.length) && (a = r.length); for (var e = 0, n = Array(a); e < a; e++) n[e] = r[e]; return n; }
function _iterableToArrayLimit(r, l) { var t = null == r ? null : "undefined" != typeof Symbol && r[Symbol.iterator] || r["@@iterator"]; if (null != t) { var e, n, i, u, a = [], f = !0, o = !1; try { if (i = (t = t.call(r)).next, 0 === l) { if (Object(t) !== t) return; f = !1; } else for (; !(f = (e = i.call(t)).done) && (a.push(e.value), a.length !== l); f = !0); } catch (r) { o = !0, n = r; } finally { try { if (!f && null != t["return"] && (u = t["return"](), Object(u) !== u)) return; } finally { if (o) throw n; } } return a; } }
function _arrayWithHoles(r) { if (Array.isArray(r)) return r; }
function _classCallCheck(a, n) { if (!(a instanceof n)) throw new TypeError("Cannot call a class as a function"); }
function _defineProperties(e, r) { for (var t = 0; t < r.length; t++) { var o = r[t]; o.enumerable = o.enumerable || !1, o.configurable = !0, "value" in o && (o.writable = !0), Object.defineProperty(e, _toPropertyKey(o.key), o); } }
function _createClass(e, r, t) { return r && _defineProperties(e.prototype, r), t && _defineProperties(e, t), Object.defineProperty(e, "prototype", { writable: !1 }), e; }
function _toPropertyKey(t) { var i = _toPrimitive(t, "string"); return "symbol" == _typeof(i) ? i : i + ""; }
function _toPrimitive(t, r) { if ("object" != _typeof(t) || !t) return t; var e = t[Symbol.toPrimitive]; if (void 0 !== e) { var i = e.call(t, r || "default"); if ("object" != _typeof(i)) return i; throw new TypeError("@@toPrimitive must return a primitive value."); } return ("string" === r ? String : Number)(t); }
var WebSocketClass;
if (typeof window !== "undefined" && typeof window.WebSocket !== "undefined") {
  // Ambiente do navegador
  WebSocketClass = window.WebSocket;
} else {
  // Ambiente do Node.js
  WebSocketClass = require("ws");
}
var _require = require("uuid"),
  uuidv4 = _require.v4;
var ZapProxy = /*#__PURE__*/function () {
  function ZapProxy(proxyHost, proxyPort, targetHost, targetPort) {
    _classCallCheck(this, ZapProxy);
    this.proxyHost = proxyHost;
    this.proxyPort = proxyPort;
    this.targetHost = targetHost;
    this.targetPort = targetPort;
    this.accumulatedData = [];
    this.proofId = uuidv4();
    this.HTTP_PORT = 8080; // Define HTTP_PORT here
  }
  return _createClass(ZapProxy, [{
    key: "connect",
    value: function connect() {
      var _this = this;
      return new Promise(function (resolve, reject) {
        _this.socket = new WebSocketClass("ws://".concat(_this.proxyHost, ":").concat(_this.proxyPort));
        _this.socket.onopen = function () {
          _this.socket.send("CONNECT ".concat(_this.targetHost, ":").concat(_this.targetPort, " HTTP/1.1\r\nHost: ").concat(_this.targetHost, "\r\n\r\n"));
        };
        _this.socket.onmessage = function (event) {
          var data = event.data;

          // Verifica se o dado é um Blob no navegador e converte para texto
          if (data instanceof Blob) {
            data.text().then(function (text) {
              if (text.includes("200 Connection established")) {
                resolve();
              } else {
                reject(new Error("Failed to establish a connection through the proxy"));
              }
            })["catch"](function (err) {
              return reject(new Error("Error processing Blob: " + err.message));
            });
          } else {
            // Caso não seja Blob, processa diretamente
            var textData = data.toString();
            if (textData.includes("200 Connection established")) {
              resolve();
            } else {
              reject(new Error("Failed to establish a connection through the proxy"));
            }
          }
        };
        _this.socket.onerror = function (err) {
          reject(new Error("WebSocket error: " + err.message));
        };
      });
    }
  }, {
    key: "createHttpRequest",
    value: function createHttpRequest(_ref) {
      var method = _ref.method,
        url = _ref.url,
        _ref$headers = _ref.headers,
        headers = _ref$headers === void 0 ? {} : _ref$headers,
        _ref$body = _ref.body,
        body = _ref$body === void 0 ? "" : _ref$body;
      var fullPath = url.startsWith("http") ? new URL(url).pathname : url;
      var request = "".concat(method, " ").concat(fullPath, " HTTP/1.1\r\nHost: ").concat(this.targetHost, "\r\n");
      for (var _i = 0, _Object$entries = Object.entries(headers); _i < _Object$entries.length; _i++) {
        var _Object$entries$_i = _slicedToArray(_Object$entries[_i], 2),
          key = _Object$entries$_i[0],
          value = _Object$entries$_i[1];
        request += "".concat(key, ": ").concat(value, "\r\n");
      }
      if (body && method !== "GET") {
        request += "Content-Length: ".concat(new TextEncoder().encode(body).length, "\r\n\r\n");
        request += body;
      } else {
        request += "\r\n";
      }
      return request;
    }
  }, {
    key: "prove",
    value: function prove(schemaJson) {
      var _this2 = this;
      return new Promise(function (resolve, reject) {
        if (_this2.socket.readyState === 1) {
          var request = _this2.createHttpRequest(schemaJson);
          _this2.socket.send(request);
          _this2.socket.onmessage = function (event) {
            var data = event.data;

            // Verifica se o dado é um Blob no navegador
            if (data instanceof Blob) {
              data.text().then(function (text) {
                _this2.accumulatedData.push(text);
              })["catch"](function (err) {
                return reject(new Error("Error processing Blob: " + err.message));
              });
            } else {
              _this2.accumulatedData.push(data.toString());
            }
          };
          _this2.socket.onclose = function () {
            resolve(_this2.sendProof(schemaJson.id));
          };
          _this2.socket.onerror = function (err) {
            reject(new Error("Error during WebSocket communication: " + err.message));
          };
        } else {
          reject(new Error("WebSocket connection is not established"));
        }
      });
    }
  }, {
    key: "sendProof",
    value: function sendProof(schemaId) {
      var _this3 = this;
      return new Promise(function (resolve, reject) {
        var accumulatedDataStr = _this3.accumulatedData.join("");
        var bodyStartIndex = accumulatedDataStr.indexOf("\r\n\r\n") + 4;
        var responseBody = accumulatedDataStr.slice(bodyStartIndex);
        var filteredData;
        var msg = {};
        var names = {};
        switch (schemaId) {
          case "spotify":
            try {
              var _filteredData;
              var parsedData = JSON.parse(responseBody);
              var items = parsedData.items;
              filteredData = items === null || items === void 0 ? void 0 : items.map(function (item) {
                return {
                  trackId: item.track.id,
                  trackName: item.track.name,
                  artists: item.track.artists.map(function (artist) {
                    return {
                      artistId: artist.id,
                      artistName: artist.name
                    };
                  }),
                  durationMs: item.track.duration_ms,
                  playedAt: item.played_at
                };
              });
              (_filteredData = filteredData) === null || _filteredData === void 0 || _filteredData.forEach(function (item) {
                var firstArtist = item.artists[0];
                if (firstArtist) {
                  var artistId = firstArtist.artistId;
                  if (msg[artistId]) {
                    msg[artistId] += 1;
                  } else {
                    msg[artistId] = 1;
                  }
                  var artistName = firstArtist.artistName;
                  names[artistId] = artistName;
                }
              });
              if (!filteredData) {
                return reject(new Error("Could not find 'items' data in the response"));
              }
            } catch (err) {
              return reject(new Error("Failed to parse 'spotify' response: ".concat(err.message)));
            }
            break;
          default:
            filteredData = responseBody;
        }
        var jsonResponse = JSON.stringify({
          proofId: _this3.proofId,
          schemaId: schemaId,
          proofData: filteredData,
          message: msg,
          tokens: names
        });

        // Use the HTTP_PORT defined in the constructor
        fetch("http://".concat(_this3.proxyHost, ":").concat(_this3.HTTP_PORT, "/proof"), {
          method: "POST",
          headers: {
            "Content-Type": "application/json"
          },
          body: jsonResponse
        }).then(function (response) {
          return response.json();
        }).then(function (data) {
          if (data) {
            resolve(data);
          } else {
            reject(new Error("Failed to send proof: Unknown error"));
          }
        })["catch"](function (err) {
          reject(new Error("Failed to send proof: " + err.message));
        });
      });
    }
  }]);
}();
module.exports = ZapProxy;