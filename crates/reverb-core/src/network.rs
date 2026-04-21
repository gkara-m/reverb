use std::{any::Any, collections::HashMap};

use crate::failure::failure::{Failure, FailureType};
use anyhow::anyhow;
use postcard::{from_bytes, to_slice};


// Major release when there is a breaking change to the packet structure or protocol.
//  e.g. changing header fields, removing possible functions from payload ect.
// Minor release when there is a non-breaking change to the packet structure or protocol that is backwards compatible.
//  e.g. adding new possible functions to payload ect.
// Patch release when there is a change to the packet structure or protocol which is backwards compatible and does not add any new features.
//  e.g. fixing a bug, changing error messages, changing a functions internals, ect.
pub static NETWORK_VERSION: [u8; 3] = [0, 0, 0];

pub enum QueryOrNotify {
    Query,
    Notify
}

pub fn parse_command(data: Vec<u8>) -> Result<Box<dyn NetworkCommand + Send + Sync>, Failure> {
    println!("command size: {} bytes", data.len()); // Debug line
    let cmd_number = data[0];

    match cmd_number {
        DefaultCommand::ID => Ok(Box::new(DefaultCommand{})),
        Skip::ID => Ok(Box::new(Skip{})),
        Echo::ID => Ok(Box::new(Echo::parse(data)?)),
        OnlineUsers::ID => Ok(Box::new(OnlineUsers::parse(data)?)),
        _ => Err(Failure::from((anyhow!("invalid command"), FailureType::Warning)))
    }
}

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

pub struct DefaultCommand {}
pub struct Skip {}
pub struct Echo {
    pub echo_type: EchoType,
    pub echo_target: String
}
#[derive(Clone, Debug)]
pub struct OnlineUsers {
    pub users: HashMap<u16, String>
}

pub struct GetOnlineUsers {}
pub struct RequestUserData {}

pub enum EchoType {
    Group = 0,
    User = 1
}

impl EchoType {
    fn parse(data: u8) -> Result<Self, Failure> {
        match data {
            x if x == EchoType::Group as u8 => Ok(EchoType::Group),
            x if x == EchoType::User as u8 => Ok(EchoType::User),
            _ => Err(Failure::from((anyhow!("invalid echotype"), FailureType::Warning)))
        }
    }
}

pub trait NetworkCommandID {
    const ID: u8;
}

impl NetworkCommandID for DefaultCommand {
    const ID: u8 = 0;
}
impl NetworkCommandID for Skip {
    const ID: u8 = 1;
}
impl NetworkCommandID for Echo {
    const ID: u8 = 2;
}
impl NetworkCommandID for OnlineUsers {
    const ID: u8 = 3;
}
impl NetworkCommandID for RequestUserData {
    const ID: u8 = 4;
}
impl NetworkCommandID for GetOnlineUsers {
    const ID: u8 = 5;
}


impl NetworkCommand for DefaultCommand {
    fn number(&self) -> u8 {
        DefaultCommand::ID
    }

    fn serialize(&self) -> Result<Vec<u8>, Failure> {
        Ok(vec![])
    }
    fn parse(_data: Vec<u8>) -> Result<Self, Failure> {
        Ok(DefaultCommand{})
    }

    fn query_or_notify(&self) -> QueryOrNotify {
        QueryOrNotify::Query
    }

    fn as_any(&self) -> &dyn Any { self }
}

impl NetworkCommand for Skip {
    fn number(&self) -> u8 {
        Skip::ID
    }
    fn serialize(&self) -> Result<Vec<u8>, Failure> {
        Ok(vec![])
    }
    fn parse(_data: Vec<u8>) -> Result<Self, Failure> {
        Ok(Skip{})
    }

    fn query_or_notify(&self) -> QueryOrNotify {
        QueryOrNotify::Notify
    }

    fn as_any(&self) -> &dyn Any { self }
}

impl NetworkCommand for Echo {
    fn number(&self) -> u8 {
        Echo::ID
    }
    fn serialize(&self) -> Result<Vec<u8>, Failure> {
        let mut data = match self.echo_type {
            EchoType::Group => vec![EchoType::Group as u8],
            EchoType::User => vec![EchoType::User as u8]
        };
        data.extend_from_slice(self.echo_target.as_bytes());
        Ok(data)
    }

    fn parse(data: Vec<u8>) -> Result<Self, Failure> where Self: Sized {
        let echo_type = EchoType::parse(data[1])?;
        let target_data = data[2..].to_vec();
        let echo_target = String::from_utf8(target_data)
            .map_err(|e| Failure::from((anyhow!("failed to parse echo target: {e}"), FailureType::Warning)))?;

        Ok(Echo{
            echo_type,
            echo_target
        })
    }

    fn query_or_notify(&self) -> QueryOrNotify {
        QueryOrNotify::Query
    }

