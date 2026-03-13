use crate::matrix::bridge::Error;
use crate::matrix::session::ClientSession;

use super::Action;
use super::Channel;
use super::ClientWrapper;
use super::Event;
use super::State;

pub(super) async fn handle(
    action: Action,
    channel: &mut Channel,
    client: &mut ClientWrapper,
    client_session: &mut ClientSession,
) -> Option<State> {
    match action {
        Action::CreateMatrixClient {
            homeserver,
            passphrase,
        } => super::create_bridge(channel, homeserver, passphrase).await,
        Action::Authenticate { username, password } => {
            match client
                .authenticate(username, password, client_session.clone())
                .await
            {
                Ok(_) => {
                    channel.send(Event::Syncing).await;
                    match client.sync_once().await {
                        Ok(_) => {
                            client.start_sync();
                            channel.send(Event::Authenticated).await;
                            Some(State::Authenticated(client.clone()))
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
        Action::RestoreSession => {
            super::restore_session(channel).await;
            None
        }
        _ => {
            channel.send(Error::NotAuthenticated).await;
            None
        }
    }
}
