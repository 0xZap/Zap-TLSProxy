pub mod serialize {
    use crate::types::Endpoint;

    pub fn request(endpoint: &Endpoint, http_method: http::Method, http_version: http::Version) -> String {
        let mut request_string;

        if http_method == http::Method::CONNECT {
            request_string = format!(
                "{} {}:{} {:?}\r\n",
                http_method,
                endpoint.get_host(),
                endpoint.get_port(),
                http_version
            );
        } else {
            request_string = format!("{} {} {:?}\r\n", http_method, endpoint.get_route(), http_version);
        }

        let mut headers = endpoint.get_headers().clone();
        headers.push(("Host".to_string(), endpoint.get_host().to_string()));
        headers.push(("Connection".to_string(), "close".to_string()));

        for (name, value) in headers {
            request_string.push_str(&format!("{}: {}\r\n", name, value));
        }

        request_string.push_str("\r\n");

        request_string
    }
}

pub mod extract {
    use crate::types::{SecretData, SecretsPayload};
    use rustls::{ConnectionTrafficSecrets, ExtractedSecrets};

    fn secret_data(secret: ConnectionTrafficSecrets) -> SecretData {
        match secret {
            ConnectionTrafficSecrets::Aes128Gcm { ref key, ref iv } => {
                SecretData::new("Aes128Gcm", &hex::encode(key.as_ref()), &hex::encode(iv.as_ref()))
            }
            ConnectionTrafficSecrets::Aes256Gcm { ref key, ref iv } => {
                SecretData::new("Aes256Gcm", &hex::encode(key.as_ref()), &hex::encode(iv.as_ref()))
            }
            ConnectionTrafficSecrets::Chacha20Poly1305 { ref key, ref iv } => {
                SecretData::new("Chacha20Poly1305", &hex::encode(key.as_ref()), &hex::encode(iv.as_ref()))
            }
            _ => panic!("Unsupported cipher suite, unable to extract secrets"),
        }
    }

    pub fn secrets_payload(extracted_secrets: ExtractedSecrets) -> SecretsPayload {
        let (rx_sequence_number, rx_secret) = extracted_secrets.rx;
        let (tx_sequence_number, tx_secret) = extracted_secrets.tx;

        SecretsPayload::new(
            rx_sequence_number,
            tx_sequence_number,
            secret_data(rx_secret),
            secret_data(tx_secret),
        )
    }
}
