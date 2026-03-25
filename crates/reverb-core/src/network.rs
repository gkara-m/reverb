use std::vec;

use crate::failure::failure::{Failure, FailureType};
use anyhow::anyhow;


// Major release when there is a breaking change to the packet structure or protocol.
//  e.g. changing header fields, removing possible functions from payload ect.
// Minor release when there is a non-breaking change to the packet structure or protocol that is backwards compatible.
//  e.g. adding new possible functions to payload ect.
// Patch release when there is a change to the packet structure or protocol which is backwards compatible and does not add any new features.
//  e.g. fixing a bug, changing error messages, changing a functions internals, ect.
pub static VERSION: [u8; 3] = [0, 0, 0];

pub enum PacketType {
    Action,
    Query,
}

impl PacketType {
    pub fn from_u8(value: u8) -> Result<Self, Failure> {
        match value {
            0 => Ok(PacketType::Action),
            1 => Ok(PacketType::Query),
            _ => Err(Failure::from((anyhow!("Invalid packet type value: {}", value), FailureType::Warning))),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            PacketType::Action => 0,
            PacketType::Query => 1,
        }
    }
}

pub enum Commands {
    DefaultCommand(DefaultCommand),
    Skip(Skip),
}

impl Commands {
    pub fn parse(data: Vec<u8>) -> Result<Self, Failure> {
        match data.get(0) {
            Some(num) => match num {
                &DefaultCommand::NUMBER => Ok(Commands::DefaultCommand(DefaultCommand::parse(data[1..].to_vec())?)),
                _ => Err(Failure::from((anyhow!("Unknown command type: {}", num), FailureType::Warning))),
            }
            None => Err(Failure::from((anyhow!("No command type specified in data"), FailureType::Warning))),
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, Failure> {
        match self {
            Commands::DefaultCommand(cmd) => {
                let mut data = vec![cmd.number()];
                data.append(&mut cmd.serialize()?);
                Ok(data)
            },
            Commands::Skip(cmd) => {
                let mut data = vec![cmd.number()];
                data.append(&mut cmd.serialize()?);
                Ok(data)
            }
        }
    }
}

pub trait Command {
    const NUMBER: u8;
    fn number(&self) -> u8 { Self::NUMBER }
    fn serialize(&self) -> Result<Vec<u8>, Failure>;
    fn parse(data: Vec<u8>) -> Result<Self, Failure> where Self: Sized;
}

pub struct DefaultCommand {}
pub struct Skip {}

impl Command for DefaultCommand {
    const NUMBER: u8 = 0;

    fn serialize(&self) -> Result<Vec<u8>, Failure> {
        Ok(vec![])
    }

    fn parse(_data: Vec<u8>) -> Result<Self, Failure> {
        Ok(DefaultCommand{})
    }
}

impl Command for Skip {
    const NUMBER: u8 = 1;

    fn serialize(&self) -> Result<Vec<u8>, Failure> {
        Ok(vec![])
    }

    fn parse(data: Vec<u8>) -> Result<Self, Failure> {
        Ok(Skip{})
    }
}

pub struct Packet {
    version: [u8; 3],
    username: String,
    group: String,
    packet_type: PacketType,
    payload: Commands,
}

impl Packet {
    pub fn new(
        username: &str,
        group: &str,
        packet_type: PacketType,
        payload: Commands,
    ) -> Result<Self, Failure> {
        check_parameters(username, group)?;

        Ok(Packet {
            version: VERSION,
            username: username.to_string(),
            group: group.to_string(),
            packet_type,
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
        let packet_type = PacketType::from_u8(_data[51])?;
        let payload = Commands::parse(_data[52..].to_vec())?;

        Ok(Packet {
            version,
            username,
            group,
            packet_type,
            payload,
        })
    }

    pub fn serialize(&self) -> Result<Vec<u8>, Failure> {
        check_parameters(&self.username, &self.group)?;
        let mut data = VERSION.to_vec();
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
        data.push(self.packet_type.to_u8());
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
    pub fn packet_type(&self) -> &PacketType {
        &self.packet_type
    }
    pub fn payload(&self) -> &Commands {
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
