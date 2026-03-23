use std::{fs, net::SocketAddr};

use anyhow::{Result, anyhow};
use quinn::Connection;
use quinn_proto::crypto::rustls::QuicClientConfig;
use rustls::pki_types::CertificateDer;
use std::net::AddrParseError;

use std::sync::Arc;

use crate::failure::failure::{Failure, FailureType};


struct InternetClient {}

pub fn connect() {
    // Install the default cryptographic provider for Rustls (required for cryptographic operations)
    let _ = rustls::crypto::ring::default_provider().install_default();
    // Define the server address and the message to send
    let server = "127.0.0.1:4433".to_string(); // TODO change to collect from config
    let message = "Hello".to_string();
    println!("Client connecting to {}", server);
    // Run the async client logic and handle any errors
    if let Err(e) = run(server, message) {
        eprintln!("Client error: {e}");
        std::process::exit(1);
    }
}

async fn connect_to(server: String) -> Result<Connection, Failure> {
    // Locate the directory where the server's certificate is stored (shared location)
    let path = std::path::Path::new("certs"); // TODO collect from config, add command to add it to teh config
    let cert_path = path.join("cert.der");

    println!("Loading server certificate from: {:?}", cert_path);

    // Create an empty root certificate store
    let mut roots = rustls::RootCertStore::empty();
    // If the server's certificate exists, add it to the root store for TLS validation
    if let Ok(cert) = fs::read(&cert_path) {
        roots.add(CertificateDer::from(cert))
        .map_err(|e| Failure::from((anyhow!(e), FailureType::Warning)))?;
    }

    println!("{:?}", roots);

    // Build the client cryptographic configuration with the root certificates and no client authentication
    let client_crypto = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();


    // Wrap the client crypto config in a Quinn QUIC client config
    let client_config =
        quinn::ClientConfig::new(Arc::new(QuicClientConfig::try_from(client_crypto)
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?));
    // Create a new QUIC endpoint for the client, binding to an ephemeral port on all interfaces
    let mut endpoint = quinn::Endpoint::client("[::]:0".parse()
        .map_err(|e: AddrParseError| Failure::from((e.into(), FailureType::Warning)))?)
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
    // Set the default client configuration for outgoing connections
    endpoint.set_default_client_config(client_config);

    // Extract the host part for SNI (Server Name Indication) and certificate validation
    let host_part = server.split(':').next().unwrap_or("localhost");
    let host = if host_part == "127.0.0.1" || host_part == "::1" {// TODO ip should be collected from config aswell as name
        "localhost"
    } else {
        host_part
    };
    // Parse the server address string into a SocketAddr
    let remote = server
        .parse::<SocketAddr>()
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;

    // Initiate a QUIC connection to the server with the given address and host
    let conn = endpoint
        .connect(remote, host)
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?
        .await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;

    Ok(conn)
}

#[tokio::main]
async fn run(server: String, message: String) -> Result<(), Failure> {

    let conn = connect_to(server).await?;

    println!("Connected, opening stream");

    // Open a bidirectional stream to the server
    let (mut send, mut recv) = conn.open_bi().await
    .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;

    // Send the message to the server
    send.write_all(message.as_bytes()).await
    .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
    // Indicate that no more data will be sent on this stream
    send.finish()
    .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
    println!("Sent: {}", message);

    // Wait for the server's response and print it
    match recv.read_to_end(1024).await {
        Ok(data) => {
            println!("Received response: {}", String::from_utf8_lossy(&data));
        }
        Err(e) => {
            eprintln!("Receive error: {e}");
        }
    }

    Ok(())
}