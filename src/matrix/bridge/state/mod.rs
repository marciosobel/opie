use futures::StreamExt;
use matrix_sdk::{
    Client,
    ruma::events::{
        key::verification::request::ToDeviceKeyVerificationRequestEvent,
        room::message::{MessageType, OriginalSyncRoomMessageEvent},
    },
};

use crate::matrix::{bridge::EventSender, services::sas_verification, session::ClientSession};

use super::{Action, Channel, ClientWrapper, Event};

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
    Authenticated {
        client: ClientWrapper,
        sas_verification: sas_verification::Bridge,
    },
}

impl State {
    pub(super) fn new() -> Self {
        Self::WaitingForClient
    }

    /// Handles the action for the current state. Note that, dependending on the action, the state may change.
    pub(super) async fn handle_action(
        &mut self,
        action: Action,
        channel: &mut Channel<Action, Event>,
    ) {
        let maybe_self = match self {
            State::WaitingForClient => waiting_for_client::handle(action, channel).await,
            State::Initialized {
                client,
                client_session,
            } => initialized::handle(action, channel, client, client_session).await,
            State::Authenticated {
                client,
                sas_verification,
            } => authenticated::handle(action, channel, client, sas_verification).await,
        };

        match maybe_self {
            Some(new_state) => *self = new_state,
            None => {}
        }
    }
}

/// Creates a new client bridge, returning the new state if successful
pub(self) async fn create_bridge(
    channel: &mut Channel<Action, Event>,
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

/// Attempts to restore the session, returning the new state if successful
pub(self) async fn restore_session(channel: &mut Channel<Action, Event>) -> Option<State> {
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
            let sas_verification = create_sas_bridge(client.inner(), channel.sender());

            Some(State::Authenticated {
                client,
                sas_verification,
            })
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

/// Creates a new bridge with the sas verification service
pub(self) fn create_sas_bridge(
    client: Client,
    mut matrix_event_tx: EventSender<Event>,
) -> sas_verification::Bridge {
    let listener = sas_verification::Listener::new(client.clone());

    let tx = listener.event_tx.clone();
    let sas = listener.sas.clone();
    client.add_event_handler(
        |ev: ToDeviceKeyVerificationRequestEvent, client: Client| async move {
            let request = client
                .encryption()
                .get_verification_request(&ev.sender, &ev.content.transaction_id)
                .await
                .expect("Request object wasn't created");

            tokio::spawn(sas_verification::request_verification(request, tx, sas));
        },
    );

    let tx = listener.event_tx.clone();
    let sas = listener.sas.clone();
    client.add_event_handler(
        |ev: OriginalSyncRoomMessageEvent, client: Client| async move {
            if let MessageType::VerificationRequest(_) = &ev.content.msgtype {
                let request = client
                    .encryption()
                    .get_verification_request(&ev.sender, &ev.event_id)
                    .await
                    .expect("Request object wasn't created");

                tokio::spawn(sas_verification::request_verification(request, tx, sas));
            }
        },
    );

    let mut rx = listener.event_rx;
    tokio::spawn(async move {
        while let Some(event) = rx.next().await {
            matrix_event_tx.send(event).await;
        }
    });

    listener.action_tx
}
