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
use hex;
use tokio::sync::Mutex;
use aes_gcm::{Aes256Gcm, Key, Nonce}; // Import the AES-GCM cipher
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::Error as AesGcmError;
use std::error::Error as StdError;
use anyhow::{Result, Context};
use std::fmt;

const LISTEN_PORT: u16 = 55688;
const LOG_FILE: &str = "utils/proxy.log";
const PRIVATE_KEY_FILE: &str = "utils/private-key.pem";

#[derive(Debug)]
enum DecryptError {
    HexDecodeError(hex::FromHexError),
    AesGcmError(aes_gcm::Error),
    // Add other error variants as needed
}

impl fmt::Display for DecryptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecryptError::HexDecodeError(e) => write!(f, "Hex decode error: {}", e),
            DecryptError::AesGcmError(e) => write!(f, "AES-GCM error: {}", e),
            // Handle other variants
        }
    }
}

impl StdError for DecryptError {}

impl From<hex::FromHexError> for DecryptError {
    fn from(err: hex::FromHexError) -> DecryptError {
        DecryptError::HexDecodeError(err)
    }
}

impl From<aes_gcm::Error> for DecryptError {
    fn from(err: aes_gcm::Error) -> DecryptError {
        DecryptError::AesGcmError(err)
    }
}

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

                            // Attempt to decrypt the logged data
                            {
                                let data_log = data_log.lock().await;
                                // match decrypt_data(&data_log, &proof_data) {
                                //     Ok(plaintext) => {
                                //         println!("Decrypted data:\n{}", plaintext);
                                //     }
                                //     Err(e) => {
                                //         eprintln!("Decryption failed: {:?}", e);
                                //     }
                                // }
                                let decrypted_data = mock_decrypt(&data_log, &proof_data);
                                println!("Decrypted data: {}", decrypted_data);
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

// fn decrypt_data(
//     data_log: &[(Direction, Vec<u8>)],
//     secrets: &SecretsPayload,
// ) -> Result<String, DecryptError> {
//     // Initialize sequence numbers
//     let mut rx_sequence_number = secrets.rx_sequence_number;
//     let mut tx_sequence_number = secrets.tx_sequence_number;

//     let rx_key = hex::decode(&secrets.rx_secret.key)?;
//     let rx_iv = hex::decode(&secrets.rx_secret.iv)?;
//     let tx_key = hex::decode(&secrets.tx_secret.key)?;
//     let tx_iv = hex::decode(&secrets.tx_secret.iv)?;

//     let rx_cipher = Aes256Gcm::new_from_slice(&rx_key)?;
//     let tx_cipher = Aes256Gcm::new_from_slice(&tx_key)?;

//     let mut plaintext = Vec::new();

//     for (direction, data) in data_log {
//         let (cipher, iv, sequence_number) = match direction {
//             Direction::ClientToServer => (&tx_cipher, &tx_iv, &mut tx_sequence_number),
//             Direction::ServerToClient => (&rx_cipher, &rx_iv, &mut rx_sequence_number),
//         };

//         let mut cursor = &data[..];

//         while cursor.len() >= 5 {
//             // Parse TLS record header
//             let content_type = cursor[0];
//             let version = &cursor[1..3];
//             let length = u16::from_be_bytes([cursor[3], cursor[4]]) as usize;

//             // Ensure we have the full record
//             if cursor.len() < 5 + length {
//                 break; // Incomplete record
//             }

//             let header = &cursor[..5];
//             let encrypted_record = &cursor[5..5 + length];

//             // Compute nonce
//             let mut nonce = vec![0u8; 12]; // 96-bit nonce
//             nonce[..4].copy_from_slice(&[0u8; 4]); // First 4 bytes are zeros
//             let seq_num_bytes = (*sequence_number).to_be_bytes();
//             nonce[4..12].copy_from_slice(&seq_num_bytes);

//             for i in 0..12 {
//                 nonce[i] ^= iv[i];
//             }

//             let nonce = Nonce::from_slice(&nonce);

//             // Decrypt the record
//             let decrypted_data = cipher.decrypt(
//                 nonce,
//                 aes_gcm::aead::Payload {
//                     msg: encrypted_record,
//                     aad: header,
//                 },
//             );

//             match decrypted_data {
//                 Ok(mut data) => {
//                     // Remove content type and padding
//                     if let Some(&last_byte) = data.last() {
//                         data.pop(); // Remove content type byte
//                         // Remove padding (zeros before content type)
//                         while data.last() == Some(&0) {
//                             data.pop();
//                         }
//                     }
//                     plaintext.extend_from_slice(&data);
//                 }
//                 Err(e) => {
//                     eprintln!("Decryption failed: {:?}", e);
//                 }
//             }

//             // Advance cursor
//             cursor = &cursor[5 + length..];

//             // Increment sequence number
//             *sequence_number += 1;
//         }
//     }

//     let plaintext_str = String::from_utf8_lossy(&plaintext).to_string();
//     Ok(plaintext_str)
// }

fn mock_decrypt( 
    data_log: &[(Direction, Vec<u8>)],
    secrets: &SecretsPayload,
) -> String {
    // Mock decryption logic
    // In reality, you'd use the secrets to decrypt the data
    format!(
        "Mock decrypted data using cipher: {}",
        secrets.rx_secret.cipher_suite
    )
}