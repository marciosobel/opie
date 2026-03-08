use std::{collections::HashMap, sync::Arc};

use iced::{
    Element, Length,
    widget::{button, center, column, container, row, text},
};
use matrix_sdk::ruma::OwnedRoomId;

use crate::{
    Action,
    components::collapsible,
    matrix::{
        bridge::{Action as MatrixAction, Event as MatrixEvent, MatrixBridgeSender},
        services::Room,
    },
};

#[derive(Debug, Clone)]
pub struct State {
    // bridge: MatrixBridgeSender,
    rooms: Arc<HashMap<OwnedRoomId, Room>>,
    collapsible_spaces: HashMap<OwnedRoomId, bool>,
    collapsible_dms_open: bool,
    focused_room: Option<OwnedRoomId>,
}

#[derive(Debug, Clone)]
pub enum Message {
    MatrixEvent(MatrixEvent),
    SpaceOpened(OwnedRoomId),
    SpaceClosed(OwnedRoomId),
    DirectMessagesOpened,
    DirectMessagesClosed,
    FocusRoom(OwnedRoomId),
}

#[derive(Debug, Clone)]
pub enum Instruction {}

impl State {
    pub fn new(mut bridge: MatrixBridgeSender) -> Self {
        _ = bridge.try_send(MatrixAction::ListAllRooms);

        Self {
            // bridge,
            rooms: Arc::default(),
            collapsible_spaces: HashMap::new(),
            collapsible_dms_open: false,
            focused_room: None,
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
            Message::FocusRoom(id) => self.focused_room = Some(id),
        }

        Action::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        row![self.sidebar(), self.main_view()].into()
    }

    pub fn main_view(&self) -> Element<'_, Message> {
        if let Some(room_id) = &self.focused_room {
            let room = self.rooms.get(room_id).expect(&format!(
                "Something went wrong getting the room with id {}",
                room_id
            ));

            let name = room.display_name().unwrap_or("Unknown".into());
            center(text!("Focused on room {}", name)).into()
        } else {
            center(text("Welcome to the chat screen")).into()
        }
    }

    pub fn sidebar(&self) -> Element<'_, Message> {
        let mut sidebar = column![].padding(10).spacing(10);

        let root_parents = self
            .rooms
            .values()
            .filter(|room| room.parents().is_empty() || room.is_direct())
            .collect::<Vec<_>>();

        let direct_rooms = root_parents.iter().filter(|room| room.is_direct());
        let mut dms = column![].spacing(10);
        for dm in direct_rooms {
            dms = dms.push(self.render_room(dm));
        }
        let dm_collapsible = collapsible(text("Direct Messages"))
            .on_close(Message::DirectMessagesClosed)
            .on_open(Message::DirectMessagesOpened)
            .open(self.collapsible_dms_open)
            .content(dms)
            .spacing(10);
        sidebar = sidebar.push(dm_collapsible);

        let space_rooms = root_parents.iter().filter(|room| !room.is_direct());
        let mut spaces = column![].spacing(10);
        for space in space_rooms {
            spaces = spaces.push(self.render_space(space))
        }
        sidebar = sidebar.push(spaces);

        container(sidebar)
            .height(Length::Fill)
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();
                container::Style::default()
                    .background(palette.background.weak.color)
                    .color(palette.background.weak.text)
            })
            .into()
    }

    fn render_space(&self, space: &Room) -> Element<'_, Message> {
        let mut content = column![].spacing(10);

        for child_id in space.children() {
            let Some(child) = self.rooms.values().find(|room| room.id() == *child_id) else {
                tracing::error!("Failed to find child room with id {}", child_id);
                continue;
            };

            if child.is_space() {
                content = content.push(self.render_space(child));
            } else {
                content = content.push(self.render_room(child));
            }
        }

        let space_name = space
            .display_name()
            .unwrap_or("Failed to get space name".into());
        let open = self
            .collapsible_spaces
            .get(&space.id())
            .unwrap_or(&false)
            .to_owned();

        let collapsible = collapsible(text(space_name))
            .on_close(Message::SpaceClosed(space.id()))
            .on_open(Message::SpaceOpened(space.id()))
            .content(content)
            .open(open)
            .spacing(10);

        collapsible.into()
    }

    fn render_room(&self, room: &Room) -> Element<'_, Message> {
        let name = room
            .display_name()
            .unwrap_or("Failed to get room name".into());

        button(text(name))
            .on_press(Message::FocusRoom(room.id()))
            .into()
    }

    fn matrix_event(&mut self, event: MatrixEvent) -> Action<Instruction, Message> {
        match event {
            MatrixEvent::RoomList(rooms) => self.rooms = rooms,
            _ => {}
        }
        Action::none()
    }
}
