use std::{fs, net::SocketAddr, sync::mpsc};

use anyhow::{Result, anyhow};


use crate::{CONFIG, Command, DATA_FOLDER, MAIN_SENDER, config::internet::{self, ServerConfig}, failure::failure::{Failure, FailureType}, internal::internet::communicator};

static VERSION: &str = "0.1.0";


#[derive(Debug)]
pub enum ConnectionStatus {
    Connected(mpsc::Sender<String>),
    Connecting,
    NotConnected,
}

pub struct InternetClient {
    connection_status: ConnectionStatus,
}

impl InternetClient {
    pub fn new() -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        InternetClient { 
            connection_status: ConnectionStatus::NotConnected,
        }
    }

    pub fn connect(&mut self) -> Result<(), Failure> {
        match self.connection_status {
            ConnectionStatus::Connected(_) => {
                return Err(Failure::from((anyhow!("Already connected to server"), FailureType::Warning)));
            },
            ConnectionStatus::Connecting => {
                return Err(Failure::from((anyhow!("Already connecting to server"), FailureType::Warning)));
            },
            ConnectionStatus::NotConnected => {
                self.connection_status = ConnectionStatus::Connecting;

                let data_folder = DATA_FOLDER.get().ok_or(Failure::from((anyhow!("Data folder not found"), FailureType::Fetal)))?.clone();
                let server_config = toml::from_str::<ServerConfig>(&std::fs::read_to_string(format!("{}{}", data_folder, internet::SERVER_CONFIG_PATH))
                    .map_err(|e| Failure::from((e.into(), "Failed to read server config, to add a server please run the server setup command", FailureType::Warning)))?)
                    .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
                println!("Attempting to connect to server at {} with name {} and certificate path {}", server_config.server_address, server_config.server_name, server_config.server_cert_path);
                communicator::start_communicator_thread(server_config);
                Ok(())
            },
        }
    }

    pub fn send_message(&mut self, message: String) -> Result<(), Failure> {
        println!("Attempting to send message to server: {}", message);
        match &mut self.connection_status {
            ConnectionStatus::Connected(sender) => {sender.clone().send(message).map_err(|e| Failure::from((e.into(), FailureType::Warning)))},
            ConnectionStatus::Connecting => Err(Failure::from((anyhow!("Currently connecting to server, cannot send message"), FailureType::Warning))),
            ConnectionStatus::NotConnected => Err(Failure::from((anyhow!("Not connected to server, cannot send message"), FailureType::Warning))),
        }
    }

    pub fn update_connection(&mut self, connection_status: ConnectionStatus) {
        self.connection_status = connection_status;
    }
}

