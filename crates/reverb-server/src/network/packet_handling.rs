use anyhow::anyhow;

use reverb_core::{failure::failure::{Failure, FailureType}, network::*, network_command::{ID::NetworkCommandID, default_command::DefaultCommand, get_online_users::GetOnlineUsers, set_echo_availability::SetEchoAvailability}};
use crate::{SERVER_NAME, SERVER_GROUP, command_handling, network::connection::{UserAvailability, User, add_user}};


pub fn handle_packet(packet: Packet, user_id: &u16) -> Result<Option<Packet>, Failure> {
    match packet.payload.number() {
        DefaultCommand::ID => {Ok(Some(Packet::new(SERVER_NAME, SERVER_GROUP, Box::new(DefaultCommand{}))?))},
        GetOnlineUsers::ID => {
            let outgoing_command = command_handling::handle_get_online_users(packet);
            Ok(Some(Packet {
                version: NETWORK_VERSION,
                username: SERVER_NAME.to_string(),
                group: SERVER_GROUP.to_string(), 
                payload: outgoing_command
            }))
        },
        SetEchoAvailability::ID => {
            println!("matched SetEchoAvailability"); // debug
            command_handling::handle_set_echo_availability(packet, user_id)?;
            Ok(None)
        },
        _ => {Err(Failure::from((anyhow!("packet handling error: command not found"), FailureType::Warning)))}
    }
}

pub fn handle_user_info(packet: Packet) -> u16 {
    let username = packet.username;
    let group = packet.group;
    let availability = UserAvailability::ClosedToEcho;
    let user = User {
        username,
        group,
        availability
    };

    add_user(user)
}
