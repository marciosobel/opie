use anyhow::Result;
use matrix_sdk::{Client, Error, authentication::matrix::MatrixSession};

use crate::session::{ClientSession, Session};

/// Authenticates the user with the Matrix server.
///
/// Authentication will restore a session if it exists, or log in with provided credentials,
/// saving the session for future use.
pub async fn authenticate(
    client: Client,
    username: String,
    password: String,
    client_session: ClientSession,
) -> Result<(), matrix_sdk::Error> {
    let user_session = login(client.clone(), username, password).await?;
    let session = Session::new(user_session, client_session);
    save_session(&session)?;

    Ok(())
}

/// Logs in to the Matrix server with the provided username and password.
pub async fn login(
    client: Client,
    username: String,
    password: String,
) -> Result<MatrixSession, matrix_sdk::Error> {
    let matrix_auth = client.matrix_auth();

    tracing::info!("Logging in with credentials");
    matrix_auth
        .login_username(&username, &password)
        .initial_device_display_name("Opie for desktop")
        .await?;

    let user_session = matrix_auth.session().unwrap();
    Ok(user_session)
}

/// Restores a session from disk if it exists. Will not error if no session is found, as this is
/// expected for first-time users.
pub async fn restore_session() -> Result<Option<Session>, Error> {
    tracing::info!("Restoring session");
    let session_path = session_file();
    if !session_path.exists() {
        tracing::info!("No session file found, returning");
        return Ok(None);
    }

    let serialized_session = std::fs::read_to_string(&session_path)?;
    let session: Session = serde_json::from_str(&serialized_session)?;
    tracing::info!("Session restored from file");

    Ok(Some(session))
}

/// Stores the current session to disk for future use. If no session exists, this is a no-op.
pub fn save_session(session: &Session) -> Result<(), matrix_sdk::Error> {
    tracing::info!("Saving session");
    let serialized_session = serde_json::to_string(&session)?;
    let session_path = session_file();
    std::fs::write(session_path, serialized_session)?;

    Ok(())
}

/// Checks if the client is currently authenticated by verifying if a session exists.
pub fn is_authenticated(client: &Client) -> bool {
    client.access_token().is_some()
}

fn session_file() -> std::path::PathBuf {
    Session::path().join("session.json")
}
