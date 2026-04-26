use iced::{
    Element, Length,
    widget::{center, column, space, text},
};
use matrix::services::timeline::MessageType;

use super::{Message, State};

impl State {
    pub(super) fn main_view(&self) -> Element<'_, Message> {
        let Some(room_id) = &self.focused_room else {
            return center(text("Welcome to the chat screen")).into();
        };

        let Some(timeline) = self.timelines.get(room_id) else {
            return center(text("Loading...")).into();
        };

        let mut chat_item = column![
            text!("Timeline for {}", room_id),
            space::vertical().height(40)
        ]
        .spacing(10);

        for item in timeline.iter() {
            if let Some(item) = item.as_event() {
                let item_content = item.content();

                if let Some(message) = item_content.as_message() {
                    let element = match message.msgtype() {
                        MessageType::Audio(content) => text!("Audio: {}", content.filename()),
                        MessageType::Emote(content) => text!("Emote: {}", &content.body),
                        MessageType::File(content) => text!("File: {}", content.filename()),
                        MessageType::Image(content) => text!("Image: {}", content.filename()),
                        MessageType::Location(content) => {
                            text!("Location: {}", content.plain_text_representation())
                        }
                        MessageType::Notice(content) => text!("Notce: {}", &content.body),
                        MessageType::ServerNotice(content) => {
                            text!("Server notice: {}", &content.body)
                        }
                        MessageType::Text(content) => text!("Text: {}", &content.body),
                        MessageType::Video(content) => text!("Video: {}", content.filename()),
                        MessageType::VerificationRequest(content) => {
                            text!("Verification request: {}", content.to.to_string())
                        }
                        _ => todo!(),
                    };
                    chat_item = chat_item.push(element);
                }
            }
        }

        chat_item.width(Length::Fill).into()
    }
}
