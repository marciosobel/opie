use std::{cmp::Ordering, collections::HashMap, sync::Arc};

use anyhow::Result;
use matrix_sdk::{
    Client as ClientInner,
    config::SyncSettings,
    media::MediaFormat,
    ruma::{OwnedRoomId, api::client::filter::FilterDefinition},
};
use tokio::sync::mpsc;

use crate::{
    Channel,
    channel::{self, ActionSender},
    services::{self, Device, Room, Timeline, TimelineEvent, UserInfo},
    session::ClientSession,
};

mod error;
pub use error::Error;

mod event;
pub use event::Event;

mod action;
pub use action::Action;

mod state;
pub(crate) use state::State;

/// A connection that allows to send actions to the matrix backend.
pub type Bridge = ActionSender<Action>;

/// A bridge between the Matrix SDK and the rest of the application.
#[derive(Clone)]
pub struct Client {
    /// The original Matrix client
    inner: ClientInner,
    /// Map of active timeline tasks, where the key is the room ID.
    active_timelines: HashMap<OwnedRoomId, Arc<Timeline>>,
}

impl Client {
    /// Creates a new Matrix client wrapper.
    pub(super) async fn new(
        homeserver: String,
        passphrase: String,
    ) -> Result<(Self, ClientSession), Error> {
        let (client, client_session) = services::new_client(homeserver, passphrase)
            .await
            .map_err(Arc::new)?;

        Ok((Self::with_client(client), client_session))
    }

    /// Creates a new wrapper using the provided client.
    pub(super) fn with_client(client: ClientInner) -> Self {
        Self {
            inner: client,
            active_timelines: HashMap::new(),
        }
    }

    /// Restores a session from disk if it exists.
    pub(super) async fn restore_session() -> Result<Option<Self>, Error> {
        let session = services::restore_session().await.map_err(Arc::new)?;

        match session {
            None => Ok(None),
            Some(session) => {
                let client = services::new_client_with_session(session)
                    .await
                    .map_err(Arc::new)?;

                Ok(Some(Self::with_client(client)))
            }
        }
    }

    /// Authenticates the user with the Matrix server.
    ///
    /// Authentication will restore a session if it exists, or log in with provided credentials,
    /// saving the session for future use.
    pub(super) async fn authenticate(
        &self,
        username: String,
        password: String,
        client_session: ClientSession,
    ) -> Result<(), Error> {
        services::authenticate(self.inner(), username, password, client_session)
            .await
            .map_err(Arc::new)?;

        Ok(())
    }

    /// Synchronize the client’s state with the latest state on the server.
    pub(crate) async fn sync_once(&self) -> Result<(), Error> {
        tracing::info!("Syncing the client once");
        let sync_settings = self.sync_settings_with_lazy_loading();

        self.inner()
            .sync_once(sync_settings)
            .await
            .map_err(Arc::new)?;

        Ok(())
    }

    /// Returns a list of the rooms that the user has joined.
    pub(crate) async fn get_joined_rooms(&self) -> Result<HashMap<OwnedRoomId, Arc<Room>>, Error> {
        let rooms = services::list_joined_rooms(self.inner())
            .await
            .map_err(Arc::new)?;

        let rooms = rooms
            .into_iter()
            .map(|(id, room)| (id, Arc::new(room)))
            .collect();

        Ok(rooms)
    }

    /// Gets a reference to the underlying Matrix SDK client.
    pub(crate) fn inner(&self) -> ClientInner {
        self.inner.clone()
    }

    /// Spawns a task that syncs the client in the background. If you wish to sync the client once, use [`sync_once`](Self::sync_once) instead.
    pub(crate) fn start_sync(&self) {
        let client = self.inner();
        let sync_settings = self.sync_settings_with_lazy_loading();

        tracing::info!("Starting Matrix client sync task");
        tokio::spawn(async move {
            if let Err(error) = client.sync(sync_settings).await {
                tracing::error!("Error during sync: {:?}", error);
            }
        });
    }

