use std::any::Any;

use crate::{network_command::{ID::NetworkCommandID, helpers::{NetworkCommand, QueryOrNotify}}, failure::failure::{Failure, FailureType}};
use anyhow::anyhow;


#[derive(Debug, Clone)]
pub struct Echo {
    pub echo_type: EchoType,
    pub echo_target: String
}

#[derive(Debug, Clone)]
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

impl NetworkCommand for Echo {
    fn number(&self) -> u8 {
        Echo::ID
    }
    fn serialize(&self) -> Result<Vec<u8>, Failure> {
        let mut data = match self.echo_type {
            EchoType::Group => vec![EchoType::Group as u8],
            EchoType::User => vec![EchoType::User as u8],
            _ => {return Err(Failure::from((anyhow!("failed to serialize Echo: EchoType not found"), FailureType::Warning)))}
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