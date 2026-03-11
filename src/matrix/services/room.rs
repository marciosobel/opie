use std::collections::{HashMap, HashSet};

use bytes::Bytes;
use futures::StreamExt;
use matrix_sdk::{
    Client, Error, Room as MatrixRoom, media::MediaFormat, room::ParentSpace, ruma::OwnedRoomId,
};

/// Lists all rooms the user has joined, along with their parents and children relationships.
pub async fn list_joined_rooms(
    client: Client,
) -> Result<HashMap<OwnedRoomId, Room>, matrix_sdk::Error> {
    tracing::info!("Listing joined rooms");
    let joined_rooms = client.joined_rooms();

    let mut rooms = HashMap::new();
    for room in joined_rooms {
        let room = Room::new(room).await?;
        rooms.insert(room.id(), room);
    }

    tracing::info!("Client returning {} rooms", rooms.len());
    add_child_to_parents(&mut rooms);
    Ok(rooms)
}

/// Looks for all rooms, takes their parents, and adds the room as a child to the parent.
/// This is necessary because the Matrix SDK only provides parent information, so we need to derive the child information ourselves.
fn add_child_to_parents(rooms: &mut HashMap<OwnedRoomId, Room>) {
    tracing::info!("Adding child rooms to their parents");

    // A map where it will contain the parent's `OwnedRoomId` related to all it's children.
    let mut child_map: HashMap<OwnedRoomId, Vec<OwnedRoomId>> = HashMap::new();

    // Populate the map
    for (_, room) in rooms.iter_mut() {
        for parent_id in &room.parents {
            child_map
                .entry(parent_id.clone())
                .or_default()
                .push(room.id.clone());
        }
    }

    // Get the children and append them to the parent.
    for (parent_id, parent) in rooms {
        if let Some(child_ids) = child_map.get(parent_id) {
            parent.children.extend(child_ids.iter().cloned());
        }
    }
}

#[derive(Debug, Clone)]
pub struct Room {
    id: OwnedRoomId,
    display_name: Option<String>,
    children: HashSet<OwnedRoomId>,
    parents: HashSet<OwnedRoomId>,
    is_direct: bool,
    is_space: bool,
    avatar: Option<Bytes>,
}

impl Room {
    async fn new(matrix_room: MatrixRoom) -> Result<Self, Error> {
        let id = matrix_room.room_id().to_owned();
        let display_name = matrix_room
            .cached_display_name()
            .map(|display_name| display_name.to_string());
        let is_direct = matrix_room.is_direct().await?;
        let is_space = matrix_room.is_space();
        let avatar = match matrix_room.avatar(MediaFormat::File).await? {
            Some(bytes) => Some(Bytes::from_owner(bytes)),
            None => None,
        };

        let mut parents = HashSet::new();

        if let Ok(mut parent_stream) = matrix_room.parent_spaces().await {
            while let Some(parent_space) = parent_stream.next().await {
                let parent = match parent_space? {
                    ParentSpace::Reciprocal(room) => room,
                    ParentSpace::WithPowerlevel(room) => room,
                    // TODO: Handle these cases properly
                    ParentSpace::Illegitimate(_) => {
                        tracing::warn!("Illegitimate parent space, skipping");
                        continue;
                    }
                    ParentSpace::Unverifiable(_) => {
                        tracing::warn!("Unverifiable parent space, skipping");
                        continue;
                    }
                };

                let parent_id = parent.room_id().to_owned();
                parents.insert(parent_id);
            }
        } else {
            tracing::error!("Failed to get parents");
        }

        Ok(Self {
            id,
            display_name,
            children: HashSet::new(),
            parents,
            is_direct,
            is_space,
            avatar,
        })
    }

    /// Returns if the room is a direct message.
    pub fn is_direct(&self) -> bool {
        self.is_direct
    }

    /// Returns if the room is a space.
    pub fn is_space(&self) -> bool {
        self.is_space
    }

    /// Returns the display name of the room, if it exists.
    pub fn display_name(&self) -> Option<String> {
        self.display_name.clone()
    }

    /// Returns the parents of the room. If the room has no parents, this will return an empty set.
    pub fn parents(&self) -> &HashSet<OwnedRoomId> {
        &self.parents
    }

    /// Returns the children of the room. If the room has no children, this will return an empty set.
    pub fn children(&self) -> &HashSet<OwnedRoomId> {
        &self.children
    }

    /// Returns the ID of the room.
    pub fn id(&self) -> OwnedRoomId {
        self.id.clone()
    }

    /// Returns the avatar of the room, if it exists.
    pub fn avatar(&self) -> Option<&Bytes> {
        self.avatar.as_ref()
    }
}

impl PartialEq for Room {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Room {}
