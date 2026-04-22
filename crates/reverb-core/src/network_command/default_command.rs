use std::any::Any;

use crate::{network_command::{ID::NetworkCommandID, helpers::{NetworkCommand, QueryOrNotify}}, failure::failure::{Failure, FailureType}};
use anyhow::anyhow;

#[derive(Debug, Clone)]
pub struct DefaultCommand {}

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