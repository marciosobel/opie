use futures::channel::mpsc::{Receiver, Sender, channel};
use futures::{SinkExt, StreamExt};

use super::Action;
use super::Event;

pub const CHANNEL_SIZE: usize = 64;

/// A channel to handle sending events and receiveing actions
#[derive(Debug)]
pub struct Channel {
    receiver: Receiver<Action>,
    sender: EventSender,
}

/// A sender for actions to the Matrix bridge. This can be used to send actions to the bridge from outside of the channel.
/// Cloning this struct is OK and should not have any noticeable performance impacts.
#[derive(Debug, Clone)]
pub struct ActionSender(Sender<Action>);

/// A sender for Matrix events.
#[derive(Debug, Clone)]
pub struct EventSender(Sender<Event>);

impl Channel {
    pub(super) fn new(event_tx: Sender<Event>) -> (ActionSender, Self) {
        let (action_tx, action_rx) = channel(CHANNEL_SIZE);

        (
            ActionSender::new(action_tx),
            Self {
                sender: EventSender::new(event_tx),
                receiver: action_rx,
            },
        )
    }

    /// Sends an [`Event`](super::Event) through the channel.
    pub(super) async fn send(&mut self, event: impl Into<Event>) {
        self.sender.send(event).await
    }

    /// Polls the channel for the next [`Action`](super::Action). This will wait until an action is received, and then return it.
    pub(super) async fn next_action(&mut self) -> Action {
        self.receiver.select_next_some().await
    }

    /// Returns a copy of the [`Event`](super::Event) [`Sender`](Sender).
    pub(super) fn sender(&self) -> EventSender {
        self.sender.clone()
    }
}

impl ActionSender {
    pub(super) fn new(tx: Sender<Action>) -> Self {
        Self(tx)
    }

    /// Sends an action to the Matrix [`Bridge`](matrix::bridge::Bridge).
    /// Can be called multiple times, as the actions will be processed in the order they were received.
    pub fn send(&mut self, message: impl Into<Action>) -> &mut Self {
        match self.0.try_send(message.into()) {
            Ok(_) => {}
            Err(error) => {
                tracing::error!("Failed to send message to matrix event channel: {}", error);
            }
        }
        self
    }
}

impl EventSender {
    fn new(tx: Sender<Event>) -> Self {
        Self(tx)
    }

    /// Sends an [`Event`](super::Event) through the channel.
    pub(super) async fn send(&mut self, event: impl Into<Event>) {
        let event = event.into();
        tracing::info!("Sending event: {}", event);
        match self.0.send(event).await {
            Ok(_) => (),
            Err(error) => tracing::error!("Failed to send event: {:?}", error),
        }
    }
}
