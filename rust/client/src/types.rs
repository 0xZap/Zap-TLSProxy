use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Clone)]
pub struct SecretData {
    cipher_suite: String,
    key: String,
    iv: String,
}

impl SecretData {
    pub fn new(cipher_suite: &str, key: &str, iv: &str) -> Self {
        SecretData { cipher_suite: cipher_suite.to_string(), key: key.to_string(), iv: iv.to_string() }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct SecretsPayload {
    rx_sequence_number: u64,
    tx_sequence_number: u64,
    rx_secret: SecretData,
    tx_secret: SecretData,
}

impl SecretsPayload {
    pub fn new(
        rx_sequence_number: u64,
        tx_sequence_number: u64,
        rx_secret: SecretData,
        tx_secret: SecretData,
    ) -> Self {
        SecretsPayload { rx_sequence_number, tx_sequence_number, rx_secret, tx_secret }
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct ZapServerConfig {
    host: String,
    api_port: u16,
    proxy_port: u16,
}

impl ZapServerConfig {
    pub fn new(host: &str, api_port: u16, proxy_port: u16) -> Self {
        Self { host: host.to_string(), api_port, proxy_port }
    }

    pub fn default() -> Self {
        Self::new("localhost", 8080, 55688)
    }

    pub fn get_host(&self) -> &str {
        &self.host
    }

    pub fn get_api_port(&self) -> u16 {
        self.api_port
    }

    pub fn get_proxy_port(&self) -> u16 {
        self.proxy_port
    }

    pub fn get_proxy_url(&self) -> String {
        format!("{}:{}", self.host, self.proxy_port)
    }

    pub fn get_api_url(&self) -> String {
        format!("{}:{}", self.host, self.api_port)
    }
}

#[derive(Debug, Clone)]
pub struct Endpoint {
    host: String,
    port: u16,
    route: String,
    method: http::Method,
    headers: Vec<(String, String)>,
}

impl Endpoint {
    pub fn new(
        host: &str,
        port: u16,
        route: &str,
        method: http::Method,
        headers: Vec<(String, String)>,
    ) -> Self {
        Endpoint { host: host.to_string(), port, route: route.to_string(), method, headers }
    }

    pub fn default() -> Self {
        Endpoint::new("www.example.com", 443, "/", http::Method::GET, vec![])
    }

    pub fn get_host(&self) -> &str {
        &self.host
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }

    pub fn get_url(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn get_route(&self) -> &str {
        &self.route
    }

    pub fn get_method(&self) -> http::Method {
        self.method.clone()
    }

    pub fn get_headers(&self) -> Vec<(String, String)> {
        self.headers.clone()
    }
}

#[derive(Debug, Clone)]
pub struct EndpointBuilder {
    host: Option<String>,
    port: Option<u16>,
    route: Option<String>,
    method: Option<http::Method>,
    headers: Vec<(String, String)>,
}

impl EndpointBuilder {
    pub fn new() -> Self {
        EndpointBuilder { host: None, port: None, route: None, method: None, headers: vec![] }
    }

    pub fn host(mut self, host: &str) -> Self {
        self.host = Some(host.to_string());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn route(mut self, route: &str) -> Self {
        self.route = Some(route.to_string());
        self
    }

    pub fn method(mut self, method: http::Method) -> Self {
        self.method = Some(method);
        self
    }

    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.to_string(), value.to_string()));
        self
    }

    pub fn build(self) -> Endpoint {
        Endpoint::new(
            self.host.as_deref().expect("Host is required"),
            self.port.expect("Port is required"),
            self.route.as_deref().unwrap_or("/"),
            self.method.expect("Method is required"),
            self.headers,
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Proof {
    data: String,
    signature: String,
}

impl Proof {
    pub fn new(data: &str, signature: &str) -> Self {
        Proof { data: data.to_string(), signature: signature.to_string() }
    }

    pub fn get_data(&self) -> &str {
        &self.data
    }

    pub fn get_signature(&self) -> &str {
        &self.signature
    }
}
