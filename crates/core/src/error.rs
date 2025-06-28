use std::fmt;

/// Custom error type for the application
#[derive(Debug)]
pub enum Error {
    /// Configuration errors
    Config(String),
    /// Provider errors (API calls, network, etc.)
    Provider(String),
    /// Service container errors
    Service(String),
    /// IO errors
    Io(std::io::Error),
    /// Other errors
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Config(msg) => write!(f, "Configuration error: {}", msg),
            Error::Provider(msg) => write!(f, "Provider error: {}", msg),
            Error::Service(msg) => write!(f, "Service error: {}", msg),
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::Config(format!("TOML parsing error: {}", err))
    }
}

impl From<std::env::VarError> for Error {
    fn from(err: std::env::VarError) -> Self {
        Error::Config(format!("Environment variable error: {}", err))
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as StdError;

    #[test]
    fn test_error_display() {
        let err = Error::Config("Invalid API key".to_string());
        assert_eq!(err.to_string(), "Configuration error: Invalid API key");

        let err = Error::Provider("API rate limit exceeded".to_string());
        assert_eq!(err.to_string(), "Provider error: API rate limit exceeded");

        let err = Error::Service("Service not found".to_string());
        assert_eq!(err.to_string(), "Service error: Service not found");

        let err = Error::Other("Unknown error".to_string());
        assert_eq!(err.to_string(), "Error: Unknown error");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn test_error_source() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "Access denied");
        let err = Error::Io(io_err);
        assert!(StdError::source(&err).is_some());

        let err = Error::Config("Bad config".to_string());
        assert!(StdError::source(&err).is_none());
    }

    #[test]
    fn test_error_from_env_var() {
        let env_err = std::env::VarError::NotPresent;
        let err: Error = env_err.into();
        match err {
            Error::Config(msg) => assert!(msg.contains("Environment variable error")),
            _ => panic!("Expected Config error"),
        }
    }
}