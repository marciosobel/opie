use iced::{
    Element,
    widget::{center, column, container, row, text},
};
use matrix_sdk::Room;

use crate::{
    Action,
    matrix::bridge::{Action as MatrixAction, Event as MatrixEvent, MatrixBridgeSender},
};

#[derive(Debug, Clone)]
pub struct State {
    // bridge: MatrixBridgeSender,
    rooms: Vec<Room>,
}

#[derive(Debug, Clone)]
pub enum Message {
    MatrixEvent(MatrixEvent),
}

#[derive(Debug, Clone)]
pub enum Instruction {}

impl State {
    pub fn new(mut bridge: MatrixBridgeSender) -> Self {
        _ = bridge.try_send(MatrixAction::ListAllRooms);

        Self {
            // bridge,
            rooms: Vec::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Action<Instruction, Message> {
        match message {
            Message::MatrixEvent(event) => self.matrix_event(event),
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let mut row = row![];

        if !self.rooms.is_empty() {
            row = row.push(self.rooms());
        }

        row = row.push(center(text("Welcome to the chat screen")));
        row.into()
    }

    pub fn rooms(&self) -> Element<'_, Message> {
        let mut column = column![];

        for room in self.rooms.iter() {
            let name = room
                .cached_display_name()
                .map(|name| name.to_string())
                .unwrap_or("Failed to get name".into());

            let room_element = if room.is_space() {
                container(text(name)).style(container::primary)
            } else {
                container(text(name))
            };

            column = column.push(room_element);
        }

        column.into()
    }

    fn matrix_event(&mut self, event: MatrixEvent) -> Action<Instruction, Message> {
        match event {
            MatrixEvent::RoomList(rooms) => {
                self.rooms = rooms;
                Action::none()
            }
            _ => Action::none(),
        }
    }
}
