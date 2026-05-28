use std::{collections::HashMap, sync::Arc};

use matrix::{
    bridge::{Action as MatrixAction, Bridge, Event as MatrixEvent, action::media::MediaSource},
    services::{
        room::{Room, RoomId},
        timeline::EventId,
        user::{UserId, UserInfo},
    },
};

mod update;
mod view;

mod collapsibles;
mod focused_room;
mod image_cache;
mod is_fetching;
mod timeline;
mod user;

use collapsibles::Collapsibles;
use focused_room::FocusedRoom;
use image_cache::ImageCache;
use is_fetching::IsFetching;
use timeline::Timeline;
use user::User;

use crate::Image;

#[derive(Debug, Clone)]
pub struct State {
    bridge: Bridge,
    user_id: UserId,
    rooms: HashMap<RoomId, Arc<Room>>,
    collapsibles: Collapsibles,
    image_cache: ImageCache,
    focused_rooms: HashMap<RoomId, FocusedRoom>,
    settings: view::settings_popup::State,
    users: HashMap<UserId, User>,
    is_fetching: IsFetching,
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

    GetUser(UserId),
    GetUserResponse(User),

    SettingsPopup(view::settings_popup::Message),
    FetchTimelineImage(EventId, MediaSource),
}

#[derive(Debug, Clone)]
pub enum Instruction {}

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
            is_fetching: IsFetching::new(),
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
