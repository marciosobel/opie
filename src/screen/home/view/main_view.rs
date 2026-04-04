use iced::{
    Element, Length,
    widget::{center, column, text},
};

use super::{Message, State};

impl State {
    pub(super) fn main_view(&self) -> Element<'_, Message> {
        let Some(room_id) = &self.focused_room else {
            return center(text("Welcome to the chat screen")).into();
        };

        let Some(timeline) = self.timelines.get(room_id) else {
            return center(text("Loading...")).into();
        };

        let mut content = column![text!("Timeline for {}", room_id)];
        for item in timeline.iter() {
            if let Some(item) = item.as_event() {
                let item_content = item.content();
                if !item_content.is_message() {
                    continue;
                }

                let Some(message) = item_content.as_message() else {
                    continue;
                };

                let message = message.body().to_string();
                content = content.push(text(message));
            }
        }

        content.width(Length::Fill).into()
    }
}
