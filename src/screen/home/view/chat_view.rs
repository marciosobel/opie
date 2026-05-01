use chrono::DateTime;
use components::separator;
use iced::{
    Alignment, Element, Length, Padding,
    widget::{center, column, container, image, row, scrollable, sensor, space, text, text_input},
};
use matrix::services::{
    room::RoomId,
    timeline::{
        EventTimelineItem, MessageType, MsgLikeContent, MsgLikeKind, TimelineDetails,
        TimelineItemContent, TimelineItemKind, VirtualTimelineItem,
    },
    user::UserId,
};

use crate::screen::home::{FocusedRoom, Image, User};

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

                    match item.content() {
                        TimelineItemContent::MsgLike(content) => {
                            let message_content = match self.timeline_message(content, item) {
                                Some(element) => element,
                                None => continue,
                            };

                            let (profile_picture, message_content): (
                                Element<'_, Message>,
                                Element<'_, Message>,
                            ) = if is_same_sender {
                                let space = space::horizontal().width(PROFILE_PICTURE_SIZE).into();
                                (space, message_content)
                            } else {
                                let (username, pfp) =
                                    if let Some(profile) = self.users.get(item.sender()) {
                                        self.user_profile(profile)
                                    } else {
                                        self.get_user_profile(item)
                                    };

                                let username = text(username).font(iced::Font {
                                    weight: iced::font::Weight::Semibold,
                                    ..iced::Font::DEFAULT
                                });

                                let message_content =
                                    column![username, message_content].spacing(2.5);
                                (pfp, message_content.into())
                            };

                            row![profile_picture, message_content].spacing(10).into()
                        }
                        _ => {
                            previous_sender = None; // Not a user message
                            match self.timeline_event(item) {
                                Some(element) => element,
                                None => continue,
                            }
                        }
                    }
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

    fn timeline_event<'a>(&'a self, item: &'a EventTimelineItem) -> Option<Element<'a, Message>> {
        let content = match item.content() {
            TimelineItemContent::MembershipChange(member) => {
                text!("{}", member.user_id().to_string()).into()
            }
            TimelineItemContent::ProfileChange(member) => {
                let value = if let Some(_) = member.avatar_url_change() {
                    let name = if let Some(profile) = self.users.get(member.user_id()) {
                        profile.display_name_or_id()
                    } else {
                        member.user_id().to_string()
                    };

                    format!("{} changed avatar", name)
                } else if let Some(_) = member.displayname_change() {
                    let name = if let Some(profile) = self.users.get(member.user_id()) {
                        profile.display_name_or_id()
                    } else {
                        member.user_id().to_string()
                    };

                    format!("{} changed their display name", name)
                } else {
                    return None;
                };

                container(text!("{}", value).size(12).style(text::secondary))
                    .center_x(Length::Fill)
                    .into()
            }
            TimelineItemContent::FailedToParseMessageLike { error, .. } => {
                text!("Failed to parse messagelike: {:#?}", error).into()
            }
            TimelineItemContent::FailedToParseState { error, .. } => {
                text!("Failed to parse state: {:#?}", error).into()
            }
            TimelineItemContent::CallInvite => text("Call invite").style(text::secondary).into(),
            _ => return None,
        };

        Some(content)
    }

    fn timeline_message<'a>(
        &'a self,
        content: &'a MsgLikeContent,
        item: &'a EventTimelineItem,
    ) -> Option<Element<'a, Message>> {
        let content = match &content.kind {
            MsgLikeKind::Message(message) => match message.msgtype() {
                MessageType::Audio(content) => text!("Audio: {}", content.filename()).into(),
                MessageType::Emote(content) => text!("Emote: {}", &content.body).into(),
                MessageType::File(content) => text!("File: {}", content.filename()).into(),
                MessageType::Image(content) => {
                    let Some(id) = item.event_id() else {
                        let content = text("Failed to retrieve event id for this item")
                            .style(text::danger)
                            .into();
                        return Some(content);
                    };
                    if let Some(img) = self.image_cache.timeline.get(id) {
                        match img {
                            Image::Ready(handle) => container(image(handle)).max_height(350).into(),
                            Image::Fetching => {
                                let icon =
                                    lucide_icons::Icon::Image.widget().style(text::secondary);
                                let text = text("Loading image...");
                                row![icon, text]
                                    .spacing(10)
                                    .align_y(Alignment::Center)
                                    .into()
                            }
                            Image::None => {
                                let icon =
                                    lucide_icons::Icon::ImageOff.widget().style(text::secondary);
                                let text = text("Failed to load image");
                                row![icon, text]
                                    .spacing(10)
                                    .align_y(Alignment::Center)
                                    .into()
                            }
                        }
                    } else {
                        let icon = lucide_icons::Icon::Image.widget().style(text::secondary);
                        let text = text("Loading image...").style(text::secondary);
                        let element = row![icon, text].spacing(10).align_y(Alignment::Center);

                        sensor(element)
                            .on_show(|_| {
                                Message::FetchTimelineImage(id.to_owned(), content.source.clone())
                            })
                            .into()
                    }
                }
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
            MsgLikeKind::Sticker(sticker) => text!("Sticker: {}", sticker.content().body).into(),
            MsgLikeKind::Poll(poll_state) => {
                text!("Poll: {}", poll_state.results().question).into()
            }
            MsgLikeKind::Redacted => {
                // Redacted messages are often deleted ones or some kind of information
                // that shouldn't be seen. In the future we could add a setting to whether
                // or not to see when a redacted message appears, but, for now, we just
                // omit it.
                return None;
            }
            MsgLikeKind::UnableToDecrypt(_) => text!("Encrypted message").into(),
            MsgLikeKind::Other(message) => text!("Unknown message type: {:#?}", message).into(),
        };

        Some(content)
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
                    .align_x(Alignment::Center)
                    .wrapping(text::Wrapping::None)
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
            .padding(20)
            .align_y(Alignment::Center)
            .into()
    }

    fn get_user_profile<'a>(
        &'a self,
        item: &'a EventTimelineItem,
    ) -> (String, Element<'a, Message>) {
        let id = item.sender();

        let username = match item.sender_profile() {
            TimelineDetails::Ready(profile) => match profile.display_name {
                Some(ref name) if name.len() > 0 => name.to_owned(),
                _ => id.to_string(),
            },
            _ => id.to_string(),
        };

        let mock_pfp = sensor(mock_pfp(username.chars().next().unwrap()))
            .on_show(|_| Message::GetUser(id.to_owned()))
            .into();
        (username, mock_pfp)
    }

    fn user_profile<'a>(&'a self, user: &'a User) -> (String, Element<'a, Message>) {
        let username = user.display_name_or_id();
        let username_char = username.chars().next().unwrap();

        let pfp = match user.avatar() {
            Image::Ready(handle) => image(handle)
                .border_radius(100)
                .width(PROFILE_PICTURE_SIZE)
                .height(PROFILE_PICTURE_SIZE)
                .into(),
            Image::Fetching => mock_pfp(username_char),
            Image::None => mock_pfp(username_char),
        };

        (username, pfp)
    }
}

fn mock_pfp<'a, S: text::IntoFragment<'a>>(name: S) -> Element<'a, Message> {
    container(text(name))
        .center(PROFILE_PICTURE_SIZE)
        .style(|theme: &iced::Theme| {
            let palette = theme.extended_palette();
            let mut style = container::Style::default();
            style.border = style.border.rounded(100);
            style.background(palette.primary.base.color)
        })
        .into()
}
