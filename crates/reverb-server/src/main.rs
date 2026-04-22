use std::sync::{LazyLock, atomic::AtomicU16};
use quinn::Endpoint;
use arc_swap::ArcSwap;
use im::HashMap as ImHashMap;

use reverb_core::failure::failure::Failure;
use crate::network::connection::{self, User};

mod network;
mod server_startup;
mod command_handling;


// The address and port the server will listen on
const LISTEN_ADDR: &str = "127.0.0.1:4433";
// The server version, included in responses for client verification
const _VERSION: &str = "0.1.0";
const SERVER_NAME: &str = "server";
const SERVER_GROUP: &str = "server";

static USERS: LazyLock<ArcSwap<ImHashMap<u16, User>>> = LazyLock::new(|| {ArcSwap::from_pointee(ImHashMap::new())});
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

