use std::{any::Any, collections::HashMap};

use crate::{network_command::{ID::NetworkCommandID, helpers::{NetworkCommand, QueryOrNotify}}, failure::failure::{Failure, FailureType}};
use anyhow::anyhow;
use postcard::{from_bytes, to_slice};

#[derive(Debug, Clone)]
pub struct OnlineUsers {
    pub users: HashMap<u16, String>
}

impl NetworkCommand for OnlineUsers {
    fn number(&self) -> u8 {
        OnlineUsers::ID
    }
    fn serialize(&self) -> Result<Vec<u8>, Failure> {
        let mut buffer = [0u8; 512];
        let user_data = to_slice(&self.users, &mut buffer)
            .map_err(|e| Failure::from((anyhow!("failed to serialize OnlineUsers: {e}"), FailureType::Warning)))?;
        let data = user_data.to_vec();

        Ok(data)
    }
    fn parse(data: Vec<u8>) -> Result<Self, Failure> where Self: Sized {
        let users: HashMap<u16, String> = from_bytes(&data[1..])
            .map_err(|e| Failure::from((anyhow!("failed to parse OnlineUsers: {e}"), FailureType::Warning)))?;

        Ok(OnlineUsers { users })
    }

    fn query_or_notify(&self) -> QueryOrNotify {
        QueryOrNotify::Query
    }

    fn as_any(&self) -> &dyn Any { self }

}