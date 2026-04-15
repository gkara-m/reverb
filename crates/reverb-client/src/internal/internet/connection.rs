use std::sync::mpsc;
use anyhow::{Result, anyhow};

use crate::{CONFIG, DATA_FOLDER, config::internet::{self, ServerConfig}, internal::internet::communicator};
use reverb_core::{failure::failure::{Failure, FailureType}, network::*};



#[derive(Debug)]
pub enum ConnectionStatus {
    Connected(mpsc::Sender<Packet>),
    Connecting,
    NotConnected,
}

pub struct InternetClient {
    connection_status: ConnectionStatus,
    group: Option<String>
}

impl InternetClient {
    pub fn new() -> Self {
        let _ = rustls::crypto::ring::default_provider().install_default();
        InternetClient { 
            connection_status: ConnectionStatus::NotConnected,
            group: None
        }
    }

    pub fn connect(&mut self) -> Result<(), Failure> {
        match self.connection_status {
            ConnectionStatus::Connected(_) => {
                Err(Failure::from((anyhow!("Already connected to server"), FailureType::Warning)))
            },
            ConnectionStatus::Connecting => {
                Err(Failure::from((anyhow!("Already connecting to server"), FailureType::Warning)))
            },
            ConnectionStatus::NotConnected => {
                self.connection_status = ConnectionStatus::Connecting;

                let data_folder = DATA_FOLDER.get().ok_or(Failure::from((anyhow!("Data folder not found"), FailureType::Fatal)))?.clone();
                let server_config = toml::from_str::<ServerConfig>(&std::fs::read_to_string(format!("{}{}", data_folder, internet::SERVER_CONFIG_PATH))
                    .map_err(|e| Failure::from((e.into(), "Failed to read server config, to add a server please run the server setup command", FailureType::Warning)))?)
                    .map_err(|e| Failure::from((e.into(), FailureType::Warning)))?;
                println!("Attempting to connect to server at {} with name {} and certificate path {}", server_config.server_address, server_config.server_name, server_config.server_cert_path);
                communicator::start_communicator_thread(server_config);
                Ok(())
            },
        }
    }

    pub fn send_message(&mut self, command: Box<dyn NetworkCommand + Send + Sync>) -> Result<(), Failure> {
        println!("Attempting to send message to server: ");
        let packet = Packet::new(
            CONFIG.get().ok_or(Failure::from((anyhow!("Config not created"), FailureType::Fatal)))?.username.clone().as_str(),
            self.group.clone().unwrap_or_else(|| "none".to_string()).as_str(),
            command
        )?;
        match &mut self.connection_status {
            ConnectionStatus::Connected(sender) => {sender.clone().send(packet).map_err(|e| Failure::from((e.into(), FailureType::Warning)))},
            ConnectionStatus::Connecting => Err(Failure::from((anyhow!("Currently connecting to server, cannot send message"), FailureType::Warning))),
            ConnectionStatus::NotConnected => Err(Failure::from((anyhow!("Not connected to server, cannot send message"), FailureType::Warning))),
        }
    }

    pub fn update_connection(&mut self, connection_status: ConnectionStatus) {
        self.connection_status = connection_status;
    }
}

