use std::{collections::HashMap, sync::{LazyLock, Arc, atomic::{AtomicU16, Ordering}}};
use anyhow::anyhow;
use quinn::Endpoint;
use arc_swap::ArcSwap;

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

static USERS: LazyLock<ArcSwap<HashMap<u16, User>>> = LazyLock::new(|| {ArcSwap::from_pointee(HashMap::new())});
static NEXT_ID: AtomicU16 = AtomicU16::new(1);

/// Entry point for the server. Installs the default crypto provider, starts the async runtime,
/// and runs the main server logic. Exits with error code 1 if the server fails at startup or error
/// code 2 if fails at runtime.

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

fn add_user(user: User) -> u16 {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed); // panics if exceeds 65535

    let mut map = (**USERS.load()).clone();
    map.insert(id, user);
    USERS.store(Arc::new(map));

    id
}
