use bytes::Bytes;

use super::State;
use crate::{
    Action, Channel, Error, Event,
    bridge::{
        self, Client,
        action::{MediaAction, TimelineAction},
    },
    services::{TimelineEvent, sas_verification},
};

/// Struct for the `authenticated` state of the bridge.
pub(crate) struct AuthenticatedState {
    /// The client facade.
    client: Client,
    /// The SAS Auth verification bridge.
    sas_verification: sas_verification::Bridge,
}

impl std::ops::Deref for AuthenticatedState {
    type Target = Client;
    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl std::ops::DerefMut for AuthenticatedState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}

impl AuthenticatedState {
    pub fn new(client: Client, sas_verification: sas_verification::Bridge) -> Self {
        Self {
            client,
            sas_verification,
        }
    }

    pub async fn handle_action(
        &mut self,
        action: Action,
        channel: &mut Channel<Action, Event>,
    ) -> Option<State> {
        match action {
            Action::CreateMatrixClient { .. } | Action::Auth(_) => {
                channel.send(Error::InvalidAction).await;
                None
            }
            Action::SasVerification(action) => {
                self.sas_verification.send(action);
                None
            }
            Action::ListAllRooms => {
                let sender = channel.sender();
                let client = self.client.clone();
                tokio::spawn(async move {
                    let mut tx = sender;
                    match client.get_joined_rooms().await {
                        Ok(rooms) => {
                            tx.send(Event::RoomList(rooms)).await;
                        }
                        Err(error) => {
                            tx.send(error).await;
                        }
                    }
                });
                None
            }
            Action::Timeline(action) => self.handle_timeline_action(action, channel).await,
            Action::GetDevices => {
                match self.client.devices().await {
                    Ok(devices) => channel.send(Event::DeviceList(devices)).await,
                    Err(error) => channel.send(error).await,
                }
                None
            }
            Action::Media(action) => self.handle_media_action(action, channel).await,
        }
    }

    async fn handle_media_action(
        &mut self,
        action: bridge::action::MediaAction,
        channel: &mut Channel<Action, Event>,
    ) -> Option<State> {
        match action {
            MediaAction::FetchUserAvatar(user_id, uri) => match self.fetch_user_avatar(uri).await {
                Ok(data) => {
                    let mut tx = channel.sender();
                    tokio::spawn(async move {
                        let bytes = Bytes::copy_from_slice(&data);
                        tx.send(Event::UserAvatarFetched(user_id, bytes)).await
                    });
                }
                Err(error) => channel.send(error).await,
            },
        }
        None
    }

    async fn handle_timeline_action(
        &mut self,
        action: bridge::action::TimelineAction,
        channel: &mut Channel<Action, Event>,
    ) -> Option<State> {
        match action {
            TimelineAction::Get(room_id) => {
                let mut timeline_event_rx = match self.room_timeline(room_id.clone()).await {
                    Ok(tuple) => tuple,
                    Err(error) => {
                        channel.send(error).await;
                        return None;
                    }
                };

                let mut tx = channel.sender();
                tokio::spawn(async move {
                    while let Some(timeline_event) = timeline_event_rx.recv().await {
                        tx.send(timeline_event).await;
                    }
                });

                None
            }
            TimelineAction::Close(room_id) => {
                self.close_timeline(room_id.clone()).await;
                channel.send(TimelineEvent::Closed(room_id)).await;
                None
            }
            TimelineAction::PaginateBackwards(room_id) => {
                match self.paginate_timeline_backwards(room_id.clone()).await {
                    Ok(true) => channel.send(TimelineEvent::Start(room_id)).await,
                    Err(error) => channel.send(error).await,
                    _ => {}
                };
                None
            }
            TimelineAction::PaginateForwards(room_id) => {
                match self.paginate_timeline_forwards(room_id.clone()).await {
                    Ok(true) => channel.send(TimelineEvent::End(room_id)).await,
                    Err(error) => channel.send(error).await,
                    _ => {}
                };
                None
            }
            TimelineAction::SendMessage(room_id, content) => {
                if let Err(error) = self.send_message(room_id, content).await {
                    channel.send(error).await;
                }
                None
            }
        }
    }
}
