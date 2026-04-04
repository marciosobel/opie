use iced::{
    Color, Element, padding,
    widget::{center, container, mouse_area, opaque, row, stack},
};

const DEPTH_PADDING: f32 = 24.0;
const HORIZONTAL_PADDING: f32 = 10.0;

use super::{Image, Message, State};

mod main_view;
pub(super) mod settings_popup;
mod sidebar;

impl State {
    pub fn view(&self) -> Element<'_, Message> {
        let base = row![self.sidebar(), self.main_view()];

        if !self.settings_popup.open {
            return base.into();
        }

        let content = container(self.settings_popup.view().map(Message::SettingsPopup))
            .style(|theme| container::background(theme.palette().background));

        let modal = container(opaque(
            mouse_area(center(opaque(content)).padding(padding::vertical(160).horizontal(80)))
                .on_press(Message::CloseSettingsPopup),
        ))
        .style(|_| {
            container::Style::default().background(Color {
                a: 0.8,
                ..Color::BLACK
            })
        });

        stack![base, modal].into()
    }
}
