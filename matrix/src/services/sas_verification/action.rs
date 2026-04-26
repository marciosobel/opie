use matrix_sdk::ruma::OwnedDeviceId;

/// An action to be performed by the sas verification service
#[derive(Debug, Clone)]
pub enum Action {
    /// Send a sas verification request for this `OwnedDeviceId`
    VerifyDevice(OwnedDeviceId),
    /// Cancel the ongoing sas verification
    Cancel,
    /// Accept the ongoing verification
    Accept,
    /// Cancel the ongoing verification because the emojis presented doesn't match
    Mismatch,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Cancel => write!(f, "Cancel"),
            Action::VerifyDevice(device_id) => write!(f, "VerifyDevice({})", device_id.to_string()),
            Action::Accept => write!(f, "Accept"),
            Action::Mismatch => write!(f, "Mismatch"),
        }
    }
}
