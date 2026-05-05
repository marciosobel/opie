use iced::widget::image;
use matrix::services::UserInfo;

use crate::Image;

#[derive(Debug, Clone)]
pub struct User {
    pub info: UserInfo,
    pub avatar: Image,
}

impl std::ops::Deref for User {
    type Target = UserInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

impl User {
    pub fn new(info: UserInfo) -> Self {
        let avatar = match info.avatar().cloned() {
            Some(bytes) => Image::Ready(image::Handle::from_bytes(bytes)),
            None => Image::None,
        };

        User { info, avatar }
    }

    pub fn avatar(&self) -> &Image {
        &self.avatar
    }

    pub fn display_name_or<'a, S: ToString>(&'a self, fallback: S) -> String {
        match self.display_name() {
            Some(name) => name,
            None => fallback.to_string(),
        }
    }

    pub fn display_name_or_id(&self) -> String {
        self.display_name_or(self.id())
    }
}
