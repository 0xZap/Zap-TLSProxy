use std::fs::File;
use std::io::{self, Write, stdout, Read};
use std::net::TcpStream;
use std::sync::Arc;
use rustls::{ClientConfig, RootCertStore, KeyLogFile, ConnectionTrafficSecrets, ProtocolVersion};
use rustls::pki_types::ServerName;
use hex;
use reqwest::blocking::Client as HttpClient; 
use serde::Serialize; 

fn to_hex_string(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

#[derive(Serialize)]
struct SecretsPayload {
    rx_sequence_number: u64,
    tx_sequence_number: u64,
    rx_secret: SecretData,
    tx_secret: SecretData,
}

#[derive(Serialize)]
struct SecretData {
    cipher_suite: String,
    key: String,
    iv: String,
}

fn main() -> io::Result<()> {
    let proxy_host = "localhost";
    let proxy_port = 55688;

    let target_host = "www.example.com";
    let target_port = 443;

    let proxy_address = format!("{}:{}", proxy_host, proxy_port);

    let root_store = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.into(),
    };
    let mut config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    config.key_log = Arc::new(KeyLogFile::new());
    config.enable_secret_extraction = true;

    let server_name = ServerName::try_from(target_host).unwrap();

    let mut conn = rustls::ClientConnection::new(Arc::new(config), server_name.clone()).unwrap();
    
    // let mut sock = TcpStream::connect(format!("{}:{}", target_host, target_port)).unwrap();
    let mut sock = TcpStream::connect(proxy_address).unwrap();

    let connect_request = format!("CONNECT {}:{} HTTP/1.1\r\nHost: {}\r\n\r\n", target_host, target_port, target_host);
    sock.write_all(connect_request.as_bytes())?;

    let mut response = [0; 4096];
    sock.read(&mut response)?;

    if conn.is_handshaking() {
        conn.process_new_packets().unwrap();
    }

    {
        let mut tls = rustls::Stream::new(&mut conn, &mut sock);
        tls.write_all(
            concat!(
                "GET / HTTP/1.1\r\n",
                "Host: www.example.com\r\n",
                "Connection: close\r\n",
                "\r\n"
            )
            .as_bytes(),
        )
        .unwrap();
    }

    let tls_version = conn.protocol_version();
    match tls_version {
        Some(ProtocolVersion::TLSv1_2) => println!("Using TLS 1.2"),
        Some(ProtocolVersion::TLSv1_3) => println!("Using TLS 1.3"),
        _ => println!("Unknown TLS version"),
    }

    // conn.refresh_traffic_keys();
    // conn.process_new_packets().unwrap();

    // println!("Keys updated!");

    // {
    //     let mut tls = rustls::Stream::new(&mut conn, &mut sock);
    //     tls.write_all(
    //         concat!(
    //             "Connection: close\r\n",
    //             "\r\n"
    //         )
    //         .as_bytes(),
    //     )
    //     .unwrap();

    //     let mut plaintext = Vec::new();
    //     tls.read_to_end(&mut plaintext).unwrap();

    //     stdout().write_all(&plaintext).unwrap();
    // }

    let extracted_secrets = conn.dangerous_extract_secrets().expect("Failed to extract secrets");
    
    let (rx_sequence_number, rx_secret) = extracted_secrets.rx;
    let (tx_sequence_number, tx_secret) = extracted_secrets.tx;
    
    let rx_secret_data = match rx_secret {
        rustls::ConnectionTrafficSecrets::Aes128Gcm { ref key, ref iv } => SecretData {
            cipher_suite: "Aes128Gcm".to_string(),
            key: to_hex_string(key.as_ref()),
            iv: to_hex_string(iv.as_ref()),
        },
        rustls::ConnectionTrafficSecrets::Aes256Gcm { ref key, ref iv } => SecretData {
            cipher_suite: "Aes256Gcm".to_string(),
            key: to_hex_string(key.as_ref()),
            iv: to_hex_string(iv.as_ref()),
        },
        rustls::ConnectionTrafficSecrets::Chacha20Poly1305 { ref key, ref iv } => SecretData {
            cipher_suite: "Chacha20Poly1305".to_string(),
            key: to_hex_string(key.as_ref()),
            iv: to_hex_string(iv.as_ref()),
        },
        _ => {
            println!("RX secret has an unknown or unsupported cipher suite.");
            return Ok(());
        }
    };

    let tx_secret_data = match tx_secret {
        rustls::ConnectionTrafficSecrets::Aes128Gcm { ref key, ref iv } => SecretData {
            cipher_suite: "Aes128Gcm".to_string(),
            key: to_hex_string(key.as_ref()),
            iv: to_hex_string(iv.as_ref()),
        },
        rustls::ConnectionTrafficSecrets::Aes256Gcm { ref key, ref iv } => SecretData {
            cipher_suite: "Aes256Gcm".to_string(),
            key: to_hex_string(key.as_ref()),
            iv: to_hex_string(iv.as_ref()),
        },
        rustls::ConnectionTrafficSecrets::Chacha20Poly1305 { ref key, ref iv } => SecretData {
            cipher_suite: "Chacha20Poly1305".to_string(),
            key: to_hex_string(key.as_ref()),
            iv: to_hex_string(iv.as_ref()),
        },
        _ => {
            println!("TX secret has an unknown or unsupported cipher suite.");
            return Ok(());
        }
    };

    let secrets_payload = SecretsPayload {
        rx_sequence_number,
        tx_sequence_number,
        rx_secret: rx_secret_data,
        tx_secret: tx_secret_data,
    };

    let payload_json = serde_json::to_string(&secrets_payload).expect("Failed to serialize secrets");

    let proxy_url = format!("http://{}:8080/proof", proxy_host);
    let http_client = HttpClient::new();

    let response = http_client
        .post(&proxy_url)
        .header("Content-Type", "application/json")
        .body(payload_json)
        .send();

    match response {
        Ok(res) => println!("Sent secrets to proxy, received status: {}", res.status()),
        Err(err) => eprintln!("Failed to send secrets to proxy: {}", err),
    }
    
    Ok(())
}
