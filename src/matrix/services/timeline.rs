use std::sync::Arc;

use anyhow::Result;
use futures::StreamExt;
use matrix_sdk::{Room, ruma::OwnedRoomId};
use matrix_sdk_ui::{
    eyeball_im::{Vector, VectorDiff},
    timeline::{self, RoomExt, Timeline as MatrixTimeline, TimelineItem},
};
use tokio::{sync::mpsc, task::JoinHandle};

pub type TimelineDiff = VectorDiff<Arc<TimelineItem>>;

/// Create a new timeline for the given room. This will subscribe to updates and paginate backwards to load the initial items.
pub async fn timeline(
    room: Room,
) -> Result<(Timeline, mpsc::Receiver<TimelineUpdateEvent>), timeline::Error> {
    let room_id = room.room_id().to_owned();
    let timeline = Arc::new(room.timeline().await?);
    let inner = timeline.clone();
    let (tx, rx) = mpsc::channel(8);

    let (items, mut stream) = timeline.subscribe().await;

    // Send the initial items as an update event
    _ = tx
        .send(TimelineUpdateEvent::Initial(room_id.clone(), items))
        .await;

    let tx_clone = tx.clone();
    let room_id_clone = room_id.clone();
    let task = tokio::spawn(async move {
        while let Some(diffs) = stream.next().await {
            let room_id = room_id_clone.clone();
            tracing::info!("Received {} diffs", diffs.len());
            match tx_clone
                .send(TimelineUpdateEvent::Updated(room_id, diffs))
                .await
            {
                Ok(_) => (),
                Err(error) => {
                    tracing::error!("Failed to send update event: {}", error);
                }
            };
        }
    });

    tokio::spawn(async move {
        match timeline.paginate_backwards(40).await {
            Ok(_) => (),
            Err(error) => {
                tracing::error!("Failed to paginate backwards: {}", error);
            }
        };
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
    tx: mpsc::Sender<TimelineUpdateEvent>,
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
        tracing::info!("Closing timeline for room {}", self.room_id());
        self.task.abort();
        _ = self
            .tx
            .send(TimelineUpdateEvent::Closed(self.room_id.clone()))
            .await;
    }
}

impl Drop for Timeline {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/// An event representing an update to the timeline.
#[derive(Debug, Clone)]
pub enum TimelineUpdateEvent {
    /// The timeline was initialized with the given items.
    Initial(OwnedRoomId, Vector<Arc<TimelineItem>>),
    /// The timeline was updated with the given diffs.
    Updated(OwnedRoomId, Vec<TimelineDiff>),
    /// The timeline was closed and will no longer receive updates.
    Closed(OwnedRoomId),
}

impl std::fmt::Display for TimelineUpdateEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimelineUpdateEvent::Initial(room_id, generic_vector) => {
                write!(
                    f,
                    "TimelineUpdateEvent::Initial({}, {} items)",
                    room_id,
                    generic_vector.len()
                )
            }
            TimelineUpdateEvent::Updated(room_id, diffs) => {
                write!(
                    f,
                    "TimelineUpdateEvent::Updated({}, {} diffs)",
                    room_id,
                    diffs.len()
                )
            }
            TimelineUpdateEvent::Closed(room_id) => {
                write!(f, "TimelineUpdateEvent::Closed({})", room_id,)
            }
        }
    }
}
