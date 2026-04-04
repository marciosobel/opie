use matrix_sdk::ruma::OwnedRoomId;

use crate::matrix::services::sas_verification;

/// Actions (or commands) that can be sent to the Matrix bridge.
#[derive(Debug, Clone)]
pub enum Action {
    /// Create the matrix client with the given homeserver
    CreateMatrixClient {
        homeserver: String,
        passphrase: String,
    },
    /// Authenticates a user to the Matrix server.
    Authenticate { username: String, password: String },
    /// Attempt to restore a session from disk.
    RestoreSession,
    /// Gets the list to all rooms
    ListAllRooms,
    /// Gets the timeline for the given room.
    GetTimeline(OwnedRoomId),
    /// Closes the timeline for the given room, aborting the background task that listens for updates.
    CloseTimeline(OwnedRoomId),
    /// Get the devices this account is linked to
    GetDevices,
    /// Creates an emoji verification request with the specified `DeviceId`.
    SasVerification(sas_verification::Action),
}

impl From<sas_verification::Action> for Action {
    fn from(value: sas_verification::Action) -> Self {
        Self::SasVerification(value)
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::CreateMatrixClient { homeserver, .. } => {
                write!(f, "CreateMatrixClient {{ homeserver: {}, .. }}", homeserver,)
            }
            Action::Authenticate { username, .. } => {
                write!(f, "Authenticate {{ username: {} }}", username)
            }
            Action::RestoreSession => write!(f, "RestoreSession"),
            Action::ListAllRooms => write!(f, "ListAllRooms"),
            Action::GetTimeline(room_id) => {
                write!(f, "GetTimeline {{ room_id: {} }}", room_id)
            }
            Action::CloseTimeline(room_id) => {
                write!(f, "CloseTimeline {{ room_id: {} }}", room_id)
            }
            Action::GetDevices => write!(f, "GetDevices"),
            Action::SasVerification(action) => {
                write!(f, "SasVerification({})", action)
            }
        }
    }
}
