trait Failure {
    fn message(&self) -> &str;
    fn std_err(&self) -> Option<&std::io::Error>;
    fn is_std_err(&self) -> bool {
        self.std_err().is_some()
    }
    fn from_message(message: String) -> Self;
    fn from_std_err(std_err: std::io::Error) -> Self;
}

struct Error {
    message: String,
    std_err: Option<std::io::Error>,
}

struct Warning {
    message: String,
    std_err: Option<std::io::Error>,
}

impl Failure for Error {
    fn message(&self) -> &str {
        &self.message
    }

    fn std_err(&self) -> Option<&std::io::Error> {
        self.std_err.as_ref()
    }

    fn from_message(message: String) -> Self {
        Self {
            message,
            std_err: None,
        }
    }

    fn from_std_err(std_err: std::io::Error) -> Self {
        Self {
            message: std_err.to_string(),
            std_err: Some(std_err),
        }
    }
}

impl Failure for Warning {
    fn message(&self) -> &str {
        &self.message
    }

    fn std_err(&self) -> Option<&std::io::Error> {
        self.std_err.as_ref()
    }

    fn from_message(message: String) -> Self {
        Self {
            message,
            std_err: None,
        }
    }

    fn from_std_err(std_err: std::io::Error) -> Self {
        Self {
            message: std_err.to_string(),
            std_err: Some(std_err),
        }
    }
}