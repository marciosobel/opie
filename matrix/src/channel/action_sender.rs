use futures::channel::mpsc::Sender;
use std::fmt::Display;

/// A sender for actions. Cloning should not have any noticeable performance impacts.
#[derive(Debug, Clone)]
pub struct ActionSender<T: Clone + Display>(Sender<T>);

impl<T: Clone + Display + Send> ActionSender<T> {
    pub(super) fn new(tx: Sender<T>) -> Self {
        Self(tx)
    }

    /// Sends an `Action` through the channel.
    /// Can be called multiple times, as the actions will be processed in the order they were received.
    pub fn send(&mut self, action: impl Into<T>) -> &mut Self {
        match self.0.try_send(action.into()) {
            Ok(_) => {}
            Err(error) => {
                tracing::error!("Failed to send action: {}", error);
            }
        }
        self
    }
}
