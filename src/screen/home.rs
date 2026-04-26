use std::{collections::HashMap, sync::Arc};

use iced::widget::image;

use matrix::{
    bridge::{Action as MatrixAction, Bridge, Event as MatrixEvent},
    services::{
        room::{Room, RoomId},
        timeline::{self, TimelineItem},
        user::{UserId, UserInfo},
    },
};

mod update;
mod view;

#[derive(Debug, Clone)]
pub struct State {
    bridge: Bridge,
    user_id: UserId,
    rooms: HashMap<RoomId, Arc<Room>>,
    collapsible_spaces: HashMap<RoomId, bool>,
    collapsible_dms_open: bool,
    focused_room: Option<RoomId>,
    room_avatar_cache: HashMap<RoomId, Image>,
    timelines: HashMap<RoomId, timeline::Vector<Arc<TimelineItem>>>,
    settings_popup: view::settings_popup::State,
    users: HashMap<UserId, User>,
}

#[derive(Debug, Clone)]
pub enum Message {
    MatrixEvent(MatrixEvent),
    SpaceOpened(RoomId),
    SpaceClosed(RoomId),
    DirectMessagesOpened,
    DirectMessagesClosed,
    FocusRoom(RoomId),
    LoadRoomAvatar(RoomId),
    RoomAvatarLoaded(RoomId, Image),
    OpenSettingsPopup,
    CloseSettingsPopup,
    SettingsPopup(view::settings_popup::Message),
}

#[derive(Debug, Clone)]
pub enum Instruction {}

#[derive(Debug, Clone)]
pub struct User {
    info: UserInfo,
    avatar: Image,
}

impl std::ops::Deref for User {
    type Target = UserInfo;

    fn deref(&self) -> &Self::Target {
        &self.info
    }
}

#[derive(Debug, Clone)]
pub enum Image {
    Ready(image::Handle),
    None,
}

impl State {
    pub fn new(mut bridge: Bridge, user_info: UserInfo) -> Self {
        bridge.send(MatrixAction::ListAllRooms);

        let user = User::new(user_info);
        let user_id = user.id();
        let mut users = HashMap::new();
        users.insert(user.id(), user);

        Self {
            bridge,
            user_id,
            rooms: HashMap::new(),
            collapsible_spaces: HashMap::new(),
            collapsible_dms_open: false,
            focused_room: None,
            room_avatar_cache: HashMap::new(),
            timelines: HashMap::new(),
            settings_popup: view::settings_popup::State::new(),
            users,
        }
    }

    pub fn own_user(&self) -> &User {
        self.users
            .get(&self.user_id)
            .expect("State should have user logged in")
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
}
