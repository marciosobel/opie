use std::sync::Arc;

use anyhow::Result;
use iced::futures::{SinkExt, Stream, StreamExt, channel::mpsc};
use matrix_sdk::Client;
use thiserror::Error;

use crate::matrix::services::{self, RestoreStatus as SessionRestoreStatus};

const CHANNEL_SIZE: usize = 64;

/// A bridge between the Matrix SDK and the rest of the application.
pub struct Bridge {
    /// A reference to the main SDK client.
    client: Client,
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
}

/// Current state of the Matrix bridge.
enum State {
    /// The bridge is connected, but is waiting for the homeserver name to be provided.
    WaitingForServerName,
    /// The bridge has been initialized and is ready to use.
    Initialized(Bridge),
}

/// A sender for sending actions to the Matrix bridge.
pub type MatrixBridgeSender = mpsc::Sender<Action>;

impl Bridge {
    /// Creates a connection to the Matrix bridge, returning a stream of its events.
    pub fn connect() -> impl Stream<Item = Event> {
        iced::stream::channel(CHANNEL_SIZE, subscription_handler)
    }

    /// Creates a new Matrix bridge.
    pub async fn new(server: String) -> Result<Self, Error> {
        match services::new_client(server).await {
            Ok(client) => Ok(Self { client }),
            Err(error) => Err(Arc::new(error).into()),
        }
    }

    /// Authenticates the user with the Matrix server.
    ///
    /// Authentication will restore a session if it exists, or log in with provided credentials,
    /// saving the session for future use.
    pub async fn authenticate(
        &self,
        username: String,
        password: String,
    ) -> Result<(), matrix_sdk::Error> {
        services::authenticate(self.client(), username, password).await
    }

    /// Restores a session from disk if it exists.
    pub async fn restore_session(&self) -> Result<SessionRestoreStatus, Error> {
        services::restore_session(self.client())
            .await
            .map_err(|error| Arc::new(error).into())
    }

    /// Gets a reference to the Matrix SDK client.
    pub fn client(&self) -> Client {
        self.client.clone()
    }
}

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("Failed to build the Matrix client")]
    ClientBuildError(#[from] Arc<matrix_sdk::ClientBuildError>),

    #[error("An error occurred in the Matrix SDK")]
    SdkError(#[from] Arc<matrix_sdk::Error>),

    #[error("The action sent is not valid for the current state of the bridge")]
    InvalidAction,
}

async fn subscription_handler(mut emitter: mpsc::Sender<Event>) {
    tracing::info!("Subscription handler started");
    let mut state = State::WaitingForServerName;
    let (sender, mut receiver) = mpsc::channel(CHANNEL_SIZE);
    send(Event::Stale(sender), &mut emitter).await;

    loop {
        let action = receiver.select_next_some().await;
        tracing::info!("Received action: {:?}", action);
        match action {
            Action::CreateMatrixClient { server } => match &state {
                State::WaitingForServerName | State::Initialized(_) => {
                    match Bridge::new(server).await {
                        Ok(bridge) => {
                            send(Event::Ready, &mut emitter).await;
                            state = State::Initialized(bridge);
                        }
                        Err(error) => send(Event::Error(error), &mut emitter).await,
                    }
                }
            },

            Action::Authenticate { username, password } => match &state {
                State::Initialized(bridge) => match bridge.authenticate(username, password).await {
                    Ok(_) => send(Event::Authenticated, &mut emitter).await,
                    Err(error) => send(Event::Error(Arc::new(error).into()), &mut emitter).await,
                },
                _ => invalid_action(&mut emitter).await,
            },

            Action::RestoreSession => match &state {
                State::Initialized(bridge) => match bridge.restore_session().await {
                    Ok(status) => match status {
                        SessionRestoreStatus::Restored => {
                            send(Event::Authenticated, &mut emitter).await
                        }
                        SessionRestoreStatus::NoSession => {
                            send(Event::SessionRestoreFailed, &mut emitter).await
                        }
                    },
                    Err(error) => send(Event::Error(error), &mut emitter).await,
                },
                _ => invalid_action(&mut emitter).await,
            },
        }
    }
}

/// Helper function to send an event to the emitter.
async fn send(event: Event, emitter: &mut mpsc::Sender<Event>) {
    tracing::info!("Sending event: {:?}", event);
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

/// Creates an [`iced`] subscription for the matrix bridge.
pub fn subscribe() -> iced::Subscription<Event> {
    iced::Subscription::run(Bridge::connect)
}
