use matrix::services::sas_verification::{CancelInfo, Emoji, Error, Event as SasVerificationEvent};

/// The state of the ongoing verification state, if any.
#[derive(Debug, Clone)]
pub enum VerificationState {
    /// No verification is currently running.
    Stale,
    /// A verification is currently running.
    Ongoing,
    /// Verification has been cancelled.
    Cancelled(CancelInfo),
    /// Verification has been completed.
    Done,
    /// Verification is waiting for the confirmation if the emojis match.
    Emoji([Emoji; 7]),
    /// Something went wrong in the verification flow.
    Errored(Error),
    /// The verification has been accepted by our side.
    Confirmed,
    /// A SAS verification flow has been created by us.
    Created,
}

impl From<SasVerificationEvent> for VerificationState {
    fn from(value: SasVerificationEvent) -> Self {
        match value {
            SasVerificationEvent::Done => Self::Done,
            SasVerificationEvent::Cancelled(info) => Self::Cancelled(info),
            SasVerificationEvent::Error(error) => Self::Errored(error),
            SasVerificationEvent::Started => Self::Ongoing,
            SasVerificationEvent::VerifyEmojis(emoji) => Self::Emoji(emoji),
            SasVerificationEvent::Confirmed => Self::Confirmed,
            SasVerificationEvent::Created => Self::Created,
        }
    }
}

impl VerificationState {
    /// Returns `true` if a validation flow is currently running.
    pub fn is_validating(&self) -> bool {
        !matches!(self, Self::Stale)
    }
}
