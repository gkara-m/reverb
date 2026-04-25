use std::{collections::HashMap, sync::Arc};
use crate::connection::UserAvailability;

use anyhow::anyhow;

use reverb_core::{failure::failure::{Failure, FailureType}, network::*, network_command::{helpers::NetworkCommand, online_users::OnlineUsers, set_echo_availability::SetEchoAvailability}};
use crate::USERS;

pub fn handle_get_online_users(_packet: Packet) -> Box<dyn NetworkCommand + Send + Sync> {
    // let command = try_get_online_users(packet.payload)?;
    let users_guard = USERS.load();
    let open_users: HashMap<u16, String> = users_guard.iter()
        .filter(|(_, user)| matches!(user.availability, UserAvailability::OpenToEcho))
        .map(|(&id, user)| (id, user.username.clone())).collect();
    
    Box::new(OnlineUsers { users: open_users })
}

fn try_get_set_echo_availability(item: &Box<dyn NetworkCommand + Send + Sync>) -> Result<SetEchoAvailability, Failure> {
    if let Some(command) = item.as_any().downcast_ref::<SetEchoAvailability>() {
        Ok(command.clone())
    } else {
        Err(Failure::from((anyhow!("failed to read SetEchoAvailability from Box"), FailureType::Warning)))
    }
}

pub fn handle_set_echo_availability(packet: Packet, user_id: &u16) -> Result<(), Failure> {
    println!("trying get set echo availability"); //debug
    let command = try_get_set_echo_availability(packet.payload())?;
    let mut user = USERS.load()[user_id].clone();
    let echo_availability = match command.0 {
        false => UserAvailability::ClosedToEcho,
        true => UserAvailability::OpenToEcho
    };
    
    user.availability = echo_availability;

    USERS.rcu(|user_hashmap| Arc::new(user_hashmap.update(*user_id, user.clone())));

    Ok(())
}
