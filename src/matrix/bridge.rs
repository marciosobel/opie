use std::sync::Arc;

use anyhow::Result;
use iced::futures::{SinkExt, StreamExt, channel::mpsc};
use matrix_sdk::{Client, Room};
use thiserror::Error;

use crate::matrix::services::{self, RestoreStatus as SessionRestoreStatus};

const CHANNEL_SIZE: usize = 64;

/// A bridge between the Matrix SDK and the rest of the application.
#[derive(Clone)]
pub struct Bridge(Client);

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
    RoomList(Vec<Room>),
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
    Authenticated(Bridge),
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

    pub fn get_joined_rooms(&self) -> Vec<Room> {
        self.client().joined_rooms()
    }

    /// Gets a reference to the Matrix SDK client.
    pub fn client(&self) -> Client {
        self.0.clone()
    }
}

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Failed to build the Matrix client: {0}")]
    ClientBuildError(#[from] Arc<matrix_sdk::ClientBuildError>),

    #[error("An error occurred in the Matrix SDK: {0}")]
    SdkError(#[from] Arc<matrix_sdk::Error>),

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
                    let rooms = bridge.get_joined_rooms();
                    send(Event::RoomList(rooms), &mut emitter).await;
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
