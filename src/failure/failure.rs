use std::fmt::Debug;
use anyhow::{Error as AnyError, anyhow};

pub enum Failure {
    Fetal(AnyError),
    Warning(AnyError),
}

pub enum FailureType {
    Fetal,
    Warning,
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Failure::Fetal(e) => format!("Fetal error: {}", e),
            Failure::Warning(e) => format!("Warning: {}", e),
        })
    }
}

impl From<(anyhow::Error, FailureType)> for Failure {
    fn from((err, failure_type): (anyhow::Error, FailureType)) -> Self {
        let e: AnyError = err;
        match failure_type {
            FailureType::Fetal => Failure::Fetal(e),
            FailureType::Warning => Failure::Warning(e),
        }
    }
}