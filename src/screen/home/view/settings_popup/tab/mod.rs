use std::fmt::{Display, Formatter, Result};

pub(super) use super::{Message, State};
use iced::Element;

mod device;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    Devices,
}

impl Tab {
    pub fn render(&self, state: &State) -> Element<'_, Message> {
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
