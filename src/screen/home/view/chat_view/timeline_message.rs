use crate::screen::home::{FocusedRoom, Image, Message, State, User};

use chrono::DateTime;
use components::separator;
use iced::{
    Alignment, Element, Length, Padding,
    widget::{column, container, image, row, sensor, space, text},
};
use lucide_icons::Icon;
use matrix::services::{
    timeline::{
        EventTimelineItem, MembershipChange, MessageType, MsgLikeContent, MsgLikeKind,
        TimelineDetails, TimelineItemContent, VirtualTimelineItem,
    },
    user::UserId,
};

const PROFILE_PICTURE_SIZE: u32 = 32;

/// An enum representing different timeline message kinds.
pub enum TimelineMessage<'a> {
    /// A user message.
    User(Element<'a, Message>),
    /// A virtual message, may be a notice, event or notification.
    Virtual(Element<'a, Message>),
    /// An item that should not be rendered.
    Blank,
}

impl<'a> TimelineMessage<'a> {
    /// Renders a virtual timeline item
    pub fn from_virtual(item: &'a VirtualTimelineItem, room: &'a FocusedRoom) -> Self {
        let mut style: fn(&iced::Theme) -> container::Style = container::secondary;

        let content: Element<'a, Message> = match item {
            VirtualTimelineItem::DateDivider(milliseconds) => {
                let milliseconds = milliseconds.get().into();
                if let Some(date) = DateTime::from_timestamp_millis(milliseconds) {
                    let date = date.format("%d/%m/%Y").to_string();
                    text(date).size(12).style(text::secondary).into()
                } else {
                    separator::horizontal().style(style).into()
                }
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

        let item = row::Row::new()
            .push(separator::horizontal().style(style))
            .push(content)
            .push(separator::horizontal().style(style))
            .spacing(5)
            .padding(10)
            .align_y(Alignment::Center);

        Self::Virtual(item.into())
    }

    /// Renders an event timeline item
    pub fn from_event(
        item: &'a EventTimelineItem,
        state: &'a State,
        render_profile_picture: bool,
    ) -> Self {
        match item.content() {
            TimelineItemContent::MembershipChange(member) => {
                let Some(change) = member.change() else {
                    return Self::Blank;
                };

                let id = member.user_id();
                let username = if let Some(user) = state.users.get(id) {
                    user.display_name_or_id()
                } else {
                    id.to_string()
                };

                let message = match change {
                    MembershipChange::None => return Self::Blank,
                    MembershipChange::Error => return Self::Blank,
                    MembershipChange::NotImplemented => return Self::Blank,
                    MembershipChange::Joined => text!("{} joined the room", username),
                    MembershipChange::Left => text!("{} left the room", username),
                    MembershipChange::Banned => text!("{} has been banned", username),
                    MembershipChange::Unbanned => text!("{} has been unbanned", username),
                    MembershipChange::Kicked => text!("{} has been kicked", username),
                    MembershipChange::Invited => text!("{} was invited to the room", username),
                    MembershipChange::KickedAndBanned => {
                        text!("{} was kicked and banned from the room", username)
                    }
                    MembershipChange::InvitationAccepted => {
                        text!("{} accepted the invite", username)
                    }
                    MembershipChange::InvitationRejected => {
                        text!("{} rejected the invite", username)
                    }
                    MembershipChange::InvitationRevoked => {
                        text!("{} invitation has been revoked", username)
                    }
                    MembershipChange::Knocked => text!("{} knocked on the room", username),
                    MembershipChange::KnockAccepted => text!("{} knock was accepted", username),
                    MembershipChange::KnockRetracted => text!("{} retracted their knock", username),
                    MembershipChange::KnockDenied => text!("{} knock was denied", username),
                };

                let message =
                    container(message.style(text::secondary).size(12)).center_x(Length::Fill);
                Self::Virtual(message.into())
            }
            TimelineItemContent::ProfileChange(member) => {
                let value = if let Some(_) = member.avatar_url_change() {
                    let name = if let Some(profile) = state.users.get(member.user_id()) {
                        profile.display_name_or_id()
                    } else {
                        member.user_id().to_string()
                    };

                    format!("{} changed avatar", name)
                } else if let Some(_) = member.displayname_change() {
                    let name = if let Some(profile) = state.users.get(member.user_id()) {
                        profile.display_name_or_id()
                    } else {
                        member.user_id().to_string()
                    };

                    format!("{} changed their display name", name)
                } else {
                    return Self::Blank;
                };

                let content =
                    container(text(value).size(12).style(text::secondary)).center_x(Length::Fill);
                Self::Virtual(content.into())
            }
            TimelineItemContent::FailedToParseMessageLike { error, .. } => {
                let content = text!("Failed to parse messagelike: {:#?}", error);
                Self::User(content.into())
            }
            TimelineItemContent::FailedToParseState { error, .. } => {
                let content = text!("Failed to parse state: {:#?}", error);
                Self::User(content.into())
            }
            TimelineItemContent::CallInvite => {
                let username = state.get_username_of(item.sender().to_owned());
                let content = text!("{} started a call", username).style(text::secondary);
                Self::Virtual(content.into())
            }
            TimelineItemContent::MsgLike(content) => {
                let msglike = Self::msglike(content, item, state);
                let Self::User(content) = msglike else {
                    return msglike;
                };

                let (profile_picture, content): (Element<'_, Message>, Element<'_, Message>) =
                    if !render_profile_picture {
                        let space = space::horizontal().width(PROFILE_PICTURE_SIZE).into();
                        (space, content)
                    } else {
                        let (username, pfp) = if let Some(profile) = state.users.get(item.sender())
                        {
                            user_profile(profile)
                        } else {
                            get_user_profile(item)
                        };

                        let username = text(username)
                            .font(iced::Font {
                                weight: iced::font::Weight::Semibold,
                                ..iced::Font::DEFAULT
                            })
                            .align_y(Alignment::Center);

                        let timestamp: Element<'_, Message> = {
                            let millis = item.timestamp().get().into();
                            if let Some(date) = DateTime::from_timestamp_millis(millis) {
                                let time = date.format("%H:%M").to_string();
                                text(time).size(12).style(text::secondary).into()
                            } else {
                                space().into()
                            }
                        };

                        let heading = row![username, timestamp]
                            .spacing(5)
                            .align_y(Alignment::Center);

                        let message_content = column![heading, content].spacing(2.5);
                        (pfp, message_content.into())
                    };

                let content = row![profile_picture, content].spacing(10);
                Self::User(content.into())
            }
            _ => Self::Blank,
        }
    }

    /// Renders a msglike timeline item
    fn msglike(content: &'a MsgLikeContent, item: &'a EventTimelineItem, state: &'a State) -> Self {
        let message = match &content.kind {
            MsgLikeKind::Sticker(sticker) => text!("Sticker: {}", sticker.content().body).into(),
            MsgLikeKind::Poll(poll_state) => {
                text!("Poll: {}", poll_state.results().question).into()
            }
            MsgLikeKind::Redacted => {
                // Redacted messages are often deleted ones or some kind of information
                // that shouldn't be seen. In the future we could add a setting to whether
                // or not to see when a redacted message appears, but, for now, we just
                // omit it.
                return Self::Blank;
            }
            MsgLikeKind::UnableToDecrypt(_) => text!("Encrypted message").into(),
            MsgLikeKind::Other(message) => text!("Unknown message type: {:#?}", message).into(),
            MsgLikeKind::Message(message) => match message.msgtype() {
                MessageType::Audio(content) => text!("Audio: {}", content.filename()).into(),
                MessageType::Emote(content) => text!("Emote: {}", &content.body).into(),
                MessageType::File(content) => text!("File: {}", content.filename()).into(),
                MessageType::Image(content) => {
                    let Some(id) = item.event_id() else {
                        return Self::User(image_status(ImageStatus::Failed));
                    };

                    if let Some(img) = state.image_cache.timeline.get(id) {
                        match img {
                            Image::Ready(handle) => container(image(handle))
                                .padding(Padding::ZERO.bottom(5))
                                .max_height(350)
                                .into(),
                            Image::Fetching => image_status(ImageStatus::Loading),
                            Image::None => image_status(ImageStatus::Failed),
                        }
                    } else {
                        sensor(image_status(ImageStatus::Loading))
                            .key(id.to_owned())
                            .anticipate(250)
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
                    let mut element = text(&content.body);
                    if item.is_local_echo() {
                        element = element.style(text::secondary);
                    }
                    element.into()
                }
                MessageType::Video(content) => text!("Video: {}", content.filename()).into(),
                MessageType::VerificationRequest(content) => {
                    let user_from = state.get_username_of(item.sender().to_owned());
                    let user_to = state.get_username_of(content.to.to_owned());
                    let content =
                        text!("{} sent a verification request for {}", user_from, user_to)
                            .size(12)
                            .style(text::secondary);
                    return Self::Virtual(content.into());
                }
                _ => unreachable!(),
            },
        };

        Self::User(message)
    }
}

fn get_user_profile(item: &EventTimelineItem) -> (String, Element<'_, Message>) {
    let id = item.sender();

    let username = match item.sender_profile() {
        TimelineDetails::Ready(profile) => match &profile.display_name {
            Some(name) => name.to_owned(),
            None => id.to_string(),
        },
        _ => id.to_string(),
    };

    let mock_pfp = sensor(mock_pfp(username.chars().next().unwrap()))
        .anticipate(1000)
        .key(id.to_owned())
        .on_show(|_| Message::GetUser(id.to_owned()))
        .into();
    (username, mock_pfp)
}

fn user_profile(user: &User) -> (String, Element<'_, Message>) {
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

enum ImageStatus {
    Loading,
    Failed,
}

fn image_status<'a>(status: ImageStatus) -> Element<'a, Message> {
    let (content, icon) = match status {
        ImageStatus::Loading => ("Loading image...", Icon::Image),
        ImageStatus::Failed => ("Failed to load image", Icon::ImageOff),
    };

    let icon = icon.widget().style(text::secondary);
    let text = text(content).style(text::secondary);
    row![icon, text]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
}

trait StateExt {
    fn get_username_of(&self, id: UserId) -> String;
}

impl StateExt for State {
    fn get_username_of(&self, id: UserId) -> String {
        match self.users.get(&id) {
            Some(profile) => profile.display_name_or_id(),
            None => id.to_string(),
        }
    }
}
