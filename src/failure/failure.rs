pub enum Failure {
    Error(FailureError),
    Warning(FailureError),
}

pub enum FailureType {
    Error,
    Warning,
}

impl Failure {
    fn message(&self) -> &str {
        self.unwrap().message()
    }
    fn std_err(&self) -> &Option<std::io::Error> {
        self.unwrap().std_err()
    }
    fn is_std_err(&self) -> bool {
        self.std_err().is_some()
    }

    fn from(std_err: Option<std::io::Error>, message: Option<String>, failure_t: FailureType) -> Self {
        match failure_t {
            FailureType::Error => Failure::Error(FailureError::from
        (std_err, message)),
            FailureType::Warning => Failure::Warning(FailureError::from
        (std_err, message)),
        }
    }

    fn unwrap(&self) -> &FailureError {
        match self {
            Failure::Error(e) => e,
            Failure::Warning(w) => w,
        }
    }
}

struct FailureError {
    message: String,
    std_err: Option<std::io::Error>,
}


impl FailureError {
    fn message(&self) -> &str {
        &self.message
    }

    fn std_err(&self) -> &Option<std::io::Error> {
        &self.std_err
    }

    fn from(std_err: Option<std::io::Error>, message: Option<String>) -> Self {
        let message = match (std_err.as_ref(), message) {
            (Some(std_err), None) => std_err.to_string(),
            (None, Some(message)) => message,
            (Some(std_err), Some(message)) => format!("{}: {}", message, std_err),
            (None, None) => panic!("FailureError must have either a std_err or a message"),
        };
        FailureError { message, std_err }
    }
}