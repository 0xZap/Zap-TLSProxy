use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use std::fs::{self, OpenOptions};
use std::sync::Arc;
use std::io::Write;
use openssl::pkey::PKey;
use openssl::sign::Signer;
use openssl::hash::MessageDigest;
use hyper::{Body, Response, Server, Method};
use hyper::service::{make_service_fn, service_fn};
use serde::{Serialize, Deserialize};
use chrono;
use tokio::sync::Mutex;
use aes_gcm::{aead::{Aead, KeyInit, Payload}, Aes256Gcm, Nonce, Key}; 
use hex::{decode, encode}; 
use anyhow::{Result, Context};
use std::str;

const LISTEN_PORT: u16 = 55688;
const LOG_FILE: &str = "utils/proxy.log";
const PRIVATE_KEY_FILE: &str = "utils/private-key.pem";

#[derive(Clone, Copy)]
enum Direction {
    ClientToServer,
    ServerToClient,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SecretsPayload {
    rx_sequence_number: u64,
    tx_sequence_number: u64,
    rx_secret: SecretData,
    tx_secret: SecretData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SecretData {
    cipher_suite: String,
    key: String,
    iv: String,
}

async fn handle_client(
    mut client_socket: TcpStream,
    private_key: Arc<PKey<openssl::pkey::Private>>,
    data_log: Arc<Mutex<Vec<(Direction, Vec<u8>)>>>,
) -> io::Result<()> {
    let mut buffer = vec![0; 1024];
    let n = client_socket.read(&mut buffer).await?;
    if n == 0 {
        // Client closed the connection
        return Ok(());
    }

    if let Some((target_host, target_port)) = parse_connect_request(&buffer[..n]) {
        log_data(&format!("CONNECT request to {}:{}", target_host, target_port));

        let signature = sign_data(
            &private_key,
            format!("CONNECT request to {}:{}", target_host, target_port).as_bytes(),
        );
        log_data(&format!("Signature: {:?}", signature));

        let target_addr = format!("{}:{}", target_host, target_port);
        if let Ok(mut target_socket) = TcpStream::connect(target_addr).await {
            client_socket
                .write_all(b"HTTP/1.1 200 Connection established\r\n\r\n")
                .await?;

            let (mut client_reader, mut client_writer) = client_socket.split();
            let (mut target_reader, mut target_writer) = target_socket.split();

            let data_log_clone1 = data_log.clone();
            let data_log_clone2 = data_log.clone();

            let client_to_target = log_and_copy(
                &mut client_reader,
                &mut target_writer,
                Direction::ClientToServer,
                data_log_clone1,
            );
            let target_to_client = log_and_copy(
                &mut target_reader,
                &mut client_writer,
                Direction::ServerToClient,
                data_log_clone2,
            );

            tokio::try_join!(client_to_target, target_to_client)?;
        } else {
            eprintln!("Failed to connect to target server");
        }
    } else {
        eprintln!("Invalid CONNECT request");
    }

    Ok(())
}

async fn log_and_copy<R, W>(
    reader: &mut R,
    writer: &mut W,
    direction: Direction,
    data_log: Arc<Mutex<Vec<(Direction, Vec<u8>)>>>,
) -> io::Result<()>
where
    R: AsyncReadExt + Unpin,
    W: AsyncWriteExt + Unpin,
{
    let mut buffer = [0; 4096];
    loop {
        let n = reader.read(&mut buffer).await?;
        if n == 0 {
            break;
        }

        log_data(&format!(
            "[{}] {}",
            chrono::Utc::now().to_rfc3339(),
            match direction {
                Direction::ClientToServer => "Encrypted data sent to server:",
                Direction::ServerToClient => "Encrypted data received from server:",
            }
        ));
        log_data(&hex::encode(&buffer[..n]));

        {
            let mut data_log = data_log.lock().await;
            data_log.push((direction, buffer[..n].to_vec()));
        }

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

    let data_log = Arc::new(Mutex::new(Vec::<(Direction, Vec<u8>)>::new()));
    let secrets = Arc::new(Mutex::new(None));

    let private_key_clone = private_key.clone();
    let data_log_clone = data_log.clone();
    let secrets_clone = secrets.clone();

    tokio::spawn(async move {
        run_http_server(private_key_clone, secrets_clone, data_log_clone).await;
    });

    let listener = TcpListener::bind(("0.0.0.0", LISTEN_PORT)).await?;
    println!("Proxy server listening on port {}", LISTEN_PORT);

    loop {
        let (client_socket, _) = listener.accept().await?;
        let private_key = private_key.clone();
        let data_log = data_log.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_client(client_socket, private_key, data_log).await {
                eprintln!("Failed to handle client: {}", e);
            }
        });
    }
}

async fn run_http_server(
    private_key: Arc<PKey<openssl::pkey::Private>>,
    secrets: Arc<Mutex<Option<SecretsPayload>>>,
    data_log: Arc<Mutex<Vec<(Direction, Vec<u8>)>>>,
) {
    let make_svc = make_service_fn(|_conn| {
        let secrets = secrets.clone();
        let data_log = data_log.clone();

        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let secrets = secrets.clone();
                let data_log = data_log.clone();

                async move {
                    match (req.method(), req.uri().path()) {
                        (&Method::POST, "/proof") => {
                            let body_bytes = match hyper::body::to_bytes(req.into_body()).await {
                                Ok(bytes) => bytes,
                                Err(_) => {
                                    return Ok::<_, hyper::Error>(
                                        Response::builder()
                                            .status(400)
                                            .body(Body::from("Failed to read request body"))
                                            .unwrap(),
                                    );
                                }
                            };
                
                            let proof_data: SecretsPayload = match serde_json::from_slice(&body_bytes) {
                                Ok(data) => data,
                                Err(_) => {
                                    return Ok::<_, hyper::Error>(
                                        Response::builder()
                                            .status(400)
                                            .body(Body::from("Invalid JSON"))
                                            .unwrap(),
                                    );
                                }
                            };

                            // Log and store the received secrets
                            println!("Received proof data: {:#?}", proof_data);

                            // Attempt to decrypt the logged data
                            {
                                let data_log = data_log.lock().await;
                                let decrypted_data = decrypt_data(&data_log, &proof_data);
                                println!("Decrypted data: {:?}", decrypted_data);
                            }

                            Ok::<_, hyper::Error>(
                                Response::new(Body::from("Proof data received")),
                            )
                        }
                        _ => Ok::<_, hyper::Error>(
                            Response::builder()
                                .status(404)
                                .body(Body::from("Not Found"))
                                .unwrap(),
                        ),
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

fn decrypt_data(
    data_log: &[(Direction, Vec<u8>)],
    secrets: &SecretsPayload,
) -> Result<Vec<String>> {
    let mut decrypted_strings: Vec<String> = Vec::new();

    if let Some((_direction, data)) = data_log.last() {
        let hex_data: String = data.iter().map(|byte| format!("{:02x}", byte)).collect();

        // Search for and split by the sequence '170303' in the hex data
        let chunks: Vec<&str> = hex_data.split("170303").collect();

        let mut sequence_number: u64 = 2; 

        let _tx_iv = decode(&secrets.tx_secret.iv).context("Failed to decode tx iv")?;
        let _tx_key = decode(&secrets.tx_secret.key).context("Failed to decode tx key")?;

        let rx_iv = decode(&secrets.rx_secret.iv).context("Failed to decode rx iv")?;
        let rx_key = decode(&secrets.rx_secret.key).context("Failed to decode rx key")?;
        
        for chunk in chunks.iter().filter(|&&chunk| !chunk.is_empty()) {
            if chunk.len() > 2 * 2 {  // Each byte is represented by 2 hex characters
                let cyphertext_hex = &chunk[2 * 2..];
                let aad_hex = format!("170303{}", &chunk[..2 * 2]);

                let aad = hex::decode(&aad_hex)
                    .context("Failed to decode AAD hex to bytes")?;
                let ciphertext = hex::decode(cyphertext_hex)
                    .context("Failed to decode ciphertext hex to bytes")?;

                let payload = Payload {
                    msg: &ciphertext[..],
                    aad: &aad[..],
                };

                let mut nonce = vec![0u8; 12];
                let seq_num_bytes = sequence_number.to_be_bytes();

                nonce[4..12].copy_from_slice(&seq_num_bytes);

                for i in 0..12 {
                    nonce[i] ^= rx_iv[i]; 
                }

                let nonce = Nonce::from_slice(&nonce); 
                let key = Key::<Aes256Gcm>::from_slice(&rx_key[..]);

                let cipher = Aes256Gcm::new(&key);
                let decrypted = cipher.decrypt(&nonce, payload).map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;   
                
                if let Ok(readable_text) = std::str::from_utf8(&decrypted) {
                    decrypted_strings.push(readable_text.to_string());
                } else {
                    decrypted_strings.push("Decryption error".to_string());
                }

                sequence_number += 1;
            }
        }
    }  else {
        println!("data_log is empty.");
    }
    
    Ok(decrypted_strings) 
}
