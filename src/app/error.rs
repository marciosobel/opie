use matrix::bridge;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AppError {
    #[error("Matrix bridge error: {0}")]
    MatrixBridgeError(#[from] bridge::Error),

    #[error("Failed to save settings")]
    SettingsSaveError(#[from] Arc<anyhow::Error>),

    #[error("No session found to restore")]
    MatrixSessionRestoreNotFound,
}
