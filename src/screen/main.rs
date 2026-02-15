use iced::{
    Element,
    widget::{center, text},
};
use matrix_sdk::Client;

use crate::{Action, async_dropper::AsyncDropper};

pub struct State {
    client: AsyncDropper<Client>,
}

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Debug, Clone)]
pub enum Instruction {}

impl State {
    pub fn new(client: AsyncDropper<Client>) -> Self {
        Self { client }
    }

    pub fn update(&mut self, _message: Message) -> Action<Instruction, Message> {
        Action::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        center(text("Welcome to the chat screen")).into()
    }
}