    /// Gets the timeline for a room, spawning a background task that listens for updates and updates the timeline accordingly.
    pub(crate) async fn room_timeline(
        &mut self,
        id: OwnedRoomId,
    ) -> Result<mpsc::Receiver<TimelineEvent>, Error> {
        let Some(room) = self.inner().get_room(&id) else {
            return Err(Error::RoomNotFound(id));
        };

        tracing::info!("Subscribing to timeline for room {}", id);
        let (timeline, rx) = services::timeline(room).await.map_err(Arc::new)?;

        let old_timeline = self.active_timelines.insert(id.clone(), Arc::new(timeline));
        if let Some(old_timeline) = old_timeline {
            old_timeline.close().await;
        }

        Ok(rx)
    }

    /// Closes the timeline for a room, aborting the background task that listens for updates.
    /// This should be called when a timeline is no longer needed, such as when leaving a room or closing a timeline view.
    pub(crate) async fn close_timeline(&self, room_id: OwnedRoomId) {
        if let Some(timeline) = self.active_timelines.get(&room_id) {
            tracing::info!("Closing timeline for room {}", room_id);
            timeline.close().await;
        } else {
            tracing::warn!(
                "Received request to close timeline but no timeline handler found for room {}",
                room_id
            );
        }
    }

    /// Return a `SyncSettings` struct with room members lazy-loading,
    /// it will speed up the initial sync a lot with accounts in lots of rooms.
    /// See <https://spec.matrix.org/v1.6/client-server-api/#lazy-loading-room-members>.
    fn sync_settings_with_lazy_loading(&self) -> SyncSettings {
        let filter = FilterDefinition::with_lazy_loading();
        SyncSettings::default().filter(filter.into())
    }

    /// Fetches and returns info about the logged account.
    async fn user_info(&self) -> Result<UserInfo, Error> {
        let client = self.inner();
        let account = client.account();
        let display_name = account.get_display_name().await.map_err(Arc::new)?;
        let id = client.user_id().ok_or(Error::NotAuthenticated)?.to_owned();
        let avatar = account
            .get_avatar(MediaFormat::File)
            .await
            .map_err(Arc::new)?;

        Ok(UserInfo::new(id, display_name, avatar))
    }

    /// List all devices this user is linked to
    pub(crate) async fn devices(&self) -> Result<Vec<Device>, Error> {
        let client = self.inner();
        let user_id = client.user_id().ok_or(Error::NotAuthenticated)?;
        let own_device_id = client.device_id().ok_or(Error::NotAuthenticated)?;

        let mut last_seen_map = HashMap::new();

        let response = client
            .devices()
            .await
            .map_err(|error| Arc::new(matrix_sdk::Error::Http(Box::new(error))))?;

        for device in response.devices {
            let id = device.device_id.to_owned();
            let last_seen = device.last_seen_ts;
            last_seen_map.insert(id, last_seen);
        }

        let mut devices: Vec<_> = client
            .encryption()
            .get_user_devices(user_id)
            .await
            .map_err(Arc::new)?
            .devices()
            .map(Device::from)
            .map(|mut device| {
                device.is_self = device.id == own_device_id;
                device.last_seen = last_seen_map
                    .get(&device.id)
                    .expect("Device ID should match")
                    .to_owned();
                device
            })
            .collect();

        devices.sort_by(|a, b| match (a.last_seen, b.last_seen) {
            (Some(ref a), Some(ref b)) => b.cmp(a),
            _ => Ordering::Equal,
        });

        Ok(devices)
    }
}

/// Creates an [`iced`] subscription for the matrix bridge.
pub fn subscribe() -> iced::Subscription<Event> {
    iced::Subscription::run(|| {
        iced::stream::channel(channel::CHANNEL_SIZE, async |tx| {
            tracing::info!("Subscription handler started");
            let mut state = State::new();
            let (tx, mut channel) = Channel::with_tx(tx);

            channel.send(Event::Stale(tx)).await;
            loop {
                let action = channel.recv().await;
                tracing::info!("Received action: {}", action);
                state.handle_action(action, &mut channel).await;
            }
        })
    })
}
