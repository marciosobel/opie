use std::{collections::HashMap, sync::Arc};

use iced::widget::image;
use matrix_sdk::ruma::OwnedRoomId;
use matrix_sdk_ui::{eyeball_im::Vector, timeline::TimelineItem};

use crate::matrix::{
    bridge::{Action as MatrixAction, Bridge, Event as MatrixEvent},
    services::{Room, UserInfo},
};

mod update;
mod view;

#[derive(Debug, Clone)]
pub struct State {
    bridge: Bridge,
    user: User,
    rooms: HashMap<OwnedRoomId, Arc<Room>>,
    collapsible_spaces: HashMap<OwnedRoomId, bool>,
    collapsible_dms_open: bool,
    focused_room: Option<OwnedRoomId>,
    room_avatar_cache: HashMap<OwnedRoomId, Image>,
    timelines: HashMap<OwnedRoomId, Vector<Arc<TimelineItem>>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    MatrixEvent(MatrixEvent),
    SpaceOpened(OwnedRoomId),
    SpaceClosed(OwnedRoomId),
    DirectMessagesOpened,
    DirectMessagesClosed,
    FocusRoom(OwnedRoomId),
    LoadRoomAvatar(OwnedRoomId),
    RoomAvatarLoaded(OwnedRoomId, Image),
    SidebarProfileClicked,
}

#[derive(Debug, Clone)]
pub enum Instruction {}

#[derive(Debug, Clone)]
struct User {
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

        let user_avatar = match user_info.avatar().cloned() {
            Some(bytes) => Image::Ready(image::Handle::from_bytes(bytes)),
            None => Image::None,
        };

        let user = User {
            info: user_info,
            avatar: user_avatar,
        };

        Self {
            bridge,
            user,
            rooms: HashMap::new(),
            collapsible_spaces: HashMap::new(),
            collapsible_dms_open: false,
            focused_room: None,
            room_avatar_cache: HashMap::new(),
            timelines: HashMap::new(),
        }
    }
}
