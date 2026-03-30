use crate::matrix::bridge::Error;
use crate::matrix::services::TimelineEvent;

use super::Action;
use super::Channel;
use super::ClientWrapper;
use super::Event;
use super::State;

pub async fn handle(
    action: Action,
    channel: &mut Channel,
    client: &mut ClientWrapper,
) -> Option<State> {
    match action {
        Action::CreateMatrixClient { .. }
        | Action::Authenticate { .. }
        | Action::RestoreSession => {
            channel.send(Error::InvalidAction).await;
            None
        }
        Action::ListAllRooms => match client.get_joined_rooms().await {
            Ok(rooms) => {
                channel.send(Event::RoomList(rooms)).await;
                None
            }
            Err(error) => {
                channel.send(error).await;
                None
            }
        },
        Action::GetTimeline(room_id) => {
            let mut timeline_event_rx = match client.room_timeline(room_id.clone()).await {
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
        Action::CloseTimeline(room_id) => {
            client.close_timeline(room_id.clone()).await;
            channel.send(TimelineEvent::Closed(room_id)).await;
            None
        }
        Action::GetDevices => {
            match client.devices().await {
                Ok(devices) => channel.send(Event::DeviceList(devices)).await,
                Err(error) => channel.send(error).await,
            }
            None
        }
    }
}
