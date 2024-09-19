use std::io;

use crate::http::HttpClient;
use crate::types::{Endpoint, Proof, ZapServerConfig};
pub struct ZapClient {
    zap_server_config: ZapServerConfig,
}

impl ZapClient {
    pub fn new(zap_server_config: ZapServerConfig) -> Self {
        Self { zap_server_config }
    }

    pub fn prove(&self, endpoint: Endpoint) -> io::Result<Proof> {
        let client = HttpClient::new(endpoint, self.zap_server_config.clone());
        client.perform()
    }
}
