use std::path::PathBuf;

use matrix_sdk::authentication::matrix::MatrixSession;
use serde::{Deserialize, Serialize};

/// Data needed to re-build a client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientSession {
    /// The URL of the homeserver of the user
    pub homeserver: String,

    /// The passphrase of the database
    pub passphrase: String,

    /// The path of the database
    pub session_path: PathBuf,
}

/// A Matrix session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// The data needed to re-build a Matrix client.
    pub client_session: ClientSession,

    /// The Matrix user session.
    pub user_session: MatrixSession,
}

impl ClientSession {
    pub fn new(homeserver: String, passphrase: String, session_path: PathBuf) -> Self {
        Self {
            homeserver,
            passphrase,
            session_path,
        }
    }
}

impl Session {
    pub fn new(user_session: MatrixSession, client_session: ClientSession) -> Self {
        Self {
            client_session,
            user_session,
        }
    }

    /// Returns the path where session specific data should be stored.
    pub fn path() -> std::path::PathBuf {
        let dir = dirs::data_dir()
            .expect("Failed to get data dir")
            .join("opie/");

        if !dir.exists() {
            std::fs::create_dir_all(dir.clone()).expect("Failed to create session directory");
        }

        dir
    }
}
