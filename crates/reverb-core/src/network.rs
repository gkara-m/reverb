use std::{any::Any, collections::HashMap, fmt};

use crate::{network_command::{default_command::DefaultCommand, helpers::{NetworkCommand, QueryOrNotify, parse_command, serialize}}, failure::failure::{Failure, FailureType}}; 
use anyhow::anyhow;
use postcard::{from_bytes, to_slice};


// Major release when there is a breaking change to the packet structure or protocol.
//  e.g. changing header fields, removing possible functions from payload ect.
// Minor release when there is a non-breaking change to the packet structure or protocol that is backwards compatible.
//  e.g. adding new possible functions to payload ect.
// Patch release when there is a change to the packet structure or protocol which is backwards compatible and does not add any new features.
//  e.g. fixing a bug, changing error messages, changing a functions internals, ect.
pub static NETWORK_VERSION: [u8; 3] = [0, 0, 0];


pub struct Packet {
    pub version: [u8; 3],
    pub username: String,
    pub group: String,
    pub payload: Box<dyn NetworkCommand + Send + Sync>,
}

impl fmt::Debug for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Packet")
            .field("version", &self.version)
            .field("username", &self.username)
            .field("group", &self.group)
            .field("payload_number", &self.payload.number())
            .finish()
    }
}

impl Clone for Packet {
    fn clone(&self) -> Self {
        let payload = parse_command(serialize(&self.payload).unwrap_or_default())
            .unwrap_or_else(|_| Box::new(DefaultCommand {}));

        Packet {
            version: self.version,
            username: self.username.clone(),
            group: self.group.clone(),
            payload,
        }
    }
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
