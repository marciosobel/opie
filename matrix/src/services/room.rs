use std::collections::{HashMap, HashSet};

use bytes::Bytes;
use futures::StreamExt;
use matrix_sdk::{
    Client, Error, Room as MatrixRoom, RoomMemberships,
    media::{MediaFormat, MediaThumbnailSettings},
    room::ParentSpace,
    ruma::UInt,
};

pub use matrix_sdk::ruma::OwnedRoomId as RoomId;
use tracing::{info, warn};

/// Lists all rooms the user has joined, along with their parents and children relationships.
pub async fn list_joined_rooms(client: Client) -> Result<HashMap<RoomId, Room>, matrix_sdk::Error> {
    info!("Listing joined rooms");
    let joined_rooms = client.joined_rooms();

    let mut rooms = HashMap::new();
    for room in joined_rooms {
        let room = Room::new(room).await?;
        rooms.insert(room.id(), room);
    }

    info!("Client returning {} rooms", rooms.len());
    add_child_to_parents(&mut rooms);
    Ok(rooms)
}

/// Looks for all rooms, takes their parents, and adds the room as a child to the parent.
/// This is necessary because the Matrix SDK only provides parent information, so we need to derive the child information ourselves.
fn add_child_to_parents(rooms: &mut HashMap<RoomId, Room>) {
    info!("Adding child rooms to their parents");

    // A map where it will contain the parent's `OwnedRoomId` related to all it's children.
    let mut child_map: HashMap<RoomId, Vec<RoomId>> = HashMap::new();

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
    id: RoomId,
    display_name: Option<String>,
    children: HashSet<RoomId>,
    parents: HashSet<RoomId>,
    kind: RoomKind,
    avatar: Option<Bytes>,
}

#[derive(Debug, Clone)]
enum RoomKind {
    Direct,
    Space,
    Other,
}

impl Room {
    async fn new(matrix_room: MatrixRoom) -> Result<Self, Error> {
        let id = matrix_room.room_id().to_owned();
        let display_name = matrix_room
            .cached_display_name()
            .map(|display_name| display_name.to_string());

        let kind = if matrix_room.is_space() {
            RoomKind::Space
        } else if matrix_room.is_direct().await? {
            RoomKind::Direct
        } else {
            RoomKind::Other
        };

        let avatar = match kind {
            RoomKind::Direct => get_direct_room_avatar(&matrix_room).await?,
            _ => get_space_room_avatar(&matrix_room).await?,
        };

        let mut parents = HashSet::new();

        if let Ok(mut parent_stream) = matrix_room.parent_spaces().await {
            while let Some(parent_space) = parent_stream.next().await {
                let parent = match parent_space? {
                    ParentSpace::Reciprocal(room) => room,
                    ParentSpace::WithPowerlevel(room) => room,
                    // TODO: Handle these cases properly
                    ParentSpace::Illegitimate(_) => {
                        warn!("Illegitimate parent space, skipping");
                        continue;
                    }
                    ParentSpace::Unverifiable(_) => {
                        warn!("Unverifiable parent space, skipping");
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
            kind,
            avatar,
        })
    }

    /// Returns if the room is a direct message.
    pub fn is_direct(&self) -> bool {
        matches!(self.kind, RoomKind::Direct)
    }

    /// Returns if the room is a space.
    pub fn is_space(&self) -> bool {
        matches!(self.kind, RoomKind::Space)
    }

    /// Returns if the room is a group. A group is a room that has no parents.
    pub fn is_group(&self) -> bool {
        !self.is_space() && !self.is_direct() && self.parents().is_empty()
    }

    /// Returns the display name of the room, if it exists.
    pub fn display_name(&self) -> Option<String> {
        self.display_name.clone()
    }

    /// Returns the parents of the room. If the room has no parents, this will return an empty set.
    pub fn parents(&self) -> &HashSet<RoomId> {
        &self.parents
    }

    /// Returns the children of the room. If the room has no children, this will return an empty set.
    pub fn children(&self) -> &HashSet<RoomId> {
        &self.children
    }

    /// Returns the ID of the room.
    pub fn id(&self) -> RoomId {
        self.id.clone()
    }

    /// Returns the avatar of the room, if it exists.
    pub fn avatar(&self) -> Option<&Bytes> {
        self.avatar.as_ref()
    }
}

async fn get_direct_room_avatar(room: &MatrixRoom) -> Result<Option<Bytes>, Error> {
    let own_id = room.own_user_id();
    let users_in_room = room.members(RoomMemberships::JOIN).await?;
    let Some(other_member) = users_in_room.iter().find(|m| m.user_id() != own_id) else {
        tracing::warn!("Direct room {} has no other members", room.room_id());
        return Ok(None);
    };

    let avatar = match other_member
        .avatar(MediaFormat::Thumbnail(MediaThumbnailSettings::new(
            UInt::new(64).unwrap(),
            UInt::new(64).unwrap(),
        )))
        .await?
    {
        Some(bytes) => Some(Bytes::from_owner(bytes)),
        None => None,
    };

    Ok(avatar)
}

async fn get_space_room_avatar(room: &MatrixRoom) -> Result<Option<Bytes>, Error> {
    let avatar = match room
        .avatar(MediaFormat::Thumbnail(MediaThumbnailSettings::new(
            UInt::new(64).unwrap(),
            UInt::new(64).unwrap(),
        )))
        .await?
    {
        Some(bytes) => Some(Bytes::from_owner(bytes)),
        None => None,
    };

    Ok(avatar)
}

impl PartialEq for Room {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Room {}

impl PartialOrd for Room {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let our_name = self.display_name()?.to_lowercase();
        let their_name = other.display_name()?.to_lowercase();
        our_name.partial_cmp(&their_name)
    }
}

impl Ord for Room {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(&other)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}
