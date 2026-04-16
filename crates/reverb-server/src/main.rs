use std::{collections::HashMap, fs, io, sync::{Arc, mpsc::{Receiver, Sender}}};
use anyhow::anyhow;
use quinn::{Connection, Endpoint, Incoming, RecvStream};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};


use reverb_core::{network::*, failure::failure::{Failure, FailureType}};
use crate::network::connection::{self, User};

mod network;
mod server_startup;
mod command_handling;


// The address and port the server will listen on
const LISTEN_ADDR: &str = "127.0.0.1:4433";
// The server version, included in responses for client verification
const VERSION: &str = "0.1.0";
const SERVER_NAME: &str = "server";
const SERVER_GROUP: &str = "server";

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
    
    // TODO use parking_lot::RwLock for this 
    let mut users: HashMap<String, User> = HashMap::new();     

    loop {
        if let Err(e) = run(&endpoint).await {
            eprintln!("Server runtime error: {e}");
            if let Failure::Fatal(_, _) = e {
                std::process::exit(2)
            }
        }
    }

}

// accepts incoming connections and hands them off to new tokio async task
async fn run(endpoint: &Endpoint) -> Result<(), Failure> {
    if let Some(conn) = endpoint.accept().await {
        tokio::spawn(async move {
            if let Err(e) = connection::handle_connection(conn).await {
                eprintln!("Server runtime error: error handling connection: {e}")
            };
        });
    }

    Ok(())
}

fn handle_packet(packet: Packet) -> Result<Option<Packet>, Failure> {
    match packet.payload.number() {
        DefaultCommand::ID => {Ok(Some(Packet::new(SERVER_NAME, SERVER_GROUP, Box::new(DefaultCommand{}))?))},
        GetOnlineUsers::ID => {
            let outgoing_command = command_handling::handle_get_online_users(packet)?;
            Err(Failure::from((anyhow!("packet handling error: command not implemented"), FailureType::Warning)))
        },
        _ => {Err(Failure::from((anyhow!("packet handling error: command not found"), FailureType::Warning)))}
    }
}

