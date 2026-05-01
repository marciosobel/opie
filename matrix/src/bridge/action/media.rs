use matrix_sdk::ruma::{OwnedMxcUri, OwnedUserId};

/// An action related to users.
#[derive(Debug, Clone)]
pub enum MediaAction {
    /// Fetches the avatar of the provided [`UserId`](OwnedUserId)
    FetchUserAvatar(OwnedUserId, OwnedMxcUri),
}

impl std::fmt::Display for MediaAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaAction::FetchUserAvatar(user_id, _) => {
                write!(f, "FetchUserAvatar {{ user_id: {}, .. }}", user_id)
            }
        }
    }
}
