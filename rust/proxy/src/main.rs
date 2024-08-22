use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use std::fs::{self, OpenOptions};
use std::sync::Arc;
use std::io::Write;
use openssl::pkey::PKey;
use openssl::sign::Signer;
use openssl::hash::MessageDigest;
use hyper::{Body, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use serde_json::json;
use chrono;
use hex;

const LISTEN_PORT: u16 = 55688;
const LOG_FILE: &str = "utils/proxy.log";
const PRIVATE_KEY_FILE: &str = "utils/private-key.pem";

async fn handle_client(mut client_socket: TcpStream, private_key: Arc<PKey<openssl::pkey::Private>>) -> io::Result<()> {
    let mut buffer = vec![0; 1024];
    client_socket.read(&mut buffer).await?;

    if let Some((target_host, target_port)) = parse_connect_request(&buffer) {
        log_data(&format!("CONNECT request to {}:{}", target_host, target_port));

        let signature = sign_data(&private_key, format!("CONNECT request to {}:{}", target_host, target_port).as_bytes());
        log_data(&format!("Signature: {:?}", signature));

        let target_addr = format!("{}:{}", target_host, target_port);
        if let Ok(mut target_socket) = TcpStream::connect(target_addr).await {
            client_socket.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n").await?;

            let (mut client_reader, mut client_writer) = client_socket.split();
            let (mut target_reader, mut target_writer) = target_socket.split();

            let client_to_target = log_and_copy(&mut client_reader, &mut target_writer, "Encrypted data sent to server:");
            let target_to_client = log_and_copy(&mut target_reader, &mut client_writer, "Encrypted data received from server:");

            tokio::try_join!(client_to_target, target_to_client)?;
        } else {
            eprintln!("Failed to connect to target server");
        }
    } else {
        eprintln!("Invalid CONNECT request");
    }

    Ok(())
}

async fn log_and_copy<R, W>(reader: &mut R, writer: &mut W, direction: &str) -> io::Result<()>
where
    R: AsyncReadExt + Unpin,
    W: AsyncWriteExt + Unpin,
{
    let mut buffer = [0; 1024];
    loop {
        let n = reader.read(&mut buffer).await?;
        if n == 0 {
            break;
        }

        log_data(&format!("[{}] {}", chrono::Utc::now().to_rfc3339(), direction));
        log_data(&hex::encode(&buffer[..n]));

        writer.write_all(&buffer[..n]).await?;
    }
    Ok(())
}

fn parse_connect_request(buffer: &[u8]) -> Option<(String, u16)> {
    let request = String::from_utf8_lossy(buffer);
    let re = regex::Regex::new(r"^CONNECT\s+([^\s:]+):(\d+)\s+HTTP/1\.1").unwrap();
    if let Some(captures) = re.captures(&request) {
        let host = captures.get(1).map_or("", |m| m.as_str()).to_string();
        let port = captures.get(2).map_or("0", |m| m.as_str()).parse().unwrap_or(0);
        return Some((host, port));
    }
    None
}

fn log_data(data: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(LOG_FILE)
        .expect("Unable to open log file");
    writeln!(file, "{}", data).expect("Unable to write to log file");
}

fn sign_data(private_key: &PKey<openssl::pkey::Private>, data: &[u8]) -> Vec<u8> {
    let mut signer = Signer::new(MessageDigest::sha256(), private_key).expect("Failed to create signer");
    signer.update(data).expect("Failed to update signer with data");
    signer.sign_to_vec().expect("Failed to sign data")
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let private_key_pem = fs::read(PRIVATE_KEY_FILE).expect("Unable to read private key file");
    let private_key = PKey::private_key_from_pem(&private_key_pem).expect("Failed to load private key");
    let private_key = Arc::new(private_key);

    let private_key_clone = private_key.clone();
    tokio::spawn(async move {
        run_http_server(private_key_clone).await;
    });

    let listener = TcpListener::bind(("0.0.0.0", LISTEN_PORT)).await?;
    println!("Proxy server listening on port {}", LISTEN_PORT);

    loop {
        let (client_socket, _) = listener.accept().await?;
        let private_key = private_key.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(client_socket, private_key).await {
                eprintln!("Failed to handle client: {}", e);
            }
        });
    }
}

async fn run_http_server(private_key: Arc<PKey<openssl::pkey::Private>>) {
    let make_svc = make_service_fn(|_conn| {
        let private_key = private_key.clone();
        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let private_key = private_key.clone();
                async move {
                    if req.method() == hyper::Method::GET && req.uri().path() == "/signed-log" {
                        match fs::read_to_string(LOG_FILE) {
                            Ok(log_data) => {
                                let signature = sign_data(&private_key, log_data.as_bytes());
                                let response = json!({
                                    "log": log_data,
                                    "signature": hex::encode(signature)
                                });
                                Ok::<_, hyper::Error>(Response::new(Body::from(response.to_string())))
                            },
                            Err(e) => Ok::<_, hyper::Error>(Response::builder()
                                .status(500)
                                .body(Body::from(format!("Error reading log file: {}", e)))
                                .unwrap())
                        }
                    } else {
                        Ok::<_, hyper::Error>(Response::builder()
                            .status(404)
                            .body(Body::from("Not Found"))
                            .unwrap())
                    }
                }
            }))
        }
    });

    let addr = ([0, 0, 0, 0], 8080).into();
    let server = Server::bind(&addr).serve(make_svc);

    println!("HTTP server listening on port 8080");
    if let Err(e) = server.await {
        eprintln!("HTTP server error: {}", e);
    }
}
