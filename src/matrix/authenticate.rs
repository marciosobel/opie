use anyhow::Result;
use matrix_sdk::{AuthSession, Client, authentication::matrix::MatrixSession};

use crate::{async_dropper::AsyncDropper, matrix::session_path};

/// Authenticates the user with the Matrix server.
///
/// Authentication will restore a session if it exists, or log in with provided credentials,
/// saving the session for future use.
pub async fn authenticate(
    client: AsyncDropper<Client>,
    username: String,
    password: String,
) -> Result<(), matrix_sdk::Error> {
    match restore_session(client.clone()).await? {
        RestoreStatus::Restored => {}
        RestoreStatus::NoSession => {
            login(client.clone(), username, password).await?;
            save_session(client).await?;
        }
    }

    Ok(())
}

/// Logs in to the Matrix server with the provided username and password.
pub async fn login(
    client: AsyncDropper<Client>,
    username: String,
    password: String,
) -> Result<(), matrix_sdk::Error> {
    client
        .matrix_auth()
        .login_username(&username, &password)
        .await?;

    Ok(())
}

pub enum RestoreStatus {
    Restored,
    NoSession,
}

/// Restores a session from disk if it exists. Will not error if no session is found, as this is
/// expected for first-time users.
pub async fn restore_session(
    client: AsyncDropper<Client>,
) -> Result<RestoreStatus, matrix_sdk::Error> {
    let session_path = session_file();
    if !session_path.exists() {
        // No session file found, likely because the user has not logged in before.
        return Ok(RestoreStatus::NoSession);
    }

    let serialized_session = std::fs::read_to_string(&session_path)?;
    let session: MatrixSession = serde_json::from_str(&serialized_session)?;
    client.restore_session(session).await?;

    Ok(RestoreStatus::Restored)
}

/// Stores the current session to disk for future use. If no session exists, this is a no-op.
pub async fn save_session(client: AsyncDropper<Client>) -> Result<(), matrix_sdk::Error> {
    let Some(session) = client.session() else {
        // No session to save, likely because the user is not logged in.
        return Ok(());
    };

    let AuthSession::Matrix(session) = session else {
        // This should never happen, as we're only using Matrix authentication.
        panic!("Unexpected OAuth 2.0 session");
    };

    let serialized_session = serde_json::to_string(&session)?;
    let session_path = session_file();
    std::fs::write(session_path, serialized_session)?;

    Ok(())
}

fn session_file() -> std::path::PathBuf {
    session_path().join("session.json")
}
