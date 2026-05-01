use crate::services::sas_verification::Action as SasAction;

pub mod auth;
pub mod media;
pub mod timeline;

pub use auth::AuthAction;
use matrix_sdk::ruma::OwnedUserId;
pub use media::MediaAction;
pub use timeline::TimelineAction;

/// Actions (or commands) that can be sent to the Matrix bridge.
#[derive(Debug, Clone)]
pub enum Action {
    /// Create the matrix client with the given homeserver.
    CreateMatrixClient {
        /// The homeserver to connect to.
        homeserver: String,
        /// The passphrase to use to encrypt the database.
        passphrase: String,
    },
    /// An action related to authentication.
    Auth(AuthAction),
    /// An action related to timelines.
    Timeline(TimelineAction),
    /// An action related to media.
    Media(MediaAction),
    /// Gets the list to all rooms
    ListAllRooms,
    /// Get the devices this account is linked to
    GetDevices,
    /// An action related to the SAS authentication.
    SasVerification(SasAction),
    /// Gets the user profile of the provided [`UserId`].
    GetUser(OwnedUserId),
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
            Action::Media(action) => write!(f, "Media::{}", action),
            Action::GetUser(id) => write!(f, "GetUser({})", id),
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
from!(MediaAction => Media);
