use matrix_sdk::ruma::{OwnedMxcUri, OwnedUserId};

/// An action related to users.
#[derive(Debug, Clone)]
pub enum UserAction {
    /// Fetches the avatar of the provided [`UserId`](OwnedUserId)
    FetchUserAvatar(OwnedUserId, OwnedMxcUri),
}

impl std::fmt::Display for UserAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserAction::FetchUserAvatar(user_id, _) => {
                write!(f, "FetchUserAvatar {{ user_id: {}, .. }}", user_id)
            }
        }
    }
}
