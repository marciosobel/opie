use crate::{
    Action, Channel, Event,
    bridge::{Client, State},
};

/// Attempts to restore the session, returning the new state if successful
pub async fn restore_session(channel: &mut Channel<Action, Event>) -> Option<State> {
    match Client::restore_session().await {
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
            let sas_verification = super::create_sas_bridge(client.inner(), channel.sender());

            Some(State::authenticated(client, sas_verification))
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
