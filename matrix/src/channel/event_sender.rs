use futures::{SinkExt, channel::mpsc::Sender};
use std::fmt::Display;

/// A connection for receiveing events from the Matrix backend.
#[derive(Debug, Clone)]
pub struct EventSender<T: Clone + Display>(Sender<T>);

impl<T: Clone + Display> EventSender<T> {
    pub fn new(tx: Sender<T>) -> Self {
        Self(tx)
    }

    /// Sends an `Event` through the channel.
    pub async fn send(&mut self, event: impl Into<T>) {
        let event = event.into();
        tracing::info!("Sending event: {}", event);
        match self.0.send(event).await {
            Ok(_) => (),
            Err(error) => tracing::error!("Failed to send event: {:?}", error),
        }
    }
}
