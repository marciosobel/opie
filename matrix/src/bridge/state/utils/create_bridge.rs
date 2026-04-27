use crate::{
    Action, Channel, Event,
    bridge::{Client, State},
};

/// Creates a new client bridge, returning the new state if successful
pub async fn create_bridge(
    channel: &mut Channel<Action, Event>,
    homeserver: String,
    passphrase: String,
) -> Option<State> {
    match Client::new(homeserver, passphrase).await {
        Ok((client, client_session)) => {
            channel.send(Event::Ready).await;
            Some(State::initialized(client, client_session))
        }
        Err(error) => {
            channel.send(error).await;
            None
        }
    }
}
