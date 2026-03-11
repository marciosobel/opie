use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use iced::futures::{SinkExt, StreamExt, channel::mpsc};
use matrix_sdk::{Client, config::SyncSettings, ruma::OwnedRoomId};
use thiserror::Error;

use crate::matrix::services::{
    self, RestoreStatus as SessionRestoreStatus, Room, TimelineUpdateEvent,
};

const CHANNEL_SIZE: usize = 64;

/// A bridge between the Matrix SDK and the rest of the application.
#[derive(Clone)]
pub struct Bridge {
    client: Client,
    active_timelines: HashMap<OwnedRoomId, Arc<services::Timeline>>,
}

/// Events emitted by the Matrix bridge.
#[derive(Debug, Clone)]
pub enum Event {
    /// The Matrix bridge has been initialized and is waiting for the homeserver name to be provided.
    Stale(mpsc::Sender<Action>),
    /// The Matrix client has been built and is ready to use.
    Ready,
    /// An error that occurred in the Matrix bridge.
    Error(Error),
    /// The user has been authenticated with the Matrix server.
    Authenticated,
    /// A session restore was attempted, but no session was found on disk or it was expired.
    SessionRestoreFailed,
    /// The client is syncronizing with the server, which may take some time. The client is not ready to use until the sync is complete.
    Syncing,
    /// A list of rooms that the user is a member of.
    RoomList(Arc<HashMap<OwnedRoomId, Room>>),
    /// The timeline for a room has been updated with new events or changes to existing events. The diff contains the changes that were made to the timeline.
    TimelineEvent(TimelineUpdateEvent),
}

/// Actions (or commands) that can be sent to the Matrix bridge.
#[derive(Debug)]
pub enum Action {
    /// Store the matrix server provider in the settings
    CreateMatrixClient { server: String },
    /// Authenticates a user to the Matrix server.
    Authenticate { username: String, password: String },
    /// Attempt to restore a session from disk.
    RestoreSession,
    /// Gets the list to all rooms
    ListAllRooms,
    /// Gets the timeline for the given room.
    GetTimeline(OwnedRoomId),
    /// Closes the timeline for the given room, aborting the background task that listens for updates.
    CloseTimeline(OwnedRoomId),
}

/// Current state of the Matrix bridge.
enum State {
    /// The bridge is connected, but is waiting for the homeserver name to be provided.
    WaitingForServerName,
    /// The bridge has been initialized and is ready to use.
    Initialized(Bridge),
    /// The user has been authenticated and the services have been initialized.
    Authenticated(Bridge),
}

/// A sender for sending actions to the Matrix bridge.
pub type MatrixBridgeSender = mpsc::Sender<Action>;

impl Bridge {
    /// Creates a new Matrix bridge.
    pub async fn new(server: String) -> Result<Self, Error> {
        let client = services::new_client(server).await.map_err(Arc::new)?;

        Ok(Self {
            client,
            active_timelines: HashMap::new(),
        })
    }

    /// Authenticates the user with the Matrix server.
    ///
    /// Authentication will restore a session if it exists, or log in with provided credentials,
    /// saving the session for future use.
    pub async fn authenticate(&self, username: String, password: String) -> Result<(), Error> {
        services::authenticate(self.client(), username, password)
            .await
            .map_err(Arc::new)?;

        Ok(())
    }

    /// Synchronize the client’s state with the latest state on the server.
    pub async fn sync_once(&self) -> Result<(), Error> {
        tracing::info!("Syncing the client once");
        let sync_settings = SyncSettings::default();
        self.client()
            .sync_once(sync_settings)
            .await
            .map_err(Arc::new)?;

        Ok(())
    }

    /// Restores a session from disk if it exists.
    pub async fn restore_session(&self) -> Result<SessionRestoreStatus, Error> {
        services::restore_session(self.client())
            .await
            .map_err(|error| Arc::new(error).into())
    }

    /// Returns a list of the rooms that the user has joined.
    pub async fn get_joined_rooms(&self) -> Result<Arc<HashMap<OwnedRoomId, Room>>, Error> {
        let rooms = services::list_joined_rooms(self.client())
            .await
            .map_err(Arc::new)?;

        Ok(Arc::new(rooms))
    }

