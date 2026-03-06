use std::ops::{Deref, DerefMut};

use matrix_sdk::{Client, Room as MatrixRoom};

#[derive(Debug, Clone)]
pub struct Room {
    inner: MatrixRoom,
    pub is_dm: bool,
    pub display_name: Option<String>,
    pub raw_name: Option<String>,
}

pub async fn list_joined_rooms(client: Client) -> Vec<Room> {
    let joined_rooms = client.joined_rooms();
    let mut rooms = vec![];
    for room in joined_rooms {
        rooms.push(Room::new(room).await);
    }
    rooms
}

impl Room {
    pub async fn new(inner: MatrixRoom) -> Self {
        let is_dm = inner.is_direct().await.unwrap_or(false);
        let raw_name = inner.name();
        let display_name = inner.cached_display_name().map(|name| name.to_string());
        Self {
            inner,
            is_dm,
            raw_name,
            display_name,
        }
    }
}

impl Deref for Room {
    type Target = MatrixRoom;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Room {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
