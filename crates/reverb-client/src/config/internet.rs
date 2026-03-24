use std::path::Path;

use crate::{CONFIG, };
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use reverb_core::failure::failure::{Failure, FailureType};

pub static SERVER_CONFIG_PATH: &str = "server_config.toml";


#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    pub server_address: String,
    pub server_name: String,
    pub server_cert_path: String,
}

impl ServerConfig {
    pub fn new(server_address: &str, server_name: &str, server_cert_path: &str) -> Result<ServerConfig, Failure> {
        let server_config = ServerConfig {
            server_address: server_address.to_string(),
            server_name: server_name.to_string(),
            server_cert_path: server_cert_path.to_string(),
        };
        server_config.save()?;
        Ok(server_config)
    }

    fn save(&self) -> Result<(), Failure> {
        match std::fs::create_dir_all(CONFIG.get().ok_or(Failure::from((anyhow!("Config folder not found"), FailureType::Fatal)))?.data_folder.clone()) {
            Err(e) => return Err(Failure::from((e.into(), FailureType::Warning))),
            Ok(_) => {},
        }
        match std::fs::write(
            format!("{}server_config.toml", CONFIG.get().ok_or(Failure::from((anyhow!("Config folder not found"), FailureType::Fatal)))?.data_folder.clone()),
            toml::to_string(self).map_err(|e| Failure::from((e.into(), FailureType::Warning)))?,
        ) {
            Err(e) => Err(Failure::from((e.into(), FailureType::Warning))),
            Ok(_) => Ok(()),
        }
    }
}