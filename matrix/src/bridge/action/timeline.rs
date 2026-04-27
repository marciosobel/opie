use matrix_sdk::ruma::OwnedRoomId;

/// An action related to room timelines.
#[derive(Debug, Clone)]
pub enum TimelineAction {
    /// Gets the timeline for the given room.
    Get(OwnedRoomId),
    /// Closes the timeline for the given room, aborting the background task that listens for updates.
    Close(OwnedRoomId),
    /// Paginates the timeline backwards for the given room, adding more events to the start of the list.
    PaginateBackwards(OwnedRoomId),
    /// Paginates the timeline forwards for the given room, adding more events to the end of the list.
    PaginateForwards(OwnedRoomId),
}

impl std::fmt::Display for TimelineAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimelineAction::Get(room_id) => {
                write!(f, "Get {{ room_id: {} }}", room_id)
            }
            TimelineAction::Close(room_id) => {
                write!(f, "Close {{ room_id: {} }}", room_id)
            }
            TimelineAction::PaginateBackwards(room_id) => {
                write!(f, "PaginateBackwards {{ room_id: {} }}", room_id)
            }
            TimelineAction::PaginateForwards(room_id) => {
                write!(f, "PaginateForwards {{ room_id: {} }}", room_id)
            }
        }
    }
}
