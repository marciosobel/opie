use iced::{
    Element,
    widget::{button, center, mouse_area, opaque, row, stack},
};

const DEPTH_PADDING: f32 = 24.0;
const HORIZONTAL_PADDING: f32 = 10.0;

use super::{Image, Message, State};

mod main_view;
mod sidebar;

impl State {
    pub fn view(&self) -> Element<'_, Message> {
        let base = row![self.sidebar(), self.main_view()];

        if self.profile_open {
            let popup = opaque(
                mouse_area(center(opaque(self.profile_popup())))
                    .on_press(Message::CloseProfilePopup),
            );

            stack![base, popup].into()
        } else {
            base.into()
        }
    }

    fn profile_popup<'a>(&self) -> Element<'a, Message> {
        todo!("Add popup content")
    }
}
