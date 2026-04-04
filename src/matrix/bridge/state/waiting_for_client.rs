use crate::matrix::bridge::Error;

use super::{Action, Channel, Event, State};

pub(super) async fn handle(action: Action, channel: &mut Channel<Action, Event>) -> Option<State> {
    match action {
        Action::CreateMatrixClient {
            homeserver,
            passphrase,
        } => super::create_bridge(channel, homeserver, passphrase).await,
        Action::RestoreSession => super::restore_session(channel).await,
        Action::Authenticate { .. } => {
            channel.send(Error::InvalidAction).await;
            None
        }
        _ => {
            channel.send(Error::NotAuthenticated).await;
            None
        }
    }
}
