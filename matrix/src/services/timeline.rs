use std::sync::Arc;

use anyhow::Result;
use futures::StreamExt;
use matrix_sdk::{
    Room,
    ruma::{
        OwnedRoomId,
        events::{AnyMessageLikeEventContent, room::message::RoomMessageEventContent},
    },
};
use matrix_sdk_ui::timeline::{self, RoomExt, Timeline as MatrixTimeline};
use tokio::{sync::mpsc, task::JoinHandle};

pub type TimelineDiff = VectorDiff<Arc<TimelineItem>>;
pub use matrix_sdk::ruma::events::room::message::MessageType;
pub use matrix_sdk_ui::{
    eyeball_im::{Vector, VectorDiff},
    timeline::{
        EventTimelineItem, Message, MsgLikeKind, TimelineItem, TimelineItemContent,
        TimelineItemKind, VirtualTimelineItem,
    },
};

pub const PAGINATION_SIZE: u16 = 100;

/// Create a new timeline for the given room. This will subscribe to updates and paginate backwards to load the initial items.
pub async fn timeline(
    room: Room,
) -> Result<(Timeline, mpsc::Receiver<TimelineEvent>), timeline::Error> {
    let room_id = room.room_id().to_owned();
    let timeline = Arc::new(room.timeline().await?);
    let inner = timeline.clone();
    let (tx, rx) = mpsc::channel(8);

    let (items, mut stream) = timeline.subscribe().await;

    // Send the initial items as an update event
    _ = tx
        .send(TimelineEvent::Initial(room_id.clone(), items))
        .await;

    let tx_clone = tx.clone();
    let room_id_clone = room_id.clone();
    let task = tokio::spawn(async move {
        let tx = tx_clone;
        let room_id = room_id_clone;

        while let Some(diffs) = stream.next().await {
            let room_id = room_id.clone();
            tracing::info!("Received {} diffs", diffs.len());

            if let Err(error) = tx.send(TimelineEvent::Updated(room_id, diffs)).await {
                tracing::error!("Failed to send update event: {}", error);
            };
        }
    });

    tokio::spawn(async move {
        if let Err(error) = timeline.paginate_backwards(PAGINATION_SIZE).await {
            tracing::error!("Failed to paginate backwards: {}", error);
        }
    });

    Ok((
        Timeline {
            tx,
            task,
            inner,
            room_id,
        },
        rx,
    ))
}

pub struct Timeline {
    tx: mpsc::Sender<TimelineEvent>,
    room_id: OwnedRoomId,
    inner: Arc<MatrixTimeline>,
    task: JoinHandle<()>,
}

impl Timeline {
    pub fn room_id(&self) -> &OwnedRoomId {
        &self.room_id
    }

    pub fn inner(&self) -> Arc<MatrixTimeline> {
        self.inner.clone()
    }

    pub async fn close(&self) {
        self.task.abort();
        _ = self
            .tx
            .send(TimelineEvent::Closed(self.room_id.clone()))
            .await;
    }

    /// Paginates the timeline backwards, adding more events to the start of the timeline.
    ///
    /// Returns `true` if we hit the start of the timeline.
    pub async fn paginate_backwards(&self) -> Result<bool, timeline::Error> {
        self.inner.paginate_backwards(PAGINATION_SIZE).await
    }

    /// Paginates the timeline forwards, adding more events to the end of the timeline.
    ///
    /// Returns `true` if we hit the end of the timeline.
    pub async fn paginate_forwards(&self) -> Result<bool, timeline::Error> {
        self.inner.paginate_forwards(PAGINATION_SIZE).await
    }

    pub async fn send_message(&self, content: String) -> Result<(), timeline::Error> {
        self.inner
            .send(AnyMessageLikeEventContent::RoomMessage(
                RoomMessageEventContent::text_plain(content),
            ))
            .await?;
        Ok(())
    }
}

impl Drop for Timeline {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// A room's timeline event.
#[derive(Debug, Clone)]
pub enum TimelineEvent {
    /// The timeline was initialized with the given items.
    Initial(OwnedRoomId, Vector<Arc<TimelineItem>>),
    /// The timeline was updated with the given diffs.
    Updated(OwnedRoomId, Vec<TimelineDiff>),
    /// The timeline was closed and will no longer receive updates.
    Closed(OwnedRoomId),
    /// The timeline hit the start.
    Start(OwnedRoomId),
    /// The timeline hit the end.
    End(OwnedRoomId),
}

impl std::fmt::Display for TimelineEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimelineEvent::Initial(room_id, generic_vector) => {
                write!(
                    f,
                    "TimelineEvent::Initial({}, {} items)",
                    room_id,
                    generic_vector.len()
                )
            }
            TimelineEvent::Updated(room_id, diffs) => {
                write!(
                    f,
                    "TimelineEvent::Updated({}, {} diffs)",
                    room_id,
                    diffs.len()
                )
            }
            TimelineEvent::Closed(room_id) => {
                write!(f, "TimelineEvent::Closed({})", room_id,)
            }
            TimelineEvent::Start(room_id) => write!(f, "TimelineEvent::Start({})", room_id),
            TimelineEvent::End(room_id) => write!(f, "TimelineEvent::End({})", room_id),
        }
    }
}
