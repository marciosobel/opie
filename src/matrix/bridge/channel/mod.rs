use futures::StreamExt;
use futures::channel::mpsc::{Receiver, Sender, channel};
use std::fmt::Display;

mod action_sender;
pub use action_sender::ActionSender;

mod event_sender;
pub use event_sender::EventSender;

pub const CHANNEL_SIZE: usize = 64;

/// A channel to handle sending events and receiveing actions
#[derive(Debug)]
pub struct Channel<Action, Event>
where
    Action: Clone + Display + Send,
    Event: Clone + Display,
{
    receiver: Receiver<Action>,
    sender: EventSender<Event>,
}

impl<Action, Event> Channel<Action, Event>
where
    Action: Clone + Display + Send,
    Event: Clone + Display,
{
    pub fn new() -> (Receiver<Event>, ActionSender<Action>, Self) {
        let (event_tx, event_rx) = channel(CHANNEL_SIZE);
        let (action_tx, channel) = Self::with_tx(event_tx);
        (event_rx, action_tx, channel)
    }

    /// Creates a new [`Channel`](Channel) with the provided `Event` transmitter
    pub fn with_tx(event_tx: Sender<Event>) -> (ActionSender<Action>, Self) {
        let (action_tx, action_rx) = channel(CHANNEL_SIZE);

        (
            ActionSender::new(action_tx),
            Self {
                sender: EventSender::new(event_tx),
                receiver: action_rx,
            },
        )
    }

    /// Sends the `Event` through the channel.
    pub async fn send(&mut self, event: impl Into<Event>) {
        self.sender.send(event).await
    }

    /// Receives the next `Action`. This will wait until an action is received, and then return it.
    pub async fn recv(&mut self) -> Action {
        self.receiver.select_next_some().await
    }

    /// Returns a copy of the [`EventSender`](EventSender).
    pub fn sender(&self) -> EventSender<Event> {
        self.sender.clone()
    }
}
