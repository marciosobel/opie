use super::timeline::Timeline;
use matrix::services::room::RoomId;

#[derive(Debug, Clone)]
pub struct FocusedRoom {
    pub id: RoomId,
    pub message_draft: String,
    pub timeline: Timeline,
}

impl FocusedRoom {
    pub fn new(room_id: RoomId) -> Self {
        Self {
            id: room_id,
            message_draft: String::new(),
            timeline: Timeline::new(),
        }
    }

    pub fn with_timeline(room_id: RoomId, timeline: Timeline) -> Self {
        Self {
            id: room_id,
            message_draft: String::new(),
            timeline,
        }
    }
}

impl PartialEq for FocusedRoom {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for FocusedRoom {}
