use std::sync::Arc;

use anyhow::Result;
use futures::StreamExt;
use matrix_sdk::{Room, ruma::OwnedRoomId};
use matrix_sdk_ui::timeline::{self, RoomExt, Timeline as MatrixTimeline};
use tokio::{sync::mpsc, task::JoinHandle};

pub type TimelineDiff = VectorDiff<Arc<TimelineItem>>;
pub use matrix_sdk::ruma::events::room::message::MessageType;
pub use matrix_sdk_ui::{
    eyeball_im::{Vector, VectorDiff},
    timeline::{Message, MsgLikeKind, TimelineItem, TimelineItemContent},
};

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

            let diffs = diffs
                .into_iter()
                .map(|d| {
                    d.map(|item| {
                        // TODO: gather info about user message, message kind and data.
                        item
                    })
                })
                .collect();

            match tx.send(TimelineEvent::Updated(room_id, diffs)).await {
                Ok(_) => (),
                Err(error) => {
                    tracing::error!("Failed to send update event: {}", error);
                }
            };
        }
    });

    tokio::spawn(async move {
        if let Err(error) = timeline.paginate_backwards(100).await {
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
        }
    }
}
