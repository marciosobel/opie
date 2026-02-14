use anyhow::Result;
use matrix_sdk::{
    Client, ClientBuildError, SqliteCryptoStore, SqliteEventCacheStore, SqliteStateStore,
    ThreadingSupport, encryption::EncryptionSettings, search_index::SearchIndexStoreKind,
    store::StoreConfig,
};

use crate::matrix::get_session_path;

/// Creates and initializes the Matrix SDK client
///
/// * `server` - The server name or the homeserver url to connect to
pub async fn init_client(server: String) -> Result<Client, ClientBuildError> {
    let session_path = get_session_path();
    let store_config = StoreConfig::new("opie".to_owned())
        .crypto_store(SqliteCryptoStore::open(session_path.join("crypto"), None).await?)
        .state_store(SqliteStateStore::open(session_path.join("state"), None).await?)
        .event_cache_store(SqliteEventCacheStore::open(session_path.join("cache"), None).await?);

    let search_index_store =
        SearchIndexStoreKind::UnencryptedDirectory(session_path.join("index_data"));

    let encryption_settings = EncryptionSettings {
        auto_enable_backups: true,
        auto_enable_cross_signing: true,
        backup_download_strategy:
            matrix_sdk::encryption::BackupDownloadStrategy::AfterDecryptionFailure,
    };

    let threading_support = ThreadingSupport::Enabled {
        with_subscriptions: true,
    };

    let client_builder = Client::builder()
        .store_config(store_config)
        .server_name_or_homeserver_url(server)
        .with_encryption_settings(encryption_settings)
        .search_index_store(search_index_store)
        .with_threading_support(threading_support)
        .with_enable_share_history_on_invite(true);

    client_builder.build().await
}
