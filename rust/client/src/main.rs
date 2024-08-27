use std::fs::File;
use std::io::{self, Write, stdout, Read};
use std::net::TcpStream;
use std::sync::Arc;
use rustls::{ClientConfig, RootCertStore, KeyLogFile, ConnectionTrafficSecrets, ProtocolVersion};
use rustls::pki_types::ServerName;
use hex;

fn to_hex_string(bytes: &[u8]) -> String {
    hex::encode(bytes)
}


fn main() -> io::Result<()> {
    let proxy_host = "localhost";
    let proxy_port = 55688;

    let target_host = "www.example.com";
    let target_port = 443;

    let proxy_address = format!("{}:{}", proxy_host, proxy_port);

    // Configurar TLS
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
    // Example of direct connection without proxy
    // let mut sock = TcpStream::connect(format!("{}:{}", target_host, target_port)).unwrap();
    let mut sock = TcpStream::connect(proxy_address).unwrap();

    // Enviar a requisição CONNECT para o proxy
    let connect_request = format!("CONNECT {}:{} HTTP/1.1\r\nHost: {}\r\n\r\n", target_host, target_port, target_host);
    sock.write_all(connect_request.as_bytes())?;

    // Ler a resposta do proxy
    let mut response = [0; 4096];
    sock.read(&mut response)?;

    // TLS Stream
    {
        let mut tls = rustls::Stream::new(&mut conn, &mut sock);
        tls.write_all(
            concat!(
                "GET / HTTP/1.1\r\n",
                "Host: www.example.com\r\n",
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

    // // Attempt to update traffic keys (usually after significant data transfer)
    // if conn.is_handshaking() {
    //     conn.process_new_packets().unwrap();
    // }

    conn.refresh_traffic_keys();
    conn.process_new_packets().unwrap();

    println!("Keys updated!");

    // Send remaining data and close connection
    {
        let mut tls = rustls::Stream::new(&mut conn, &mut sock);
        tls.write_all(
            concat!(
                "Connection: close\r\n",
                "\r\n"
            )
            .as_bytes(),
        )
        .unwrap();

        let mut plaintext = Vec::new();
        tls.read_to_end(&mut plaintext).unwrap();

        stdout().write_all(&plaintext).unwrap();
    }

    let extracted_secrets = conn.dangerous_extract_secrets().expect("Failed to extract secrets");
    
    // Access the RX secrets (used to decrypt data received from the server)
    let (rx_sequence_number, rx_secret) = extracted_secrets.rx;
    
    // Access the TX secrets (used to encrypt data sent to the server)
    let (tx_sequence_number, tx_secret) = extracted_secrets.tx;
    
    // Display the RX secrets
    match rx_secret {
        rustls::ConnectionTrafficSecrets::Aes128Gcm { ref key, ref iv } => {
            println!("RX Aes128Gcm Key: {}", to_hex_string(key.as_ref()));
            println!("RX Aes128Gcm IV: {}", to_hex_string(iv.as_ref()));
        },
        rustls::ConnectionTrafficSecrets::Aes256Gcm { ref key, ref iv } => {
            println!("RX Aes256Gcm Key: {}", to_hex_string(key.as_ref()));
            println!("RX Aes256Gcm IV: {}", to_hex_string(iv.as_ref()));
        },
        rustls::ConnectionTrafficSecrets::Chacha20Poly1305 { ref key, ref iv } => {
            println!("RX Chacha20Poly1305 Key: {}", to_hex_string(key.as_ref()));
            println!("RX Chacha20Poly1305 IV: {}", to_hex_string(iv.as_ref()));
        },
        _ => {
            println!("RX secret has an unknown or unsupported cipher suite.");
        }
    }
    
    // Display the TX secrets
    match tx_secret {
        rustls::ConnectionTrafficSecrets::Aes128Gcm { ref key, ref iv } => {
            println!("TX Aes128Gcm Key: {}", to_hex_string(key.as_ref()));
            println!("TX Aes128Gcm IV: {}", to_hex_string(iv.as_ref()));
        },
        rustls::ConnectionTrafficSecrets::Aes256Gcm { ref key, ref iv } => {
            println!("TX Aes256Gcm Key: {}", to_hex_string(key.as_ref()));
            println!("TX Aes256Gcm IV: {}", to_hex_string(iv.as_ref()));
        },
        rustls::ConnectionTrafficSecrets::Chacha20Poly1305 { ref key, ref iv } => {
            println!("TX Chacha20Poly1305 Key: {}", to_hex_string(key.as_ref()));
            println!("TX Chacha20Poly1305 IV: {}", to_hex_string(iv.as_ref()));
        },
        _ => {
            println!("TX secret has an unknown or unsupported cipher suite.");
        }
    }

    // Display the sequence numbers
    println!("RX Sequence Number: {}", rx_sequence_number);
    println!("TX Sequence Number: {}", tx_sequence_number);

    println!("TLS keys have been logged to the file specified by SSLKEYLOGFILE.");
    Ok(())
}
