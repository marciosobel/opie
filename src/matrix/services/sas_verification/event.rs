use super::Error;
use matrix_sdk::{
    Error as MatrixError,
    encryption::verification::{CancelInfo, Emoji},
};

/// An event that occurred in the verification process.
#[derive(Debug, Clone)]
pub enum Event {
    /// Verification is complete and accepted by both parties.
    Done,
    /// Verification has been cancelled.
    Cancelled(CancelInfo),
    /// An error that occurred in the verification process.
    Error(Error),
    /// A new SAS verification flow has been created by us.
    Created,
    /// The SAS verification has been initiated.
    Started,
    /// The emojis to be verified.
    VerifyEmojis([Emoji; 7]),
    /// The verification flow has been accepted by our side.
    Confirmed,
}

impl Into<Event> for Error {
    fn into(self) -> Event {
        Event::Error(self)
    }
}

impl Into<Event> for MatrixError {
    fn into(self) -> Event {
        Event::Error(self.into())
    }
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Done => write!(f, "Done"),
            Event::Cancelled(info) => write!(f, "Cancelled({{ reason: {} }})", info.reason()),
            Event::Error(error) => write!(f, "Error({})", error),
            Event::Started => write!(f, "Started"),
            Event::Confirmed => write!(f, "Confirmed"),
            Event::Created => write!(f, "Created"),
            Event::VerifyEmojis(emojis) => {
                write!(
                    f,
                    "VerifyEmojis([{}])",
                    emojis
                        .iter()
                        .map(|emoji| emoji.symbol)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
}
