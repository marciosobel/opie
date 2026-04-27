use chrono::DateTime;
use components::separator;
use iced::{
    Element, Length,
    widget::{center, column, scrollable, sensor, space, text, text_input},
};
use lucide_icons::Icon;
use matrix::services::timeline::{
    EventTimelineItem, MessageType, MsgLikeKind, TimelineItemContent, TimelineItemKind,
    VirtualTimelineItem,
};

use crate::screen::home::TimelineMessage;

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

        if timeline.hit_start {
            chat_item = chat_item.push(text("Hit start of timeline"));
        } else {
            chat_item = chat_item.push(
                sensor(text("[backwards] Loading more messages...")).on_show(|_| {
                    Message::Timeline(TimelineMessage::PaginateBackwards(room_id.clone()))
                }),
            )
        }

        for item in &timeline.items {
            let element: Element<'_, Message> = match item.kind() {
                TimelineItemKind::Virtual(item) => self.virtual_timeline_item(item),
                TimelineItemKind::Event(item) => self.timeline_item(item),
            };

            chat_item = chat_item.push(element);
        }

        if timeline.hit_end {
            chat_item = chat_item.push(text("Hit end of timeline"));
        } else {
            chat_item = chat_item.push(
                sensor(text("[forwards] Loading more messages...")).on_show(|_| {
                    Message::Timeline(TimelineMessage::PaginateForwards(room_id.clone()))
                }),
            )
        }

        let message_input = {
            let current_message_input = match self.message_inputs.get(room_id) {
                Some(old_message) => old_message.as_str(),
                None => "",
            };

            text_input("Message...", current_message_input)
                .on_input(|text| Message::MessageInputChanged(room_id.clone(), text))
                .on_submit(Message::SendMessage(room_id.clone()))
        };
        let conversation = scrollable(chat_item.width(Length::Fill))
            .height(Length::Fill)
            .anchor_bottom();

        column![conversation, message_input].into()
    }

    fn virtual_timeline_item(&self, item: &VirtualTimelineItem) -> Element<'_, Message> {
        match item {
            VirtualTimelineItem::DateDivider(date) => {
                if let Some(date) = DateTime::from_timestamp_millis(date.get().into()) {
                    let date = date.format("%Y-%m-%d").to_string();
                    text(date).style(text::secondary).into()
                } else {
                    separator::horizontal().into()
                }
            }
            VirtualTimelineItem::ReadMarker => Icon::CheckCheck.widget().into(),
            VirtualTimelineItem::TimelineStart => text("Start of the timeline").into(),
        }
    }

    fn timeline_item(&self, item: &EventTimelineItem) -> Element<'_, Message> {
        match item.content() {
            TimelineItemContent::MsgLike(content) => match &content.kind {
                MsgLikeKind::Message(message) => match message.msgtype() {
                    MessageType::Audio(content) => text!("Audio: {}", content.filename()).into(),
                    MessageType::Emote(content) => text!("Emote: {}", &content.body).into(),
                    MessageType::File(content) => text!("File: {}", content.filename()).into(),
                    MessageType::Image(content) => text!("Image: {}", content.filename()).into(),
                    MessageType::Location(content) => {
                        text!("Location: {}", content.plain_text_representation()).into()
                    }
                    MessageType::Notice(content) => text!("Notce: {}", &content.body).into(),
                    MessageType::ServerNotice(content) => {
                        text!("Server notice: {}", &content.body).into()
                    }
                    MessageType::Text(content) => {
                        let mut element = text!("Text: {}", &content.body);
                        if item.is_local_echo() {
                            element = element.style(text::secondary);
                        }
                        element.into()
                    }
                    MessageType::Video(content) => text!("Video: {}", content.filename()).into(),
                    MessageType::VerificationRequest(content) => {
                        text!("Verification request: {}", content.to.to_string()).into()
                    }
                    _ => todo!(),
                },
                MsgLikeKind::Sticker(sticker) => {
                    text!("Sticker: {}", sticker.content().body).into()
                }
                MsgLikeKind::Poll(poll_state) => {
                    text!("Poll: {}", poll_state.results().question).into()
                }
                MsgLikeKind::Redacted => text("Redacted.").into(),
                MsgLikeKind::UnableToDecrypt(_) => text!("Encrypted message").into(),
                MsgLikeKind::Other(message) => text!("Unknown message type: {:#?}", message).into(),
            },
            TimelineItemContent::MembershipChange(member) => {
                text!("Membership change: {}", member.user_id().to_string()).into()
            }
            TimelineItemContent::ProfileChange(member) => {
                text!("Profile Change: {}", member.user_id()).into()
            }
            TimelineItemContent::FailedToParseMessageLike { error, .. } => {
                text!("Failed to parse messagelike: {:#?}", error).into()
            }
            TimelineItemContent::FailedToParseState { error, .. } => {
                text!("Failed to parse state: {:#?}", error).into()
            }
            TimelineItemContent::CallInvite => text("Call invite").style(text::secondary).into(),
            _ => space().into(),
        }
    }
}