    /// Gets a reference to the Matrix SDK client.
    pub fn client(&self) -> Client {
        self.client.clone()
    }

    /// Spawns a task that syncs the client in the background. If you wish to sync the client once, use [`sync_once`](Self::sync_once) instead.
    pub fn start_sync(&self) {
        let client = self.client();

        tracing::info!("Starting Matrix client sync task");
        tokio::spawn(async move {
            if let Err(error) = client.sync(SyncSettings::default()).await {
                tracing::error!("Error during sync: {:?}", error);
            }
        });
    }

    /// Gets the timeline for a room, spawning a background task that listens for updates and updates the timeline accordingly.
    pub async fn room_timeline(
        &mut self,
        id: OwnedRoomId,
    ) -> Result<tokio::sync::mpsc::Receiver<TimelineUpdateEvent>, Error> {
        let Some(room) = self.client().get_room(&id) else {
            return Err(Error::RoomNotFound(id));
        };

        tracing::info!("Subscribing to timeline for room {}", id);
        let (timeline, rx) = services::timeline(room).await.map_err(Arc::new)?;

        let old_timeline = self.active_timelines.insert(id.clone(), Arc::new(timeline));

        // drop the old timeline, preventing duplicate events.
        {
            if old_timeline.is_some() {
                tracing::info!("Old timeline handler found for room {}", &id);
            }
        }

        Ok(rx)
    }

