use crate::{Error, bridge::action::AuthAction};

use super::{Action, Channel, Event, State, utils};

pub(super) async fn handle_action(
    action: Action,
    channel: &mut Channel<Action, Event>,
) -> Option<State> {
    match action {
        Action::CreateMatrixClient {
            homeserver,
            passphrase,
        } => utils::create_bridge(channel, homeserver, passphrase).await,
        Action::Auth(AuthAction::RestoreSession) => utils::restore_session(channel).await,
        Action::Auth(_) => {
            channel.send(Error::InvalidAction).await;
            None
        }
        _ => {
            channel.send(Error::NotAuthenticated).await;
            None
        }
    }
}
