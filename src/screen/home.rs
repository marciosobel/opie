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
    is_fetching_rooms: bool,
    collapsibles: Collapsibles,
    image_cache: ImageCache,
    focused_rooms: HashMap<RoomId, FocusedRoom>,
    settings: view::settings_popup::State,
    users: HashMap<UserId, User>,
}

#[derive(Debug, Clone)]
struct FocusedRoom {
    id: RoomId,
    message_draft: String,
    timeline: Timeline,
}

#[derive(Debug, Clone, Default)]
struct Collapsibles {
    spaces: HashMap<RoomId, bool>,
    dms: bool,
    groups: bool,
}

#[derive(Debug, Clone, Default)]
struct ImageCache {
    rooms: HashMap<RoomId, Image>,
}

#[derive(Debug, Clone)]
struct Timeline {
    items: timeline::Vector<Arc<timeline::TimelineItem>>,
    hit_start: bool,
    hit_end: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    MatrixEvent(MatrixEvent),

    /// Loads more messages at the end of the timeline items.
    PaginateForwards(RoomId),
    /// Loads more messages at the start of the timeline items.
    PaginateBackwards(RoomId),
    OpenTimeline(RoomId),
    CloseTimeline(RoomId),
    TimelineStart(RoomId),

    LoadRoomAvatar(RoomId),
    RoomAvatarLoaded(RoomId, Image),

    ToggleSpaceOpen(RoomId),
    ToggleDirectMessagesOpen,
    ToggleGroupMessagesOpen,
    ToggleSettingsPopupOpen,

    MessageInputChanged(RoomId, String),
    SendMessage(RoomId),

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
            users,
            rooms: HashMap::new(),
            is_fetching_rooms: true,
            focused_rooms: HashMap::new(),
            collapsibles: Collapsibles::default(),
            image_cache: ImageCache::default(),
            settings: view::settings_popup::State::new(),
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

    fn with_items(items: timeline::Vector<Arc<timeline::TimelineItem>>) -> Self {
        Self {
            items,
            hit_end: false,
            hit_start: false,
        }
    }
}

impl FocusedRoom {
    fn new(room_id: RoomId) -> Self {
        Self {
            id: room_id,
            message_draft: String::new(),
            timeline: Timeline::new(),
        }
    }

    fn with_timeline(room_id: RoomId, timeline: Timeline) -> Self {
        Self {
            id: room_id,
            message_draft: String::new(),
            timeline,
        }
    }
}

impl PartialEq for FocusedRoom {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for FocusedRoom {}
