use std::{fmt, string, error};

/// An error from Runt.
pub struct RuntError(pub String);

impl fmt::Debug for RuntError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for RuntError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for RuntError {}

impl From<string::FromUtf8Error> for RuntError {
    fn from(err: string::FromUtf8Error) -> Self {
        RuntError(err.to_string())
    }
}

impl From<std::io::Error> for RuntError {
    fn from(err: std::io::Error) -> Self {
        RuntError(err.to_string())
    }
}

// Helper method to collapse nested Results
pub trait RichResult<T, E> {
    fn collapse(self) -> Result<T, E>;
}

impl<T, E> RichResult<T, E> for Result<Result<T, E>, E> {
    fn collapse(self) -> Result<T, E> {
        match self {
            Ok(Ok(v)) => Ok(v),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(e),
        }
    }
}

