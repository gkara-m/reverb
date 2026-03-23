use std::{fs, net::SocketAddr, sync::mpsc};

use anyhow::{Result, anyhow};
use quinn::Connection;
use quinn_proto::crypto::rustls::QuicClientConfig;
use rustls::pki_types::CertificateDer;
use std::net::AddrParseError;

use std::sync::Arc;

use crate::{CONFIG, Command, DATA_FOLDER, MAIN_SENDER, config::internet::{self, ServerConfig}, failure::failure::{Failure, FailureType}};

static VERSION: &str = "0.1.0";


#[derive(Debug)]
pub enum ConnectionStatus {
    Connected(Connection),
    Connecting,
    NotConnected,
}

pub(super) struct InternetClient {
    connection: ConnectionStatus,
}

impl InternetClient {
    pub fn new() -> Self {
        InternetClient { 
            connection: ConnectionStatus::NotConnected,
        }
    }

    pub fn connect(&mut self) -> Result<(), Failure> {
        match self.connection {
            ConnectionStatus::Connected(_) => {
                return Err(Failure::from((anyhow!("Already connected to server"), FailureType::Warning)));
            },
            ConnectionStatus::Connecting => {
                return Err(Failure::from((anyhow!("Already connecting to server"), FailureType::Warning)));
            },
            ConnectionStatus::NotConnected => {
                 self.connection = ConnectionStatus::Connecting;

                let data_folder = DATA_FOLDER.get().ok_or(Failure::from((anyhow!("Data folder not found"), FailureType::Fetal)))?.clone();
                let server_config = toml::from_str::<ServerConfig>(&std::fs::read_to_string(format!("{}{}", data_folder, internet::SERVER_CONFIG_PATH))
                    .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?)
                    .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
                let conn = connect_to(server_config); // TODO make this async
                Ok(())
            },
        }
    }

    pub fn send_message(&mut self, message: String) -> Result<(), Failure> {
        match &mut self.connection {
            ConnectionStatus::Connected(conn) => send_message(conn.clone(), message),
            ConnectionStatus::Connecting => Err(Failure::from((anyhow!("Currently connecting to server, cannot send message"), FailureType::Warning))),
            ConnectionStatus::NotConnected => Err(Failure::from((anyhow!("Not connected to server, cannot send message"), FailureType::Warning))),
        }
    }

    pub fn update_connection(&mut self, connection_status: ConnectionStatus) {
        self.connection = connection_status;
    }
}


async fn connect_to(server_config: ServerConfig) {
    // Locate the directory where the server's certificate is stored (shared location)
    let data_folder = match CONFIG.get() {
        Some(cfg) => cfg.data_folder.clone(),
        None => {
            let _ = MAIN_SENDER.get().unwrap().clone().send(Command::Failure(Failure::from((anyhow!("Config folder not found"), FailureType::Fetal))));
            return;
        }
    };
    let path = std::path::Path::new(&data_folder); // TODO add command to add it to the config
    let cert_path = path.join(&server_config.server_cert_path);

    println!("Loading server certificate from: {:?}", cert_path);

    // Create an empty root certificate store
    let mut roots = rustls::RootCertStore::empty();
    // If the server's certificate exists, add it to the root store for TLS validation
    if let Ok(cert) = fs::read(&cert_path) {
        if let Err(e) = roots.add(CertificateDer::from(cert)) {
            let _ = MAIN_SENDER.get().unwrap().clone().send(Command::Failure(Failure::from((anyhow!(e), FailureType::Warning))));
            return;
        }
    }

    if roots.is_empty() {
        MAIN_SENDER.get().unwrap().clone().send(Command::Failure(Failure::from((anyhow!("No valid server certificate found at path: {:?}", cert_path), FailureType::Warning)))).unwrap_or_else(|e| eprintln!("Failed to send failure command: {}", e));
        return;
    }

    // Build the client cryptographic configuration with the root certificates and no client authentication
    let client_crypto = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();


    // Wrap the client crypto config in a Quinn QUIC client config
    let client_config = match QuicClientConfig::try_from(client_crypto) {
        Ok(c) => quinn::ClientConfig::new(Arc::new(c)),
        Err(e) => {
            let _ = MAIN_SENDER.get().unwrap().clone().send(Command::Failure(Failure::from((e.into(), FailureType::Warning))));
            return;
        }
    };
    let client_config =
        client_config;
    // Create a new QUIC endpoint for the client, binding to an ephemeral port on all interfaces
    let mut endpoint = match "[::]:0".parse::<SocketAddr>() {
        Ok(addr) => match quinn::Endpoint::client(addr) {
            Ok(ep) => ep,
            Err(e) => {
                let _ = MAIN_SENDER.get().unwrap().clone().send(Command::Failure(Failure::from((e.into(), FailureType::Warning))));
                return;
            }
        },
        Err(e) => {
            let _ = MAIN_SENDER.get().unwrap().clone().send(Command::Failure(Failure::from((e.into(), FailureType::Warning))));
            return;
        }
    };
    // Set the default client configuration for outgoing connections
    endpoint.set_default_client_config(client_config);

    // get the server name for TLS validation from the server config
    let host = server_config.server_name.as_str();

    // Parse the server address string into a SocketAddr
    let remote = match server_config.server_address.parse::<SocketAddr>() {
        Ok(addr) => addr,
        Err(e) => {
            let _ = MAIN_SENDER.get().unwrap().clone().send(Command::Failure(Failure::from((e.into(), FailureType::Warning))));
            return;
        }
    };

    // Initiate a QUIC connection to the server with the given address and host
    let conn = match endpoint.connect(remote, host) {
        Ok(connecting) => match connecting.await {
            Ok(conn) => conn,
            Err(e) => {
                let _ = MAIN_SENDER.get().unwrap().clone().send(Command::Failure(Failure::from((e.into(), FailureType::Warning))));
                return;
            }
        },
        Err(e) => {
            let _ = MAIN_SENDER.get().unwrap().clone().send(Command::Failure(Failure::from((e.into(), FailureType::Warning))));
            return;
        }
    };
    println!("Successfully connected to server at {}", server_config.server_address);

    MAIN_SENDER.get().unwrap().clone().send(Command::ServerUpdateStatus(ConnectionStatus::Connected(conn))).unwrap_or_else(|e| eprintln!("Failed to send server update status command: {}", e));
}

#[tokio::main]
async fn send_message(conn: Connection, message: String) -> Result<(), Failure> {


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