use std::fmt::{Display, Formatter, Result};

pub(super) use super::{Message, State};
use iced::Element;

mod device;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    Devices,
}

impl Tab {
    pub fn render<'a>(&'a self, state: &'a State) -> Element<'a, Message> {
        match self {
            Tab::Devices => self.render_device_tab(state),
        }
    }
}

impl Display for Tab {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Tab::Devices => write!(f, "Devices"),
        }
    }
}