    fn as_any(&self) -> &dyn Any { self }

}

impl NetworkCommand for OnlineUsers {
    fn number(&self) -> u8 {
        OnlineUsers::ID
    }
    fn serialize(&self) -> Result<Vec<u8>, Failure> {
        let mut data = Vec::new();
        data.append(&mut [OnlineUsers::ID].to_vec());
        let mut buffer = [0u8; 512];
        let user_data = to_slice(&self.users, &mut buffer)
            .map_err(|e| Failure::from((anyhow!("failed to serialize GetOnlineUsers: {e}"), FailureType::Warning)))?;
        data.append(&mut user_data.to_vec());

        Ok(data)
    }
    fn parse(data: Vec<u8>) -> Result<Self, Failure> where Self: Sized {
        let users: HashMap<u16, String> = from_bytes(&data[1..])
            .map_err(|e| Failure::from((anyhow!("failed to parse GetOnlineUsers: {e}"), FailureType::Warning)))?;

        Ok(OnlineUsers { users })
    }

    fn query_or_notify(&self) -> QueryOrNotify {
        QueryOrNotify::Query
    }

    fn as_any(&self) -> &dyn Any { self }

}

impl NetworkCommand for GetOnlineUsers {
    fn number(&self) -> u8 {
        GetOnlineUsers::ID
    }
    fn serialize(&self) -> Result<Vec<u8>, Failure> {
        Ok(vec![])
    }
    fn parse(_data: Vec<u8>) -> Result<Self, Failure> where Self: Sized {
        Ok(GetOnlineUsers {})
    }
    fn query_or_notify(&self) -> QueryOrNotify {
        QueryOrNotify::Query
    }
    fn as_any(&self) -> &dyn Any { self }
}

impl NetworkCommand for RequestUserData {
    fn number(&self) -> u8 {
        RequestUserData::ID
    }
    fn serialize(&self) -> Result<Vec<u8>, Failure> {
        Ok(vec![])
    }
    fn parse(_data: Vec<u8>) -> Result<Self, Failure> where Self: Sized {
        Ok(RequestUserData {})
    }
    fn query_or_notify(&self) -> QueryOrNotify {
        QueryOrNotify::Query
    }
    fn as_any(&self) -> &dyn Any { self }
}


pub struct Packet {
    pub version: [u8; 3],
    pub username: String,
    pub group: String,
    pub payload: Box<dyn NetworkCommand + Send + Sync>,
}

impl Packet {
    pub fn new(
        username: &str,
        group: &str,
        payload: Box<dyn NetworkCommand + Send + Sync>,
    ) -> Result<Self, Failure> {
        check_parameters(username, group)?;

        Ok(Packet {
            version: NETWORK_VERSION,
            username: username.to_string(),
            group: group.to_string(),
            payload,
        })
    }

    pub fn parse(_data: &[u8]) -> Result<Packet, Failure> {
        println!("data length to parse: {} bytes", _data.len()); // Debug line
        if _data.len() < 52 {
            return Err(Failure::from((anyhow!("Data too short to be a valid packet"), FailureType::Warning)));
        }
        let version = [_data[0], _data[1], _data[2]];
        let username = String::from_utf8_lossy(&_data[3..35]).trim_matches(char::from(0)).to_string();
        let group = String::from_utf8_lossy(&_data[35..51]).trim_matches(char::from(0)).to_string();
        let payload = parse_command(_data[52..].to_vec())?;

        Ok(Packet {
            version,
            username,
            group,
            payload,
        })
    }

    pub fn serialize(&self) -> Result<Vec<u8>, Failure> {
        check_parameters(&self.username, &self.group)?;
        let mut data = NETWORK_VERSION.to_vec();
        for i in 0..32 {
            if i < self.username.len() {
                data.push(self.username.as_bytes()[i]);
            } else {
                data.push(0);
            }
        }
        for i in 0..16 {
            if i < self.group.len() {
                data.push(self.group.as_bytes()[i]);
            } else {
                data.push(0);
            }
        }
        data.append(&mut vec![self.payload.number()]);
        data.append(&mut serialize(&self.payload)?);
        Ok(data)
    }

    pub fn version(&self) -> &[u8; 3] {
        &self.version
    }
    pub fn username(&self) -> &str {
        &self.username
    }
    pub fn group(&self) -> &str {
        &self.group
    }
    pub fn payload(&self) -> &Box<dyn NetworkCommand + Send + Sync> {
        &self.payload
    }
}

fn check_parameters(username: &str, group: &str) -> Result<(), Failure> {
    if username.len() > 32 {
        return Err(Failure::from((anyhow!("username too long"), FailureType::Warning)));
    }
    if group.len() > 16 {
        return Err(Failure::from((anyhow!("group name too long"), FailureType::Warning)));
    }
    Ok(())
}
