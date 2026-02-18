use iced::{
    Element,
    widget::{center, text},
};

use crate::{Action, matrix::bridge::MatrixBridgeSender as MatrixBridge};

pub struct State {
    bridge: MatrixBridge,
}

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Debug, Clone)]
pub enum Instruction {}

impl State {
    pub fn new(bridge: MatrixBridge) -> Self {
        Self { bridge }
    }

    pub fn update(&mut self, _message: Message) -> Action<Instruction, Message> {
        Action::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        center(text("Welcome to the chat screen")).into()
    }
}
