use std::{pin::pin, sync::Arc};

use iced::futures::StreamExt;
use matrix_sdk_ui::{
    eyeball_im::Vector,
    room_list_service::{RoomList, RoomListItem, filters::new_filter_non_left},
};

pub type Rooms = Arc<Vector<RoomListItem>>;

pub async fn list_rooms(room_list: &RoomList) -> Rooms {
    tracing::info!("Starting to list rooms");
    let (stream, entries_controller) = room_list.entries_with_dynamic_adapters(50_000);
    entries_controller.set_filter(Box::new(new_filter_non_left()));
    let mut stream = pin!(stream);

    let mut rooms = Vector::new();

    let Some(diffs) = stream.next().await else {
        tracing::warn!("No rooms found!");
        return Arc::new(rooms);
    };

    for diff in diffs {
        diff.apply(&mut rooms);
    }

    Arc::new(rooms)
}
