use crate::services::sas_verification::Action as SasAction;

mod auth;
mod timeline;

pub use auth::AuthAction;
pub use timeline::TimelineAction;

/// Actions (or commands) that can be sent to the Matrix bridge.
#[derive(Debug, Clone)]
pub enum Action {
    /// Create the matrix client with the given homeserver
    CreateMatrixClient {
        homeserver: String,
        passphrase: String,
    },
    /// An action related to authentication.
    Auth(AuthAction),
    /// An action related to timelines.
    Timeline(TimelineAction),
    /// Gets the list to all rooms
    ListAllRooms,
    /// Get the devices this account is linked to
    GetDevices,
    /// Creates an emoji verification request with the specified `DeviceId`.
    SasVerification(SasAction),
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::CreateMatrixClient { homeserver, .. } => {
                write!(f, "CreateMatrixClient {{ homeserver: {}, .. }}", homeserver,)
            }
            Action::Auth(action) => write!(f, "Auth::{}", action),
            Action::Timeline(action) => write!(f, "TimelineAction::{}", action),
            Action::ListAllRooms => write!(f, "ListAllRooms"),
            Action::GetDevices => write!(f, "GetDevices"),
            Action::SasVerification(action) => write!(f, "SasVerification::{}", action),
        }
    }
}

macro_rules! from {
    ($from:ident => $to:ident) => {
        impl From<$from> for Action {
            fn from(value: $from) -> Self {
                Self::$to(value)
            }
        }
    };
}

from!(SasAction => SasVerification);
from!(AuthAction => Auth);
from!(TimelineAction => Timeline);
