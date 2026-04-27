use std::fmt::Display;

/// Actions related to the user authentication.
#[derive(Debug, Clone)]
pub enum AuthAction {
    /// Authenticates a user to the Matrix server.
    Authenticate { username: String, password: String },
    /// Attempt to restore a session from disk.
    RestoreSession,
}

impl Display for AuthAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthAction::Authenticate { username, .. } => {
                write!(f, "Authenticate {{ username: {} }}", username)
            }
            AuthAction::RestoreSession => write!(f, "RestoreSession"),
        }
    }
}
