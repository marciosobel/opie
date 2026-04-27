use std::{collections::HashMap, sync::Arc};

use iced::widget::image;

use matrix::{
    bridge::{Action as MatrixAction, Bridge, Event as MatrixEvent},
    services::{
        room::{Room, RoomId},
        timeline,
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
    timelines: HashMap<RoomId, Timeline>,
    settings_popup: view::settings_popup::State,
    users: HashMap<UserId, User>,
    message_inputs: HashMap<RoomId, String>,
}

#[derive(Debug, Clone)]
struct Timeline {
    items: timeline::Vector<Arc<timeline::TimelineItem>>,
    hit_start: bool,
    hit_end: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Timeline(TimelineMessage),
    MatrixEvent(MatrixEvent),
    LoadRoomAvatar(RoomId),
    RoomAvatarLoaded(RoomId, Image),
    SetSpaceOpen(RoomId, bool),
    SetDirectMessagesOpen(bool),
    SetSettingsPopupOpen(bool),
    SettingsPopup(view::settings_popup::Message),
    MessageInputChanged(RoomId, String),
    SendMessage(RoomId),
}

#[derive(Debug, Clone)]
pub enum TimelineMessage {
    PaginateForwards(RoomId),
    PaginateBackwards(RoomId),
    LoadTimeline(RoomId),
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
            message_inputs: HashMap::new(),
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

impl Timeline {
    fn new() -> Self {
        Self {
            items: timeline::Vector::new(),
            hit_end: false,
            hit_start: false,
        }
    }

    fn from_items(items: timeline::Vector<Arc<timeline::TimelineItem>>) -> Self {
        Self {
            items,
            hit_end: false,
            hit_start: false,
        }
    }
}
