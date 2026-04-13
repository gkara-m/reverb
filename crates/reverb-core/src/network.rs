use std::vec;

use crate::failure::failure::{Failure, FailureType};
use anyhow::anyhow;


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

pub fn parse(data: Vec<u8>) -> Result<Box<dyn NetworkCommand + Send + Sync>, Failure> {
    println!("packet size: {} bytes", data.len()); // Debug line
    let number = data.get(51).ok_or(Failure::from((anyhow!("invalid packet: packet too small"), FailureType::Warning)))?
        .to_owned();
    if (number == DefaultCommand{}.number()) {
        return Ok(Box::new(DefaultCommand::parse(data)?));
    }
    Err(Failure::from((anyhow!["invalid command recived"], FailureType::Warning)))
}

pub fn serialize(boxed_cmd: Box<dyn NetworkCommand + Send + Sync>) -> Result<Vec<u8>, Failure> {
    let mut data = vec![boxed_cmd.number()];
    data.append(&mut boxed_cmd.serialize()?);
    println!("serialized into: {} bytes", data.len()); // Debug line
    Ok(data)
}

pub trait NetworkCommand {
    fn number(&self) -> u8; // numbers should be changed when any functionality changes as we are NOT maintaining backwards compatability
    fn serialize(&self) -> Result<Vec<u8>, Failure>;
    fn parse(data: Vec<u8>) -> Result<Self, Failure> where Self: Sized;
    fn query_or_notify(&self) -> QueryOrNotify;
}

pub struct DefaultCommand {}
pub struct Skip {}
pub struct Echo {
    echo_type: EchoType,
    echo_target: String
}
pub struct GetOnlineUsers {}

pub enum EchoType {
    Group,
    User
}

impl NetworkCommand for DefaultCommand {
    fn number(&self) -> u8 {
        0
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
}

impl NetworkCommand for Skip {
    fn number(&self) -> u8 {
        1
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
}

impl NetworkCommand for Echo {
    fn number(&self) -> u8 {
        2
    }
    fn serialize(&self) -> Result<Vec<u8>, Failure> {
        Err(Failure::from((anyhow!("Echo command serialization not implemented yet"), FailureType::Warning)))
    }

    fn parse(data: Vec<u8>) -> Result<Self, Failure> where Self: Sized {
        Err(Failure::from((anyhow!("Echo command parsing not implemented yet"), FailureType::Warning)))
    }

    fn query_or_notify(&self) -> QueryOrNotify {
        QueryOrNotify::Query
    }
}

impl NetworkCommand for GetOnlineUsers {
    fn number(&self) -> u8 {
        3
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

    pub fn parse(_data: &[u8]) -> Result<Self, Failure> {
        if _data.len() < 52 {
            return Err(Failure::from((anyhow!("Data too short to be a valid packet"), FailureType::Warning)));
        }
        let version = [_data[0], _data[1], _data[2]];
        let username = String::from_utf8_lossy(&_data[3..35]).trim_matches(char::from(0)).to_string();
        let group = String::from_utf8_lossy(&_data[35..51]).trim_matches(char::from(0)).to_string();
        let payload = parse(_data[52..].to_vec())?;

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
        data.append(&mut self.payload.serialize()?);
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
