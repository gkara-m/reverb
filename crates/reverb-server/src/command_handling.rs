use std::collections::HashMap;
use crate::connection::UserAvailability;

use anyhow::anyhow;

use reverb_core::{network::*, failure::failure::{Failure, FailureType}};
use crate::{USERS, network::connection};

pub fn handle_get_online_users(_packet: Packet) -> Box<dyn NetworkCommand + Send + Sync> {
    // let command = try_get_online_users(packet.payload)?;
    let users_guard = USERS.load();
    let open_users: HashMap<u16, String> = users_guard.iter()
        .filter(|(_, user)| matches!(user.availability, UserAvailability::OpenToEcho))
        .map(|(&id, user)| (id, user.username.clone())).collect();
    
    Box::new(GetOnlineUsers { users: open_users })
}

// fn try_get_online_users(item: Box<dyn NetworkCommand>) -> Result<GetOnlineUsers, Failure> {
//     if let Some(command) = item.as_any().downcast_ref::<GetOnlineUsers>() {
//         Ok(command.clone())
//     } else {
//         Err(Failure::from((anyhow!("failed to read GetOnlineUsers from Box"), FailureType::Warning)))
//     }
// }