    /// Closes the timeline for a room, aborting the background task that listens for updates.
    /// This should be called when a timeline is no longer needed, such as when leaving a room or closing a timeline view.
    pub async fn close_timeline(&self, room_id: OwnedRoomId) {
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
}

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Failed to build the Matrix client: {0}")]
    ClientBuildError(#[from] Arc<matrix_sdk::ClientBuildError>),

    #[error("An error occurred in the Matrix SDK: {0}")]
    SdkError(#[from] Arc<matrix_sdk::Error>),

    #[error("An error occurred in the Matrix SDK UI timeline: {0}")]
    TimelineError(#[from] Arc<matrix_sdk_ui::timeline::Error>),

    #[error("The action sent is not valid for the current state of the bridge")]
    InvalidAction,

    #[error("Not authenticated")]
    NotAuthenticated,

    #[error("Room {0} not found")]
    RoomNotFound(OwnedRoomId),
}

impl From<Error> for Event {
    fn from(value: Error) -> Self {
        Event::Error(value)
    }
}

async fn subscription_handler(mut emitter: mpsc::Sender<Event>) {
    tracing::info!("Subscription handler started");
    let mut state = State::WaitingForServerName;
    let (sender, mut receiver) = mpsc::channel(CHANNEL_SIZE);
    send(Event::Stale(sender), &mut emitter).await;

    loop {
        let action = receiver.select_next_some().await;
        tracing::info!("Received action: {}", action);
        match action {
            Action::CreateMatrixClient { server } => match &state {
                State::WaitingForServerName | State::Initialized(_) => {
                    let event = match Bridge::new(server).await {
                        Ok(bridge) => {
                            state = State::Initialized(bridge);
                            Event::Ready
                        }
                        Err(error) => Event::Error(error),
                    };

                    send(event, &mut emitter).await;
                }
                _ => invalid_action(&mut emitter).await,
            },
            Action::Authenticate { username, password } => match &state {
                State::Initialized(bridge) => match bridge.authenticate(username, password).await {
                    Ok(_) => {
                        bridge.start_sync();
                        state = State::Authenticated(bridge.clone());
                        send(Event::Authenticated, &mut emitter).await
                    }
                    Err(error) => send(Event::Error(error), &mut emitter).await,
                },
                State::WaitingForServerName | State::Authenticated { .. } => {
                    invalid_action(&mut emitter).await
                }
            },
            Action::RestoreSession => match &state {
                State::Initialized(bridge) => match bridge.restore_session().await {
                    Ok(status) => {
                        let event = match status {
                            SessionRestoreStatus::Restored => {
                                bridge.start_sync();
                                state = State::Authenticated(bridge.clone());
                                Event::Authenticated
                            }
                            SessionRestoreStatus::NoSession => Event::SessionRestoreFailed,
                        };
                        send(event, &mut emitter).await
                    }
                    Err(error) => send(Event::Error(error), &mut emitter).await,
                },
                _ => invalid_action(&mut emitter).await,
            },
            Action::ListAllRooms => match &state {
                State::Authenticated(bridge) => {
                    let rooms = match bridge.get_joined_rooms().await {
                        Ok(rooms) => rooms,
                        Err(error) => {
                            send(Event::Error(error), &mut emitter).await;
                            continue;
                        }
                    };
                    send(Event::RoomList(rooms), &mut emitter).await;
                }
                State::Initialized(_) | State::WaitingForServerName => {
                    unauthenticated(&mut emitter).await
                }
            },
            Action::GetTimeline(room_id) => match &mut state {
                State::Authenticated(bridge) => {
                    let mut timeline_event_rx = match bridge.room_timeline(room_id.clone()).await {
                        Ok(tuple) => tuple,
                        Err(error) => {
                            send(Event::Error(error), &mut emitter).await;
                            continue;
                        }
                    };

                    let mut emitter_clone = emitter.clone();
                    tokio::spawn(async move {
                        while let Some(timeline_event) = timeline_event_rx.recv().await {
                            send(Event::TimelineEvent(timeline_event), &mut emitter_clone).await;
                        }
                    });
                }
                State::Initialized(_) | State::WaitingForServerName => {
                    unauthenticated(&mut emitter).await
                }
            },
            Action::CloseTimeline(owned_room_id) => match &state {
                State::Authenticated(bridge) => {
                    bridge.close_timeline(owned_room_id.clone()).await;
                    send(
                        Event::TimelineEvent(TimelineUpdateEvent::Closed(owned_room_id)),
                        &mut emitter,
                    )
                    .await;
                }
                State::Initialized(_) | State::WaitingForServerName => {
                    unauthenticated(&mut emitter).await
                }
            },
        }
    }
}

/// Helper function to send an event to the emitter.
async fn send(event: Event, emitter: &mut mpsc::Sender<Event>) {
    tracing::info!("Sending event: {}", event);
    match emitter.send(event).await {
        Ok(_) => (),
        Err(error) => tracing::error!("Failed to send event: {:?}", error),
    }
}

/// Helper function to send a invalid action event to the emitter.
async fn invalid_action(emitter: &mut mpsc::Sender<Event>) {
    tracing::warn!("Received invalid action for current state");
    send(Event::Error(Error::InvalidAction), emitter).await;
}

/// Helper function to send a invalid action event to the emitter.
async fn unauthenticated(emitter: &mut mpsc::Sender<Event>) {
    tracing::warn!("Received action that needs to be authenticated");
    send(Event::Error(Error::NotAuthenticated), emitter).await;
}

/// Creates an [`iced`] subscription for the matrix bridge.
pub fn subscribe() -> iced::Subscription<Event> {
    iced::Subscription::run(|| iced::stream::channel(CHANNEL_SIZE, subscription_handler))
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Stale(sender) => write!(f, "Stale({:?})", sender),
            Event::Ready => write!(f, "Ready"),
            Event::Error(error) => write!(f, "Error({})", error),
            Event::Authenticated => write!(f, "Authenticated"),
            Event::SessionRestoreFailed => write!(f, "SessionRestoreFailed"),
            Event::RoomList(rooms) => {
                write!(f, "RoomList({} rooms)", rooms.len())
            }
            Event::Syncing => write!(f, "Syncing"),
            Event::TimelineEvent(event) => write!(f, "TimelineEvent({})", event),
        }
    }
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::CreateMatrixClient { server } => {
                write!(f, "CreateMatrixClient {{ server: {} }}", server)
            }
            Action::Authenticate { username, .. } => {
                write!(f, "Authenticate {{ username: {} }}", username)
            }
            Action::RestoreSession => write!(f, "RestoreSession"),
            Action::ListAllRooms => write!(f, "ListAllRooms"),
            Action::GetTimeline(owned_room_id) => {
                write!(f, "GetTimeline {{ room_id: {} }}", owned_room_id)
            }
            Action::CloseTimeline(owned_room_id) => {
                write!(f, "CloseTimeline {{ room_id: {} }}", owned_room_id)
            }
        }
    }
}
