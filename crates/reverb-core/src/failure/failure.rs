use anyhow::Error as AnyError;
use std::{env, sync::Arc};

#[derive(Debug, Clone)]
pub enum Failure {
    Fatal(Arc<AnyError>, String),
    Warning(Arc<AnyError>, String),
}

#[derive(Debug, Clone)]
pub enum FailureType {
    Fatal,
    Warning,
}

impl Failure {
    pub fn failure_type(&self) -> FailureType {
        match self {
            Failure::Fatal(_, _) => FailureType::Fatal,
            Failure::Warning(_, _) => FailureType::Warning,
        }
    }
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (e, msg) = match self {
            Failure::Fatal(e, msg) => (e, msg),
            Failure::Warning(e, msg) => (e, msg),
        };
        match self.failure_type() {
            // if fetal, always print the backtrace, if warning, only print the backtrace if the log level is trace or debug
            FailureType::Fatal => write!(f, "{} \n{}", if msg.is_empty() {
                    e.to_string()
                } else {
                    format!("{}: {}", msg, e)
                }, 
                e.backtrace()),
            FailureType::Warning => match env::var("RUST_LOG").unwrap_or("trace".to_string()).to_lowercase().as_str() {
                "trace" | "debug" => write!(f, "{} \n{}", if msg.is_empty() {
                    e.to_string()
                } else {
                    format!("{}: {}", msg, e)
                }, 
                e.backtrace()),
                _ => write!(f, "{}", if msg.is_empty() {
                    e.to_string()
                } else {
                    format!("{}: {}", msg, e)
                }),
            },
        }
    }
}

impl From<(anyhow::Error, FailureType)> for Failure {
    fn from((err, failure_type): (anyhow::Error, FailureType)) -> Self {
        let e: Arc<AnyError> = Arc::new(err);
        match failure_type {
            FailureType::Fatal => Failure::Fatal(e, String::new()),
            FailureType::Warning => Failure::Warning(e, String::new()),
        }
    }
}

impl From<(anyhow::Error, &str, FailureType)> for Failure {
    fn from((err, msg, failure_type): (anyhow::Error, &str, FailureType)) -> Self {
        let e: Arc<AnyError> = Arc::new(err);
        match failure_type {
            FailureType::Fatal => Failure::Fatal(e, msg.into()),
            FailureType::Warning => Failure::Warning(e, msg.into()),
        }
    }
}
