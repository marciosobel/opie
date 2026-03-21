use std::fmt::Display;

use bytes::Bytes;
use matrix_sdk::ruma::OwnedUserId;

#[derive(Debug, Clone)]
pub struct UserInfo {
    id: OwnedUserId,
    display_name: Option<String>,
    avatar: Option<Bytes>,
}

impl UserInfo {
    pub fn new(id: OwnedUserId, display_name: Option<String>, avatar: Option<Vec<u8>>) -> Self {
        let avatar = match &avatar {
            Some(bytes) => Some(Bytes::copy_from_slice(bytes)),
            None => None,
        };

        Self {
            id,
            display_name,
            avatar,
        }
    }

    pub fn display_name(&self) -> Option<String> {
        self.display_name.clone()
    }

    pub fn avatar(&self) -> Option<&Bytes> {
        self.avatar.as_ref()
    }

    pub fn id(&self) -> OwnedUserId {
        self.id.clone()
    }
}

impl Display for UserInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_name = match &self.display_name {
            Some(name) => name.clone(),
            None => "Unknown".to_string(),
        };

        write!(
            f,
            "UserInfo {{ id: {}, isplay_name: {}, .. }}",
            self.id, display_name
        )
    }
}
