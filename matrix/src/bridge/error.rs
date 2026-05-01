use std::sync::Arc;

use matrix_sdk::ruma::{OwnedDeviceId, OwnedRoomId};
use thiserror::Error;

use crate::services::sas_verification;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Failed to build the Matrix client: {0}")]
    ClientBuildError(#[from] Arc<crate::services::ClientBuildErrorKind>),

    #[error(transparent)]
    SdkError(#[from] Arc<matrix_sdk::Error>),

    #[error(transparent)]
    TimelineError(#[from] Arc<matrix_sdk_ui::timeline::Error>),

    #[error("The action sent is not valid for the current state of the bridge")]
    InvalidAction,

    #[error("Not authenticated")]
    NotAuthenticated,

    #[error("Room {0} not found")]
    RoomNotFound(OwnedRoomId),

    #[error("Device {0} not found")]
    DeviceNotFound(OwnedDeviceId),

    #[error("Timeline {0} not found")]
    TimelineNotFound(OwnedRoomId),

    #[error(transparent)]
    SasVerificationError(#[from] sas_verification::Error),
}
