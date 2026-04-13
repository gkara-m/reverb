use std::{fs, io, sync::Arc};
use anyhow::anyhow;
use quinn_proto::crypto::rustls::QuicServerConfig;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};


use reverb_core::{network::*, failure::failure::{Failure, FailureType}};

mod network;
mod server_startup;


// The address and port the server will listen on
const LISTEN_ADDR: &str = "127.0.0.1:4433";
// The server version, included in responses for client verification
const VERSION: &str = "0.1.0";

/// Entry point for the server. Installs the default crypto provider, starts the async runtime,
/// and runs the main server logic. Exits with error code 1 if the server fails.
fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    println!("Server starting on {}", LISTEN_ADDR);
    // run the server (async) and handle any errors
    if let Err(e) = tokio::runtime::Runtime::new().unwrap().block_on(run()) {
        eprintln!("Server error: {e}");
        std::process::exit(1);
    }
}


async fn run() -> Result<(), Failure> {

    let endpoint = server_startup::startup()?;

    // --- Accept a single client connection ---
    if let Some(conn) = endpoint.accept().await {
        // Wait for the connection handshake to complete
        let conn = conn.await
            .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
        println!("Client connected");

        // Accept a bidirectional stream from the client
        let (mut send, mut recv) = conn.accept_bi().await
            .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;

        // Read up to 1024 bytes from the client
        let data = recv.read_to_end(1024).await
            .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;

        let packet = Packet::parse(&data)?; // TODO
        println!("Received from: {}", packet.username());

        // Prepare and send a response back to the client
        let response = format!("Server received {} bytes", data.len());
        send.write_all(response.as_bytes()).await;
        send.finish();

        // Wait for all packets to be sent before shutting down
        endpoint.wait_idle().await;
        println!("Response sent, server exiting");
    }

    Ok(())
}

