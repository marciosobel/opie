use std::sync::Arc;

use iced::{Task, widget::image};
use matrix_sdk_ui::eyeball_im::Vector;

use super::{Image, Instruction, Message, State};

use crate::Action;
use crate::matrix::services::sas_verification;
use crate::matrix::{
    bridge::{Action as MatrixAction, Event as MatrixEvent},
    services::{Room, TimelineEvent},
};
use crate::screen::home::view::settings_popup;

impl State {
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
                self.room_avatar_cache.insert(id, image);
            }
            Message::OpenSettingsPopup => self.settings_popup.open = true,
            Message::CloseSettingsPopup => self.settings_popup.open = false,
            Message::SettingsPopup(message) => {
                let action = self
                    .settings_popup
                    .update(message)
                    .map(Message::SettingsPopup);

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
            MatrixEvent::RoomList(rooms) => self.rooms = rooms,
            MatrixEvent::TimelineEvent(event) => match event {
                TimelineEvent::Initial(room_id, items) => {
                    self.timelines.insert(room_id, items);
                }
                TimelineEvent::Updated(room_id, diffs) => {
                    let current_timeline = self
                        .timelines
                        .entry(room_id.clone())
                        .or_insert_with(Vector::new);

                    for diff in diffs {
                        diff.apply(current_timeline);
                    }
                }
                TimelineEvent::Closed(room_id) => {
                    self.timelines.remove(&room_id);
                }
            },
            MatrixEvent::DeviceList(devices) => {
                self.settings_popup
                    .update(settings_popup::Message::DeviceList(devices));
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
