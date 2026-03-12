use std::path::PathBuf;

use anyhow::Result;
use matrix_sdk::{
    Client, ClientBuildError, Error, SqliteCryptoStore, SqliteEventCacheStore, SqliteStateStore,
    ThreadingSupport, encryption::EncryptionSettings, search_index::SearchIndexStoreKind,
    store::StoreConfig,
};
use thiserror::Error;

use crate::matrix::session::{ClientSession, Session};

#[derive(Debug, Error)]
pub enum ClientBuildErrorKind {
    #[error("Failed to build the Matrix client: {0}")]
    ClientBuildError(#[from] ClientBuildError),

    #[error("A Matrix error ocurred: {0}")]
    MatrixError(#[from] Error),
}

/// Creates and initializes the Matrix SDK client using the provided homeserver
pub async fn new_client(
    homeserver: String,
    passphrase: String,
) -> Result<(Client, ClientSession), ClientBuildErrorKind> {
    let client = build_client(homeserver.clone(), passphrase.clone(), Session::path()).await?;
    let session = ClientSession::new(homeserver, passphrase, Session::path());

    Ok((client, session))
}

/// Restores a client with the provided session
pub async fn new_client_with_session(session: Session) -> Result<Client, ClientBuildErrorKind> {
    let Session {
        client_session,
        user_session,
    } = session;

    let ClientSession {
        homeserver,
        passphrase,
        session_path,
    } = client_session;

    let client = build_client(homeserver, passphrase, session_path).await?;
    client.restore_session(user_session).await?;

    Ok(client)
}

async fn build_client(
    homeserver: String,
    passphrase: String,
    session_path: PathBuf,
) -> Result<Client, ClientBuildError> {
    let state_store = SqliteStateStore::open(session_path.join("state"), Some(&passphrase)).await?;
    let crypto_store =
        SqliteCryptoStore::open(session_path.join("crypto"), Some(&passphrase)).await?;
    let cache_store =
        SqliteEventCacheStore::open(session_path.join("cache"), Some(&passphrase)).await?;

    let store_config = StoreConfig::new("opie".to_owned())
        .crypto_store(crypto_store)
        .state_store(state_store)
        .event_cache_store(cache_store);

    let encryption_settings = EncryptionSettings {
        auto_enable_backups: true,
        auto_enable_cross_signing: true,
        backup_download_strategy:
            matrix_sdk::encryption::BackupDownloadStrategy::AfterDecryptionFailure,
    };

    let search_index_store = SearchIndexStoreKind::EncryptedDirectory(
        session_path.join("index_data"),
        passphrase.clone(),
    );

    let threading_support = ThreadingSupport::Enabled {
        with_subscriptions: true,
    };

    let client_builder = Client::builder()
        .store_config(store_config)
        .server_name_or_homeserver_url(homeserver)
        .with_encryption_settings(encryption_settings)
        .search_index_store(search_index_store)
        .with_threading_support(threading_support)
        .with_enable_share_history_on_invite(true);

    client_builder.build().await
}
