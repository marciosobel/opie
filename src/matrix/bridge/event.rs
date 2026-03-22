use std::collections::HashMap;
use std::sync::Arc;

use crate::matrix::bridge::channel::ActionSender;
use crate::matrix::services::{Room, TimelineEvent, UserInfo};

use super::Error;
use matrix_sdk::ruma::OwnedRoomId;

/// Events emitted by the Matrix bridge.
#[derive(Debug, Clone)]
pub enum Event {
    /// The Matrix bridge has been initialized and is waiting for either [`Action::CreateMatrixClient(homeserver)`](Action::CreateMatrixClient)
    /// or [`Action::RestoreSession`](Action::RestoreSession) to proceed.
    Stale(ActionSender),
    /// The Matrix client has been built and is ready to use.
    Ready,
    /// An error that occurred in the Matrix bridge.
    Error(Error),
    /// The user has been authenticated with the Matrix server.
    Authenticated(UserInfo),
    /// A session restore was attempted, but no session was found on disk or it was expired.
    SessionRestoreFailed,
    /// The client is syncronizing with the server, which may take some time. The client is not ready to use until the sync is complete.
    Syncing,
    /// A list of rooms that the user is a member of.
    RoomList(HashMap<OwnedRoomId, Arc<Room>>),
    /// The timeline for a room has been updated with new events or changes to existing events. The diff contains the changes that were made to the timeline.
    TimelineEvent(TimelineEvent),
}

impl From<Error> for Event {
    fn from(value: Error) -> Self {
        Event::Error(value)
    }
}

impl From<TimelineEvent> for Event {
    fn from(value: TimelineEvent) -> Self {
        Event::TimelineEvent(value)
    }
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Stale(_) => write!(f, "Stale"),
            Event::Ready => write!(f, "Ready"),
            Event::Error(error) => write!(f, "Error({})", error),
            Event::Authenticated(user) => write!(
                f,
                "Authenticated as {}",
                user.display_name().unwrap_or("unknown".into())
            ),
            Event::SessionRestoreFailed => write!(f, "SessionRestoreFailed"),
            Event::RoomList(rooms) => {
                write!(f, "RoomList({} rooms)", rooms.len())
            }
            Event::Syncing => write!(f, "Syncing"),
            Event::TimelineEvent(event) => write!(f, "TimelineEvent({})", event),
        }
    }
}
