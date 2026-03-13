use matrix_sdk::ruma::OwnedRoomId;

/// Actions (or commands) that can be sent to the Matrix bridge.
#[derive(Debug)]
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
            Action::GetTimeline(owned_room_id) => {
                write!(f, "GetTimeline {{ room_id: {} }}", owned_room_id)
            }
            Action::CloseTimeline(owned_room_id) => {
                write!(f, "CloseTimeline {{ room_id: {} }}", owned_room_id)
            }
        }
    }
}
