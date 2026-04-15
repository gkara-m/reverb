use std::{fs, io, sync::Arc};
use quinn::{Connection, Endpoint, Incoming, RecvStream};
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

#[tokio::main]
async fn main() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    println!("Server starting on {}", LISTEN_ADDR);

    // run server startup
    let endpoint = match server_startup::startup() {
        Ok(endpoint) => endpoint,
        Err(failure) => {
            eprintln!("Server startup error: {failure}");
            std::process::exit(1);
        }
    };
    
    loop {
        if let Err(e) = run(&endpoint).await {
            eprintln!("Server runtime error: {e}");
            std::process::exit(2);
        }
    }

}

// accepts incoming connections and hands them off to new tokio async task
async fn run(endpoint: &Endpoint) -> Result<(), Failure> {
    if let Some(conn) = endpoint.accept().await {
        tokio::spawn(async move {
            if let Err(e) = handle_connection(conn).await {
                eprintln!("Server runtime error: error handling connection: {e}")
            };
        });
    }

    Ok(())
}

async fn handle_connection(conn: Incoming) -> Result<(), Failure> {
    // Wait for the connection handshake to complete
    let conn_bi = conn.await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
    println!("Client connected");
    let conn_uni = conn_bi.clone();

    tokio::spawn(async move {
        loop {
            if let Err(e) = handle_bi(&conn_bi).await {eprintln!("Server connection error: {e}")}
        }
    });

    tokio::spawn(async move {
        loop {
            if let Err(e) = handle_uni(&conn_uni).await {eprintln!("Server connection error: {e}")}
        }
    });

    Ok(())
}

async fn handle_bi(conn: &Connection) -> Result<(), Failure> {
    let (mut send, mut recv) = conn.accept_bi().await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;

    let data = read_incoming(recv).await?;
    let packet = Packet::parse(&data)?;

    // Prepare and send a response back to the client
    let response = create_response(packet)?;
    send.write_all(&response.serialize()?).await;
    send.finish();
    
    Ok(())
}

async fn handle_uni(conn: &Connection) -> Result<(), Failure> {
    let mut recv = conn.accept_uni().await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;

    let data = read_incoming(recv).await?;
    let packet = Packet::parse(&data)?;

    Ok(())
}

async fn read_incoming(mut recv: RecvStream) -> Result<Vec<u8>, Failure> {
    recv.read_to_end(1024).await
        .map_err(|e| Failure::from((e.into(), FailureType::Warning)))
}

fn create_response(packet: Packet) -> Result<Packet, Failure> {
    Packet::new("server", "server", Box::new(DefaultCommand{}))
}

