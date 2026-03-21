use crate::matrix::session::ClientSession;

use super::Action;
use super::Channel;
use super::ClientWrapper;
use super::Event;

mod authenticated;
mod initialized;
mod waiting_for_client;

/// Current state of the Matrix bridge.
pub(super) enum State {
    /// The bridge channel has been created, but is waiting for the client to be created.
    WaitingForClient,
    /// The bridge has been initialized and is ready to use.
    Initialized {
        client: ClientWrapper,
        client_session: ClientSession,
    },
    /// The user is authenticated and can use the bridge freely.
    Authenticated(ClientWrapper),
}

impl State {
    pub(super) fn new() -> Self {
        Self::WaitingForClient
    }

    /// Handles the action for the current state. Note that, dependending on the action, the state may change.
    pub(super) async fn handle_action(&mut self, action: Action, channel: &mut Channel) {
        let maybe_self = match self {
            State::WaitingForClient => waiting_for_client::handle(action, channel).await,
            State::Initialized {
                client,
                client_session,
            } => initialized::handle(action, channel, client, client_session).await,
            State::Authenticated(client) => authenticated::handle(action, channel, client).await,
        };

        match maybe_self {
            Some(new_state) => *self = new_state,
            None => {}
        }
    }
}

pub(self) async fn create_bridge(
    channel: &mut Channel,
    homeserver: String,
    passphrase: String,
) -> Option<State> {
    match ClientWrapper::new(homeserver, passphrase).await {
        Ok((client, client_session)) => {
            channel.send(Event::Ready).await;
            Some(State::Initialized {
                client,
                client_session,
            })
        }
        Err(error) => {
            channel.send(error).await;
            None
        }
    }
}

pub(self) async fn restore_session(channel: &mut Channel) -> Option<State> {
    match ClientWrapper::restore_session().await {
        Ok(Some(client)) => {
            client.start_sync();

            let user_info = match client.user_info().await {
                Ok(user_info) => user_info,
                Err(error) => {
                    channel.send(error).await;
                    return None;
                }
            };

            channel.send(Event::Authenticated(user_info)).await;
            Some(State::Authenticated(client))
        }
        Ok(None) => {
            channel.send(Event::SessionRestoreFailed).await;
            None
        }
        Err(error) => {
            channel.send(error).await;
            None
        }
    }
}
