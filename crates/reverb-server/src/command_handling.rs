use anyhow::anyhow;

use reverb_core::{network::*, failure::failure::{Failure, FailureType}};
use crate::network::connection;

pub fn handle_get_online_users(packet: Packet) -> Result<Box<dyn NetworkCommand + Send + Sync>, Failure> {
    let boxed_command = packet.payload;
    let command = try_get_online_users(boxed_command)?;
    
    Err(Failure::from((anyhow!("packet handling error: command not implemented"), FailureType::Warning)))
}

fn try_get_online_users(item: Box<dyn NetworkCommand>) -> Result<GetOnlineUsers, Failure> {
    if let Some(command) = item.as_any().downcast_ref::<GetOnlineUsers>() {
        Ok(command.clone())
    } else {
        Err(Failure::from((anyhow!("failed to read GetOnlineUsers from Box"), FailureType::Warning)))
    }
}
