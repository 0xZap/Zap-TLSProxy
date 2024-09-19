use http::{method, Request, Uri};
use reqwest::blocking::Client as ReqwestClient;
use rustls::ProtocolVersion;
use rustls::{pki_types::ServerName, ClientConfig, KeyLogFile, RootCertStore};
use std::sync::Arc;
use std::{
    io::{self, Read, Write},
    net::TcpStream,
};

use crate::types::{Endpoint, Proof, SecretsPayload, ZapServerConfig};
use crate::utils::{extract, serialize};

pub struct HttpClient {
    client: ReqwestClient,
    config: ClientConfig,
    endpoint: Endpoint,
    zap_server_config: ZapServerConfig,
}

impl HttpClient {
    pub fn new(endpoint: Endpoint, zap_server_config: ZapServerConfig) -> Self {
        Self { client: ReqwestClient::new(), endpoint, zap_server_config, config: Self::bake_config() }
    }

    fn bake_config() -> ClientConfig {
        let root_store = RootCertStore { roots: webpki_roots::TLS_SERVER_ROOTS.into() };
        let mut config = ClientConfig::builder().with_root_certificates(root_store).with_no_client_auth();

        config.key_log = Arc::new(KeyLogFile::new());
        config.enable_secret_extraction = true;

        config
    }

    fn get_tls_version(&self, conn: &rustls::ClientConnection) -> ProtocolVersion {
        conn.protocol_version().expect("Failed to get protocol version")
    }

    fn establish_connection(&self) -> io::Result<TcpStream> {
        let serialized_request =
            serialize::request(&self.endpoint, http::Method::CONNECT, http::Version::HTTP_11);
        println!("Connect request: {:?}", serialized_request);
        let mut sock = TcpStream::connect(self.zap_server_config.get_proxy_url())?;
        sock.write_all(serialized_request.as_bytes())?;

        let mut response = [0; 4096];
        sock.read(&mut response)?;

        Ok(sock)
    }

    pub fn perform(&self) -> io::Result<Proof> {
        let server_name = ServerName::try_from(self.endpoint.get_host().to_string())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let mut conn = rustls::ClientConnection::new(Arc::new(self.config.clone()), server_name)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        let mut sock = self.establish_connection()?;

        if conn.is_handshaking() {
            conn.process_new_packets().unwrap();
        }

        let serialized_request =
            serialize::request(&self.endpoint, self.endpoint.get_method(), http::Version::HTTP_11);
        println!("Endpoint Request: {:?}", serialized_request);
        let mut tls = rustls::Stream::new(&mut conn, &mut sock);
        tls.write_all(serialized_request.as_bytes())?;

        println!("TLS version: {:?}", self.get_tls_version(&conn));

        let secrets_payload = self.extract_secrets_payload(conn)?;
        Ok(self.generate_proof(secrets_payload)?)
    }

    fn extract_secrets_payload(&self, conn: rustls::ClientConnection) -> io::Result<SecretsPayload> {
        let extracted_secrets = conn.dangerous_extract_secrets().expect("Failed to extract secrets");
        let secrets_payload = extract::secrets_payload(extracted_secrets);

        Ok(secrets_payload)
    }

    fn generate_proof(&self, secrets_payload: SecretsPayload) -> io::Result<Proof> {
        let api_endpoint = format!("http://{}/proof", self.zap_server_config.get_api_url());
        let payload_json = serde_json::to_string(&secrets_payload).expect("Failed to serialize secrets");

        let response = self
            .client
            .post(&api_endpoint)
            .header("Content-Type", "application/json")
            .body(payload_json)
            .send();

        match response {
            Ok(res) => {
                if res.status().is_success() {
                    let response_body = res.text().expect("Failed to read response body");
                    println!("Response body from proxy /proof: {:?}", response_body);
                    let proof: Proof =
                        serde_json::from_str(&response_body).expect("Failed to deserialize proof");
                    Ok(proof)
                } else {
                    Err(io::Error::new(io::ErrorKind::Other, "Failed to send secrets to proxy"))
                }
            }
            Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
        }
    }
}
