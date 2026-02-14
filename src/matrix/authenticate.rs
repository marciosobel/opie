use anyhow::Result;
use matrix_sdk::{AuthSession, Client, authentication::matrix::MatrixSession};

use crate::matrix::get_session_path;

/// Authenticates the user with the Matrix server.
///
/// Authentication will restore a session if it exists, or log in with provided credentials,
/// saving the session for future use.
pub async fn authenticate(
    client: Client,
    username: String,
    password: String,
) -> Result<(), matrix_sdk::Error> {
    if restore_session(&client).await? {
        return Ok(());
    }

    login(&client, username, password).await?;
    save_session(&client).await?;

    Ok(())
}

/// Logs in to the Matrix server with the provided username and password.
pub async fn login(
    client: &Client,
    username: String,
    password: String,
) -> Result<(), matrix_sdk::Error> {
    client
        .matrix_auth()
        .login_username(&username, &password)
        .await?;

    Ok(())
}

/// Restores a session from disk if it exists, returning:
/// * `Ok(true)` if a session was successfully restored.
/// * `Ok(false)` if no session was found on disk.
/// * `Err` if an error occurred while reading the session file or restoring the session.
pub async fn restore_session(client: &Client) -> Result<bool, matrix_sdk::Error> {
    let session_path = session_file();
    let result = std::fs::read_to_string(&session_path);

    if let Ok(serialized_session) = result.as_ref() {
        let session: MatrixSession = serde_json::from_str(serialized_session)?;
        client.restore_session(session).await?;
    }

    Ok(result.is_ok())
}

/// Stores the current session to disk for future use. If no session exists, this is a no-op.
pub async fn save_session(client: &Client) -> Result<(), matrix_sdk::Error> {
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
    get_session_path().join("session.json")
}
