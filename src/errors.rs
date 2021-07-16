use std::{error, fmt, string};

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

impl From<tokio::task::JoinError> for RuntError {
    fn from(err: tokio::task::JoinError) -> Self {
        RuntError(err.to_string())
    }
}

impl From<tokio::time::error::Elapsed> for RuntError {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        RuntError(err.to_string())
    }
}

// Helper method to collapse nested Results
pub trait RichResult<T, E> {
    fn collapse(self) -> Result<T, E>;
}

impl<T, E> RichResult<T, E> for Result<Result<Result<T, E>, E>, E> {
    fn collapse(self) -> Result<T, E> {
        match self {
            Ok(Ok(Ok(v))) => Ok(v),
            Ok(Ok(Err(e))) => Err(e),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(e),
        }
    }
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

pub trait RichVec<T, E> {
    fn partition_results(self) -> (Vec<T>, Vec<E>);
}
impl<T, E> RichVec<T, E> for Vec<Result<T, E>>
where
    T: std::fmt::Debug,
    E: std::fmt::Debug,
{
    fn partition_results(self) -> (Vec<T>, Vec<E>) {
        let (ts, es): (Vec<_>, Vec<_>) =
            self.into_iter().partition(|el| el.is_ok());
        (
            ts.into_iter().map(Result::unwrap).collect::<Vec<T>>(),
            es.into_iter().map(Result::unwrap_err).collect::<Vec<E>>(),
        )
    }
}
