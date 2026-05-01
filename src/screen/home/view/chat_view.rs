use chrono::DateTime;
use components::separator;
use iced::{
    Alignment, Element, Length, Padding,
    widget::{center, column, container, image, row, scrollable, sensor, space, text, text_input},
};
use matrix::services::{
    room::RoomId,
    timeline::{
        EventTimelineItem, MessageType, MsgLikeKind, TimelineDetails, TimelineItemContent,
        TimelineItemKind, VirtualTimelineItem,
    },
    user::UserId,
};

use crate::screen::home::{FocusedRoom, Image};

use super::{Message, State};

const PROFILE_PICTURE_SIZE: u32 = 32;

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
            )
        }

        let mut previous_sender: Option<UserId> = None;
        for item in &room.timeline.items {
            let mut is_same_sender = false;

            let mut message = match item.kind() {
                TimelineItemKind::Virtual(item) => {
                    previous_sender = None;
                    self.virtual_timeline_item(room, item)
                }
                TimelineItemKind::Event(item) => {
                    is_same_sender = previous_sender.map_or(false, |id| id == item.sender());
                    previous_sender = Some(item.sender().to_owned());

                    let message_content = self.timeline_item(item);
                    let (profile_picture, message_content): (
                        Element<'_, Message>,
                        Element<'_, Message>,
                    ) = if is_same_sender {
                        let space = space::horizontal().width(PROFILE_PICTURE_SIZE).into();
                        (space, message_content)
                    } else {
                        let (username, pfp) = self.get_user_username_and_profile_picture(item);
                        let username = text(username).font(iced::Font {
                            weight: iced::font::Weight::Semibold,
                            ..iced::Font::DEFAULT
                        });

                        let message_content = column![username, message_content].spacing(2.5);
                        (pfp, message_content.into())
                    };

                    row![profile_picture, message_content].spacing(10).into()
                }
            };

            if !is_same_sender {
                message = container(message)
                    .padding(iced::Padding::ZERO.top(10))
                    .into()
            }

            messages = messages.push(message);
        }

        if !room.timeline.hit_end {
            messages = messages.push(
                sensor(space())
                    .anticipate(50)
                    .on_show(|_| Message::PaginateForwards(room_id.clone())),
            )
        }

        let messages = container(messages).padding(iced::Padding::ZERO.bottom(10));
        let messages = scrollable(messages.width(Length::Fill))
            .height(Length::Fill)
            .spacing(5)
            .anchor_bottom();

        let message_input = text_input("Message...", &room.message_draft)
            .on_input(|text| Message::MessageInputChanged(room_id.clone(), text))
            .on_submit(Message::SendMessage(room_id.clone()));

        column![messages, message_input].padding(10).into()
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
                        let mut element = text!("{}", &content.body);
                        if item.is_local_echo() {
                            element = element.style(text::secondary);
                        }
                        element.into()
                    }
                    MessageType::Video(content) => text!("Video: {}", content.filename()).into(),
                    MessageType::VerificationRequest(content) => {
                        text!("Verification request: {}", content.to.to_string()).into()
                    }
                    _ => unreachable!(),
                },
                MsgLikeKind::Sticker(sticker) => {
                    text!("Sticker: {}", sticker.content().body).into()
                }
                MsgLikeKind::Poll(poll_state) => {
                    text!("Poll: {}", poll_state.results().question).into()
                }
                MsgLikeKind::Redacted => {
                    // Redacted messages are often deleted ones or some kind of information
                    // that shouldn't be seen. In the future we could add a setting to whether
                    // or not to see when a redacted message appears, but, for now, we just
                    // omit it.
                    space().into()
                }
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

    fn virtual_timeline_item<'a>(
        &'a self,
        room: &'a FocusedRoom,
        item: &'a VirtualTimelineItem,
    ) -> Element<'a, Message> {
        let mut style: fn(&iced::Theme) -> container::Style = container::secondary;

        let content: Element<'a, Message> = match item {
            VirtualTimelineItem::DateDivider(milliseconds) => {
                let milliseconds = milliseconds.get().into();
                let Some(date) = DateTime::from_timestamp_millis(milliseconds) else {
                    return separator::horizontal().style(style).into();
                };
                let date = date.format("%d/%m/%Y").to_string();
                text(date).size(12).style(text::secondary).into()
            }
            VirtualTimelineItem::ReadMarker => {
                style = container::danger;
                text("New messages").size(12).style(text::danger).into()
            }
            VirtualTimelineItem::TimelineStart => {
                let hit_start_message = text("You've hit the start of the conversation!")
                    .size(12)
                    .style(text::secondary);
                let mut element: Element<'_, Message> = container(hit_start_message)
                    .padding(Padding::ZERO.vertical(10))
                    .into();

                if !room.timeline.hit_start {
                    element = sensor(element)
                        .on_show(|_| Message::TimelineStart(room.id.clone()))
                        .into();
                };

                container(element).center_x(Length::Fill).into()
            }
        };

        row::Row::new()
            .push(separator::horizontal().style(style))
            .push(content)
            .push(separator::horizontal().style(style))
            .spacing(5)
            .align_y(Alignment::Center)
            .into()
    }

    fn get_user_username_and_profile_picture<'a>(
        &'a self,
        item: &'a EventTimelineItem,
    ) -> (String, Element<'a, Message>) {
        let id = item.sender();

        let (username, uri) = match item.sender_profile() {
            TimelineDetails::Ready(profile) => {
                let display_name = match profile.display_name {
                    Some(ref name) if name.len() > 0 => name.to_owned(),
                    _ => id.to_string(),
                };
                let uri = profile.avatar_url.as_ref();
                (display_name, uri)
            }
            _ => (id.to_string(), None),
        };

        let mock_pfp = container(text(username.chars().next().unwrap()))
            .center(PROFILE_PICTURE_SIZE)
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();
                let mut style = container::Style::default();
                style.border = style.border.rounded(100);
                style.background(palette.primary.base.color)
            });

        let pfp = if let Some(img) = self.image_cache.users.get(id) {
            match img {
                Image::Ready(handle) => image(handle)
                    .width(PROFILE_PICTURE_SIZE)
                    .height(PROFILE_PICTURE_SIZE)
                    .border_radius(100)
                    .into(),
                _ => mock_pfp.into(),
            }
        } else if let Some(uri) = uri {
            sensor(mock_pfp)
                .on_show(|_| Message::LoadUserAvatar(id.to_owned(), uri.to_owned()))
                .into()
        } else {
            mock_pfp.into()
        };

        (username, pfp)
    }
}
