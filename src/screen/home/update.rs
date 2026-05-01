use std::sync::Arc;

use iced::{Task, widget::image};
use matrix::bridge::action::UserAction;

use super::{Image, Instruction, Message, State};

use crate::Action;
use crate::screen::home::FocusedRoom;
use crate::screen::home::{Timeline, view::settings_popup};
use matrix::{
    bridge::{
        Event as MatrixEvent,
        action::{Action as MatrixAction, TimelineAction},
    },
    services::{Room, TimelineEvent, sas_verification},
};

impl State {
    pub fn update(&mut self, message: Message) -> Action<Instruction, Message> {
        match message {
            Message::MatrixEvent(event) => {
                return self.matrix_event(event);
            }
            Message::PaginateForwards(id) => {
                self.bridge.send(TimelineAction::PaginateForwards(id));
            }
            Message::PaginateBackwards(id) => {
                self.bridge.send(TimelineAction::PaginateBackwards(id));
            }
            Message::OpenTimeline(id) => match self.focused_rooms.get(&id) {
                Some(_) => {
                    // Timeline is already open. Nothing to do.
                }
                None => {
                    if !self.rooms.contains_key(&id) {
                        tracing::error!(
                            "Tried to focus room {} but it is not present in room map.",
                            id.to_string()
                        );
                        return Action::none();
                    }

                    // TODO: Support having multiple rooms opened
                    for room in self.focused_rooms.values() {
                        self.bridge.send(TimelineAction::Close(room.id.clone()));
                    }

                    let focused_room = FocusedRoom::new(id.clone());
                    self.focused_rooms.insert(id.clone(), focused_room);
                    self.bridge.send(TimelineAction::Get(id));
                }
            },
            Message::CloseTimeline(id) => {
                self.focused_rooms.remove(&id);
                self.bridge.send(TimelineAction::Close(id));
            }
            Message::TimelineStart(id) => {
                if let Some(room) = self.focused_rooms.get_mut(&id) {
                    room.timeline.hit_start = true;
                }
            }
            Message::LoadRoomAvatar(id) => {
                let Some(room) = self.rooms.get(&id) else {
                    tracing::error!(
                        "Received LoadRoomAvatar for room id {} but it was not found in the rooms list",
                        id
                    );
                    return Action::none();
                };

                let task = Task::perform(load_room_avatar(room.clone()), |image| {
                    Message::RoomAvatarLoaded(id, image)
                });

                return Action::task(task);
            }
            Message::RoomAvatarLoaded(id, image) => {
                self.image_cache.rooms.insert(id, image);
            }
            Message::ToggleSpaceOpen(id) => {
                let old = self.collapsibles.spaces.get(&id).unwrap_or(&false);
                self.collapsibles.spaces.insert(id, !*old);
            }
            Message::ToggleDirectMessagesOpen => {
                self.collapsibles.dms = !self.collapsibles.dms;
            }
            Message::ToggleGroupMessagesOpen => {
                self.collapsibles.groups = !self.collapsibles.groups;
            }
            Message::ToggleSettingsPopupOpen => self.settings.open = !self.settings.open,
            Message::MessageInputChanged(id, text) => {
                if let Some(room) = self.focused_rooms.get_mut(&id) {
                    room.message_draft = text;
                } else {
                    tracing::warn!(
                        "Received MessageInputChanged for room id {} but it is not currently focused.",
                        id
                    );
                };
            }
            Message::SendMessage(id) => {
                let Some(room) = self.focused_rooms.get_mut(&id) else {
                    tracing::warn!(
                        "Received SendMessage for room id {} but it is not currently focused.",
                        id
                    );
                    return Action::none();
                };

                let message = room.message_draft.trim().to_string();
                room.message_draft = String::new();

                if !message.is_empty() {
                    self.bridge
                        .send(TimelineAction::SendMessage(id, message.to_string()));
                }
            }
            Message::LoadUserAvatar(id, uri) => {
                if !self.image_cache.users.contains_key(&id) {
                    self.bridge
                        .send(UserAction::FetchUserAvatar(id.clone(), uri));
                    self.image_cache.users.insert(id, Image::Fetching);
                }
            }
            Message::SettingsPopup(message) => {
                let action = self.settings.update(message).map(Message::SettingsPopup);

                let instruction_task = match action.instruction {
                    Some(instruction) => self.handle_settings_instruction(instruction),
                    None => Task::none(),
                };

                return Action::task(instruction_task.chain(action.task));
            }
        }

        Action::none()
    }

    fn handle_settings_instruction(
        &mut self,
        instruction: settings_popup::Instruction,
    ) -> Task<Message> {
        match instruction {
            settings_popup::Instruction::GetDeviceList => {
                self.bridge.send(MatrixAction::GetDevices)
            }
            settings_popup::Instruction::VerifyDevice(id) => self.bridge.send(
                MatrixAction::SasVerification(sas_verification::Action::VerifyDevice(id)),
            ),
        };

        Task::none()
    }

    fn matrix_event(&mut self, event: MatrixEvent) -> Action<Instruction, Message> {
        match event {
            MatrixEvent::RoomList(rooms) => {
                self.is_fetching_rooms = false;
                self.rooms = rooms;
            }
            MatrixEvent::TimelineEvent(event) => match event {
                TimelineEvent::Initial(room_id, items) => {
                    if !self.focused_rooms.contains_key(&room_id) {
                        let timeline = Timeline::with_items(items);
                        let focused_room = FocusedRoom::with_timeline(room_id.clone(), timeline);
                        self.focused_rooms.insert(room_id, focused_room);
                    }
                }
                TimelineEvent::Updated(room_id, updated) => {
                    if let Some(room) = self.focused_rooms.get_mut(&room_id) {
                        room.timeline.items = updated;
                    }
                }
                TimelineEvent::Closed(room_id) => {
                    self.focused_rooms.remove(&room_id);
                }
                TimelineEvent::Start(room_id) => {
                    if let Some(room) = self.focused_rooms.get_mut(&room_id) {
                        room.timeline.hit_start = true;
                    }
                }
                TimelineEvent::End(room_id) => {
                    if let Some(room) = self.focused_rooms.get_mut(&room_id) {
                        room.timeline.hit_end = true;
                    }
                }
            },
            MatrixEvent::DeviceList(devices) => {
                use settings_popup::Message;
                self.settings.update(Message::DeviceList(devices));
            }
            MatrixEvent::UserAvatarFetched(user_id, bytes) => {
                self.image_cache
                    .users
                    .insert(user_id, Image::Ready(image::Handle::from_bytes(bytes)));
            }
            _ => {}
        }
        Action::none()
    }
}

async fn load_room_avatar(room: Arc<Room>) -> Image {
    match room.avatar() {
        Some(bytes) => {
            let handle = image::Handle::from_bytes(bytes.clone());
            Image::Ready(handle)
        }
        None => Image::None,
    }
}
