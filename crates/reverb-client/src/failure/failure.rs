use anyhow::Error as AnyError;

pub enum Failure {
    Fetal(AnyError, String),
    Warning(AnyError, String),
}

pub enum FailureType {
    Fetal,
    Warning,
}

impl Failure {
    pub fn failure_type(&self) -> FailureType {
        match self {
            Failure::Fetal(_, _) => FailureType::Fetal,
            Failure::Warning(_, _) => FailureType::Warning,
        }
    }
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Failure::Fetal(e, msg) => format!("Fetal error: {}: {}", e, msg),
            Failure::Warning(e, msg) => format!("Warning: {}: {}", e, msg),
        })
    }
}

impl From<(anyhow::Error, FailureType)> for Failure {
    fn from((err, failure_type): (anyhow::Error, FailureType)) -> Self {
        let e: AnyError = err;
        match failure_type {
            FailureType::Fetal => Failure::Fetal(e, String::new()),
            FailureType::Warning => Failure::Warning(e, String::new()),
        }
    }
}

impl From<(anyhow::Error, &str, FailureType)> for Failure {
    fn from((err, msg, failure_type): (anyhow::Error, &str, FailureType)) -> Self {
        let e: AnyError = err;
        match failure_type {
            FailureType::Fetal => Failure::Fetal(e, msg.into()),
            FailureType::Warning => Failure::Warning(e, msg.into()),
        }
    }
}