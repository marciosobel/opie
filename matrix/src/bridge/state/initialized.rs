use crate::bridge::Client;
use crate::bridge::action::AuthAction;

use crate::{
    Action, Channel, Error, Event,
    bridge::state::{State, utils},
    session::ClientSession,
};

/// Struct for the `initialized` state of the bridge.
pub(crate) struct InitializedState {
    /// The client facade.
    client: Client,
    /// The current session of the client.
    session: ClientSession,
}

impl std::ops::Deref for InitializedState {
    type Target = Client;
    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl std::ops::DerefMut for InitializedState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

impl InitializedState {
    pub fn new(client: Client, session: ClientSession) -> Self {
        Self { client, session }
    }

    pub async fn handle_action(
        &mut self,
        action: Action,
        channel: &mut Channel<Action, Event>,
    ) -> Option<State> {
        match action {
            Action::CreateMatrixClient {
                homeserver,
                passphrase,
            } => utils::create_bridge(channel, homeserver, passphrase).await,
            Action::Auth(action) => match action {
                AuthAction::Authenticate { username, password } => {
                    match self
                        .client
                        .authenticate(username, password, self.session.clone())
                        .await
                    {
                        Ok(_) => {
                            channel.send(Event::Syncing).await;
                            match self.client.sync_once().await {
                                Ok(_) => {
                                    self.client.start_sync();

                                    let user_info = match self.client.user_info().await {
                                        Ok(user_info) => user_info,
                                        Err(error) => {
                                            channel.send(error).await;
                                            return None;
                                        }
                                    };

                                    channel.send(Event::Authenticated(user_info)).await;
                                    let sas_verification = utils::create_sas_bridge(
                                        self.client.inner(),
                                        channel.sender(),
                                    );

                                    Some(State::authenticated(
                                        self.client.clone(),
                                        sas_verification,
                                    ))
                                }
                                Err(error) => {
                                    channel.send(error).await;
                                    None
                                }
                            }
                        }
                        Err(error) => {
                            channel.send(error).await;
                            None
                        }
                    }
                }
                AuthAction::RestoreSession => {
                    utils::restore_session(channel).await;
                    None
                }
            },
            _ => {
                channel.send(Error::NotAuthenticated).await;
                None
            }
        }
    }
}
