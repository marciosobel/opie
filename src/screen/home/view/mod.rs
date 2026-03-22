use iced::{Element, widget::row};

const DEPTH_PADDING: f32 = 24.0;
const HORIZONTAL_PADDING: f32 = 10.0;

use super::{Image, Message, State};

mod main_view;
mod sidebar;

impl super::State {
    pub fn view(&self) -> Element<'_, Message> {
        row![self.sidebar(), self.main_view()].into()
    }
}
