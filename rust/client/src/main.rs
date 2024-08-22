use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::{TlsConnector, client::TlsStream};
use rustls::{ClientConfig, ClientConnection, KeyLogFile, OwnedTrustAnchor, RootCertStore, KeyUpdateRequest};
use std::sync::Arc;
use webpki::DNSNameRef;
use std::convert::TryFrom;

#[tokio::main]
async fn main() -> io::Result<()> {
    let proxy_host = "localhost";
    let proxy_port = 55688;

    let target_host = "www.example.com";
    let target_port = 443;

    let proxy_addr = format!("{}:{}", proxy_host, proxy_port);
    let mut proxy_socket = TcpStream::connect(proxy_addr).await?;
    println!("Connected to the proxy server");

    let connect_request = format!(
        "CONNECT {}:{} HTTP/1.1\r\nHost: {}\r\n\r\n",
        target_host, target_port, target_host
    );
    proxy_socket.write_all(connect_request.as_bytes()).await?;

    let mut response = vec![0; 1024];
    proxy_socket.read(&mut response).await?;

    if String::from_utf8_lossy(&response).contains("200 Connection established") {
        println!("Proxy connected, now establishing TLS connection");

        let mut tls_stream = establish_tls_connection(target_host, proxy_socket).await?;
        println!("TLS connection established through proxy");

        send_data_with_key_updates(&mut tls_stream).await?;
    } else {
        eprintln!("Failed to establish a connection through the proxy");
    }

    Ok(())
}

async fn establish_tls_connection(server_name: &str, stream: TcpStream) -> io::Result<TlsStream<TcpStream>> {
    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
        OwnedTrustAnchor::from_subject_spki_name_constraints(ta.subject, ta.spki, ta.name_constraints)
    }));

    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(config));
    let domain = DNSNameRef::try_from_ascii_str(server_name).unwrap();

    let tls_stream = connector.connect(domain, stream).await?;
    Ok(tls_stream)
}

async fn send_data_with_key_updates(stream: &mut TlsStream<TcpStream>) -> io::Result<()> {
    let data_part1 = b"Initial Data";
    stream.write_all(data_part1).await?;
    println!("Sent data with Ksending1");

    send_key_update(stream, KeyUpdateRequest::UpdateNotRequested).await?;
    println!("Key updated to Ksending2");

    let data_part2 = b"Sensitive Data";
    stream.write_all(data_part2).await?;
    println!("Sent data with Ksending2");

    send_key_update(stream, KeyUpdateRequest::UpdateNotRequested).await?;
    println!("Key updated to Ksending3");

    let data_part3 = b"Final Data";
    stream.write_all(data_part3).await?;
    println!("Sent data with Ksending3");

    Ok(())
}

async fn send_key_update(stream: &mut TlsStream<TcpStream>, request: KeyUpdateRequest) -> io::Result<()> {
    let connection = stream.get_mut().1; 
    connection.send_key_update(request);
    stream.flush().await?;
    Ok(())
}
