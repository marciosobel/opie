use std::sync::Arc;

use anyhow::Result;
use iced::futures::{SinkExt, StreamExt, channel::mpsc};
use matrix_sdk::Client;
use matrix_sdk_ui::{
    RoomListService,
    room_list_service::{self, RoomList},
    sync_service::{self, SyncService},
};
use thiserror::Error;

use crate::matrix::services::{self, RestoreStatus as SessionRestoreStatus, Rooms};

const CHANNEL_SIZE: usize = 64;

/// A bridge between the Matrix SDK and the rest of the application.
#[derive(Clone)]
pub struct Bridge(Client);

/// Services provided by the Matrix client
#[derive(Clone)]
pub struct Services {
    /// A reference to the room list service.
    room_list: Arc<RoomListService>,
    /// A reference to the sync service.
    sync: Arc<SyncService>,
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
    /// A list of rooms that the user is a member of.
    RoomList(Rooms),
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
    /// Gets the underlying [room_list_service]
    ListAllRooms,
}

/// Current state of the Matrix bridge.
enum State {
    /// The bridge is connected, but is waiting for the homeserver name to be provided.
    WaitingForServerName,
    /// The bridge has been initialized and is ready to use.
    Initialized(Bridge),
    /// The user has been authenticated and the services have been initialized.
    Authenticated { services: Services },
}

/// A sender for sending actions to the Matrix bridge.
pub type MatrixBridgeSender = mpsc::Sender<Action>;

impl Bridge {
    /// Creates a new Matrix bridge.
    pub async fn new(server: String) -> Result<Self, Error> {
        let bridge = services::new_client(server)
            .await
            .map(Self)
            .map_err(Arc::new)?;
        Ok(bridge)
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

    /// Restores a session from disk if it exists.
    pub async fn restore_session(&self) -> Result<SessionRestoreStatus, Error> {
        services::restore_session(self.client())
            .await
            .map_err(|error| Arc::new(error).into())
    }

    /// Gets a reference to the Matrix SDK client.
    pub fn client(&self) -> Client {
        self.0.clone()
    }
}

impl Services {
    /// Creates a new instance of the services provided by the Matrix client.
    pub async fn new(client: Client) -> Result<Self, Error> {
        if !services::is_authenticated(&client) {
            return Err(Error::NotAuthenticated);
        }

        let sync = SyncService::builder(client.clone())
            .build()
            .await
            .map(Arc::new)
            .map_err(Arc::new)?;

        let room_list = sync.room_list_service();

        sync.start().await;

        Ok(Services { room_list, sync })
    }

    /// Returns the room list from the [`RoomListService`]
    pub async fn room_list(&self) -> Result<RoomList, Error> {
        let room_list = self.room_list.all_rooms().await.map_err(Arc::new)?;
        Ok(room_list)
    }
}

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Failed to build the Matrix client: {0}")]
    ClientBuildError(#[from] Arc<matrix_sdk::ClientBuildError>),

    #[error("An error occurred in the Matrix SDK: {0}")]
    SdkError(#[from] Arc<matrix_sdk::Error>),

    #[error("An error ocurred in the room list service: {0}")]
    RoomListServiceError(#[from] Arc<room_list_service::Error>),

    #[error("An error ocurred in the sync service: {0}")]
    SyncServiceError(#[from] Arc<sync_service::Error>),

    #[error("The action sent is not valid for the current state of the bridge")]
    InvalidAction,

    #[error("Not authenticated")]
    NotAuthenticated,
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
                        let services = match Services::new(bridge.client()).await {
                            Ok(services) => services,
                            Err(error) => {
                                send(error.into(), &mut emitter).await;
                                continue;
                            }
                        };

                        state = State::Authenticated { services };
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
                                let services = match Services::new(bridge.client()).await {
                                    Ok(services) => services,
                                    Err(error) => {
                                        send(error.into(), &mut emitter).await;
                                        continue;
                                    }
                                };
                                state = State::Authenticated { services };
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
                State::Authenticated { services } => {
                    let room_list = match services.room_list().await {
                        Ok(room_list) => Arc::new(room_list),
                        Err(error) => {
                            send(error.into(), &mut emitter).await;
                            continue;
                        }
                    };

                    let rooms = services::list_rooms(&room_list).await;
                    send(Event::RoomList(rooms), &mut emitter).await
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
            Event::RoomList(generic_vector) => {
                write!(f, "RoomList({} rooms)", generic_vector.len())
            }
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
        }
    }
}
