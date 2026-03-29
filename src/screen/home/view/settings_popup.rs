use iced::{Element, widget::center};

use crate::Action;

#[derive(Debug, Clone)]
pub struct State {
    pub open: bool,
}

#[derive(Debug, Clone)]
pub enum Message {}

#[derive(Debug, Clone)]
pub enum Instruction {}

impl State {
    pub fn new() -> Self {
        Self { open: false }
    }

    pub fn view(&self) -> Element<'_, Message> {
        center("hi").into()
    }

    pub fn update(&mut self, _message: Message) -> Action<Instruction, Message> {
        Action::none()
    }
}
