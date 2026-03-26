use anyhow::Error as AnyError;

#[derive(Debug)]
pub enum Failure {
    Fatal(AnyError, String),
    Warning(AnyError, String),
}

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
        write!(f, "{}", match self {
            Failure::Fatal(e, msg) => format!("Fatal error: {}: {}", e, msg),
            Failure::Warning(e, msg) => format!("Warning: {}: {}", e, msg),
        })
    }
}

impl From<(anyhow::Error, FailureType)> for Failure {
    fn from((err, failure_type): (anyhow::Error, FailureType)) -> Self {
        let e: AnyError = err;
        match failure_type {
            FailureType::Fatal => Failure::Fatal(e, String::new()),
            FailureType::Warning => Failure::Warning(e, String::new()),
        }
    }
}

impl From<(anyhow::Error, &str, FailureType)> for Failure {
    fn from((err, msg, failure_type): (anyhow::Error, &str, FailureType)) -> Self {
        let e: AnyError = err;
        match failure_type {
            FailureType::Fatal => Failure::Fatal(e, msg.into()),
            FailureType::Warning => Failure::Warning(e, msg.into()),
        }
    }
}
