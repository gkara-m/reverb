pub enum Failure {
    Fetal(std::io::Error),
    Warning(std::io::Error),
}

pub enum FailureType {
    Error,
    Warning,
}

impl Failure {
    fn message(&self) -> String {
        self.unwrap().to_string()
    }

    fn is_error(&self) -> bool {
        match self {
            Failure::Fetal(_) => true,
            Failure::Warning(_) => false,
        }
    }

    fn std_err(&self) -> &std::io::Error {
        self.unwrap()
    }

    fn unwrap(&self) -> &std::io::Error {
        match self {
            Failure::Fetal(e) => e,
            Failure::Warning(w) => w,
        }
    }
}

impl From<(std::sync::mpsc::SendError<crate::Command>, FailureType)> for Failure {
    fn from((err, failure_type): (std::sync::mpsc::SendError<crate::Command>, FailureType)) -> Self {
        let msg = format!("Failed to send on channel: {}", err);
        match failure_type {
            FailureType::Error => Failure::Fetal(std::io::Error::new(std::io::ErrorKind::BrokenPipe, msg)),
            FailureType::Warning => Failure::Warning(std::io::Error::new(std::io::ErrorKind::BrokenPipe, msg)),
        }
    }
}

impl From<(std::sync::mpsc::RecvError, FailureType)> for Failure {
    fn from((err, failure_type): (std::sync::mpsc::RecvError, FailureType)) -> Self {
        let msg = format!("Failed to receive command response: {}", err);
        match failure_type {
            FailureType::Error => Failure::Fetal(std::io::Error::new(std::io::ErrorKind::BrokenPipe, msg)),
            FailureType::Warning => Failure::Warning(std::io::Error::new(std::io::ErrorKind::BrokenPipe, msg)),
        }
    }
}