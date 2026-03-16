use std::{collections::HashMap, sync::Arc};

use iced::{
    Element, Padding, Task,
    widget::{center, column, image, row, text},
};
use matrix_sdk::ruma::OwnedRoomId;
use matrix_sdk_ui::{eyeball_im::Vector, timeline::TimelineItem};

use crate::{
    Action,
    matrix::{
        bridge::{Action as MatrixAction, Bridge, Event as MatrixEvent},
        services::{Room, TimelineUpdateEvent},
    },
};

mod sidebar;

const DEPTH_PADDING: f32 = 24.0;
const HORIZONTAL_PADDING: f32 = 10.0;
const SIDEBAR_ROOM_PADDING: Padding = Padding {
    left: HORIZONTAL_PADDING,
    right: HORIZONTAL_PADDING,
    top: 2.5,
    bottom: 2.5,
};
const SIDEBAR_ROOM_AVATAR_SIZE: u32 = 20;

#[derive(Debug, Clone)]
pub struct State {
    bridge: Bridge,
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
}

#[derive(Debug, Clone)]
pub enum Instruction {}

impl State {
    pub fn new(mut bridge: Bridge) -> Self {
        bridge.send(MatrixAction::ListAllRooms);

        Self {
            bridge,
            rooms: HashMap::new(),
            collapsible_spaces: HashMap::new(),
            collapsible_dms_open: true,
            focused_room: None,
            room_avatar_cache: HashMap::new(),
            timelines: HashMap::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Action<Instruction, Message> {
        match message {
            Message::MatrixEvent(event) => {
                return self.matrix_event(event);
            }
            Message::SpaceOpened(id) => {
                self.collapsible_spaces.insert(id, true);
            }
            Message::SpaceClosed(id) => {
                self.collapsible_spaces.insert(id, false);
            }
            Message::DirectMessagesOpened => self.collapsible_dms_open = true,
            Message::DirectMessagesClosed => self.collapsible_dms_open = false,
            Message::FocusRoom(id) => match &self.focused_room {
                Some(focused_room_id) if *focused_room_id == id => {}
                maybe_focused_room_id => {
                    if let Some(focused_room_id) = maybe_focused_room_id {
                        tracing::info!(
                            "Closing the current timeline before requesting another one"
                        );
                        self.bridge
                            .send(MatrixAction::CloseTimeline(focused_room_id.clone()));
                    }

                    self.focused_room = Some(id.clone());
                    tracing::info!("Focusing room with id {}", id);
                    self.bridge.send(MatrixAction::GetTimeline(id));
                }
            },
            Message::LoadRoomAvatar(id) => {
                let Some(room) = self.rooms.get(&id).cloned() else {
                    tracing::error!(
                        "Received LoadRoomAvatar for room id {} but it was not found in the rooms list",
                        id
                    );
                    return Action::none();
                };

                let task = Task::perform(
                    async move {
                        let image = match room.avatar() {
                            Some(bytes) => {
                                let handle = image::Handle::from_bytes(bytes.clone());
                                Image::Ready(handle)
                            }
                            None => Image::None,
                        };

                        (room.id(), image)
                    },
                    |(id, image)| Message::RoomAvatarLoaded(id, image),
                );

                return Action::task(task);
            }
            Message::RoomAvatarLoaded(id, image) => {
                self.room_avatar_cache.insert(id, image);
            }
        }

        Action::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        row![self.sidebar(), self.main_view()].into()
    }

    pub fn main_view(&self) -> Element<'_, Message> {
        let Some(room_id) = &self.focused_room else {
            return center(text("Welcome to the chat screen")).into();
        };

        let Some(timeline) = self.timelines.get(room_id) else {
            return center(text("Loading...")).into();
        };

        let mut content = column![text!("Timeline for {}", room_id)];
        for item in timeline.iter() {
            if let Some(item) = item.as_event() {
                let item_content = item.content();
                if !item_content.is_message() {
                    continue;
                }

                let Some(message) = item_content.as_message() else {
                    continue;
                };

                let message = message.body().to_string();
                content = content.push(text(message));
            }
        }

        content.into()
    }

    fn matrix_event(&mut self, event: MatrixEvent) -> Action<Instruction, Message> {
        match event {
            MatrixEvent::RoomList(rooms) => self.rooms = rooms,
            MatrixEvent::TimelineEvent(event) => match event {
                TimelineUpdateEvent::Initial(room_id, items) => {
                    self.timelines.insert(room_id, items);
                }
                TimelineUpdateEvent::Updated(room_id, diffs) => {
                    let current_timeline = self
                        .timelines
                        .entry(room_id.clone())
                        .or_insert_with(Vector::new);

                    for diff in diffs {
                        diff.apply(current_timeline);
                    }
                }
                TimelineUpdateEvent::Closed(room_id) => {
                    self.timelines.remove(&room_id);
                }
            },
            _ => {}
        }
        Action::none()
    }
}

#[derive(Debug, Clone)]
pub enum Image {
    Ready(image::Handle),
    None,
}
