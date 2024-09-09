<img src="src/assets/icon-128.png" width="64"/>

# Zap TLS Proxy

> [!IMPORTANT]
> ⚠️ Current version is not safe for private data, we still not implemented ZKPs

## Installing and Running 

### Fake Proxy using Javascript

In this version we are not able to extract key material or customize the tls connection.

1. Open the `js` folder
2. Run the proxy (websocket)
   1. Access `proxy` folder
   2. Run `node proxy.js`
3. Run a client example
   1. Access `examples` folder
   2. Run `node spotify.js`
4. Now see the log file in the `utils` folder.

Schemas can be adjusted in the `schemas.js`.

Exporting:

1. Run `npm run build` in the root
2. Run `npm link`
3. Go to the desired project to use it
4. Run `npm link zap-proxy-sdk`

Then the project will be able to use the local SDK

### Real Proxy using Rust

1. Open the `rust` folder
2. Run the proxy (websocket)
   1. Run `cargo run -p proxy`
3. Run the client
   2. Run `cargo run -p client`
4. Now see the log file in the `utils` folder.

