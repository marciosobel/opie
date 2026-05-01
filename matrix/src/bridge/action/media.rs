pub use matrix_sdk::ruma::{
    OwnedEventId as EventId, OwnedMxcUri as MxcUri, events::room::MediaSource,
};

/// An action related to users.
#[derive(Debug, Clone)]
pub enum MediaAction {
    /// Fetches the image of the provided timeline [`EventId`].
    FetchTimelineImage(EventId, MediaSource),
}

impl std::fmt::Display for MediaAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaAction::FetchTimelineImage(event_id, _) => {
                write!(f, "FetchTimelineImage {{ event_id: {}, .. }}", event_id)
            }
        }
    }
}
