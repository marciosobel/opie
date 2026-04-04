use std::sync::Arc;

use matrix_sdk::ruma::{OwnedDeviceId, OwnedRoomId};
use thiserror::Error;

use crate::matrix::services::sas_verification;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Failed to build the Matrix client: {0}")]
    ClientBuildError(#[from] Arc<crate::matrix::services::ClientBuildErrorKind>),

    #[error("An error occurred in the Matrix SDK: {0}")]
    SdkError(#[from] Arc<matrix_sdk::Error>),

    #[error("An error occurred in the Matrix SDK UI timeline: {0}")]
    TimelineError(#[from] Arc<matrix_sdk_ui::timeline::Error>),

    #[error("The action sent is not valid for the current state of the bridge")]
    InvalidAction,

    #[error("Not authenticated")]
    NotAuthenticated,

    #[error("Room {0} not found")]
    RoomNotFound(OwnedRoomId),

    #[error("Device {0} not found")]
    DeviceNotFound(OwnedDeviceId),

    #[error("Sas verification error: {0}")]
    SasVerificationError(#[from] sas_verification::Error),
}
