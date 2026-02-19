use iced::{
    Element,
    widget::{center, text},
};

use crate::{Action, matrix::bridge::MatrixBridgeSender};

pub struct State {
    bridge: MatrixBridgeSender,
}

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Debug, Clone)]
pub enum Instruction {}

impl State {
    pub fn new(bridge: MatrixBridgeSender) -> Self {
        Self { bridge }
    }

    pub fn update(&mut self, _message: Message) -> Action<Instruction, Message> {
        Action::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        center(text("Welcome to the chat screen")).into()
    }
}
