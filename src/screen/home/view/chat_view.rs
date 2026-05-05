use iced::{
    Element, Length, Padding,
    widget::{center, column, container, scrollable, sensor, space, text, text_input},
};
use matrix::services::{room::RoomId, timeline::TimelineItemKind, user::UserId};

use super::{Message, State};

mod timeline_message;
use timeline_message::TimelineMessage;

impl State {
    pub(super) fn main_view(&self) -> Element<'_, Message> {
        // TODO: render all chats present in `focused_rooms`.
        if let Some(room_id) = self.focused_rooms.keys().next() {
            self.chat(room_id)
        } else {
            center(text("Welcome to the chat screen")).into()
        }
    }

    fn chat<'a>(&'a self, room_id: &'a RoomId) -> Element<'a, Message> {
        let Some(room) = self.focused_rooms.get(room_id) else {
            return center(text("Loading...")).into();
        };

        let mut messages = column![].spacing(2.5);

        if !room.timeline.hit_start {
            messages = messages.push(
                sensor(
                    center(
                        text("Loading more messages...")
                            .size(12)
                            .style(text::secondary),
                    )
                    .padding(10),
                )
                .anticipate(50)
                .on_show(|_| Message::PaginateBackwards(room_id.clone())),
            );
        }

        let mut previous_sender: Option<UserId> = None;
        let mut is_previous_item_an_event = false;
        for item in &room.timeline.items {
            let mut is_same_sender = false;

            let message = match item.kind() {
                TimelineItemKind::Virtual(item) => {
                    previous_sender = None;
                    TimelineMessage::from_virtual(item, room)
                }
                TimelineItemKind::Event(item) => {
                    is_previous_item_an_event = false;
                    is_same_sender = previous_sender
                        .as_ref()
                        .map_or(false, |id| id == item.sender());

                    let message = TimelineMessage::from_event(item, self, !is_same_sender);
                    if let TimelineMessage::User(_) = message {
                        previous_sender = Some(item.sender().to_owned());
                    }
                    message
                }
            };

            let add_padding = |element: Element<'a, Message>| -> Element<'a, Message> {
                container(element).padding(Padding::ZERO.top(10)).into()
            };

            let message_content = match message {
                TimelineMessage::User(element) => {
                    if !is_same_sender {
                        add_padding(element)
                    } else {
                        element
                    }
                }
                TimelineMessage::Virtual(element) => {
                    if !is_previous_item_an_event {
                        add_padding(element)
                    } else {
                        is_previous_item_an_event = true;
                        element
                    }
                }
                TimelineMessage::Blank => continue,
            };

            messages = messages.push(message_content);
        }

        if !room.timeline.hit_end {
            messages = messages.push(
                sensor(space())
                    .anticipate(50)
                    .on_show(|_| Message::PaginateForwards(room_id.clone())),
            );
        }

        let messages = scrollable(messages.width(Length::Fill))
            .height(Length::Fill)
            .spacing(5)
            .anchor_bottom();

        let message_input = text_input("Message...", &room.message_draft)
            .on_input(|text| Message::MessageInputChanged(room_id.clone(), text))
            .on_submit(Message::SendMessage(room_id.clone()));

        column![messages, message_input]
            .spacing(10)
            .padding(10)
            .into()
    }
}
