use crate::{
    Action, Channel, Event, bridge::Client, services::sas_verification, session::ClientSession,
};

mod authenticated;
mod initialized;
mod waiting_for_client;

pub(self) mod utils;

use authenticated::AuthenticatedState;
use initialized::InitializedState;

/// Current state of the Matrix bridge.
pub(crate) enum State {
    /// The bridge channel has been created, but is waiting for the client to be created.
    WaitingForClient,
    /// The bridge has been initialized and is ready to use.
    Initialized(InitializedState),
    /// The user is authenticated and can use the bridge freely.
    Authenticated(AuthenticatedState),
}

impl State {
    pub(crate) fn new() -> Self {
        Self::WaitingForClient
    }

    fn authenticated(client: Client, sas_verification: sas_verification::Bridge) -> Self {
        Self::Authenticated(AuthenticatedState::new(client, sas_verification))
    }

    fn initialized(client: Client, session: ClientSession) -> Self {
        Self::Initialized(InitializedState::new(client, session))
    }

    /// Handles the action for the current state. Note that, dependending on the action, the state may change.
    pub(crate) async fn handle_action(
        &mut self,
        action: Action,
        channel: &mut Channel<Action, Event>,
    ) {
        let maybe_self = match self {
            State::WaitingForClient => waiting_for_client::handle_action(action, channel).await,
            State::Initialized(state) => state.handle_action(action, channel).await,
            State::Authenticated(state) => state.handle_action(action, channel).await,
        };

        if let Some(new_state) = maybe_self {
            *self = new_state;
        }
    }
}
