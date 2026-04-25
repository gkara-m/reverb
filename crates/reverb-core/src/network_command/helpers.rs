use std::any::Any;

use crate::network_command::ID::NetworkCommandID;
use crate::failure::failure::{Failure, FailureType};
use crate::network_command::{default_command::DefaultCommand, echo::Echo, get_online_users::GetOnlineUsers, online_users::OnlineUsers, set_echo_availability::SetEchoAvailability, skip::Skip, user_data::UserData};
use anyhow::anyhow;

pub enum QueryOrNotify {
    Query,
    Notify
}

// parse data to the apporpriate command from the netowrk
pub fn parse_command(data: Vec<u8>) -> Result<Box<dyn NetworkCommand + Send + Sync>, Failure> {
    println!("command size: {} bytes", data.len()); // Debug line
    let cmd_number = data[0];

    match cmd_number {
        DefaultCommand::ID => Ok(Box::new(DefaultCommand::parse(data)?)),
        Skip::ID => Ok(Box::new(Skip::parse(data)?)),
        Echo::ID => Ok(Box::new(Echo::parse(data)?)),
        OnlineUsers::ID => Ok(Box::new(OnlineUsers::parse(data)?)),
        GetOnlineUsers::ID => Ok(Box::new(GetOnlineUsers::parse(data)?)),
        UserData::ID => Ok(Box::new(UserData::parse(data)?)),
        SetEchoAvailability::ID => Ok(Box::new(SetEchoAvailability::parse(data)?)),
        _ => Err(Failure::from((anyhow!("invalid command"), FailureType::Warning)))
    }
}

// serialize a command to be sent over the network
pub fn serialize(boxed_cmd: &Box<dyn NetworkCommand + Send + Sync>) -> Result<Vec<u8>, Failure> {
    let mut data = vec![boxed_cmd.number()];
    data.append(&mut boxed_cmd.serialize()?);
    Ok(data)
}

pub trait NetworkCommand: Any {
    fn number(&self) -> u8; // numbers should be changed when any functionality changes as we are NOT maintaining backwards compatability
    fn serialize(&self) -> Result<Vec<u8>, Failure>;
    fn parse(data: Vec<u8>) -> Result<Self, Failure> where Self: Sized;
    fn query_or_notify(&self) -> QueryOrNotify;
    fn as_any(&self) -> &dyn Any;
}

