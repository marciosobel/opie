use std::sync::Arc;

use matrix_sdk::Error as MatrixError;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Matrix SDK error: {0}")]
    MatrixError(#[from] Arc<MatrixError>),
    #[error("Invalid verification type")]
    InvalidVerificationType,
    #[error("Not authenticated")]
    NotAuthenticated,
    #[error("Device not found")]
    DeviceNotFound,
}

impl Into<Error> for MatrixError {
    fn into(self) -> Error {
        Error::MatrixError(Arc::new(self))
    }
}
