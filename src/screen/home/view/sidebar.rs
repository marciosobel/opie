use super::{DEPTH_PADDING, HORIZONTAL_PADDING, Image, Message, State};
use iced::{
    Alignment, Color, ContentFit, Length, Padding, Theme,
    widget::{button, column, container, image, row, sensor, space, text},
};

use crate::{components::collapsible, matrix::services::Room};

type Element<'a> = iced::Element<'a, Message>;

const SIDEBAR_ROOM_PADDING: Padding = Padding {
    left: HORIZONTAL_PADDING,
    right: HORIZONTAL_PADDING,
    top: 2.5,
    bottom: 2.5,
};
const SIDEBAR_ROOM_AVATAR_SIZE: u32 = 20;
const SIDEBAR_USER_AVATAR_SIZE: u32 = 40;

impl State {
    pub(super) fn sidebar(&self) -> Element<'_> {
        let sidebar_column = column![self.spaces(), space::vertical(), self.user_info(),];

        container(sidebar_column)
            .style(|theme: &iced::Theme| {
                let palette = theme.extended_palette();
                container::Style::default()
                    .background(palette.background.weak.color)
                    .color(palette.background.weak.text)
            })
            .height(Length::Fill)
            .width(300)
            .into()
    }

    fn user_info(&self) -> Element<'_> {
        let avatar: Element<'_> = match &self.user.avatar {
            Image::Ready(handle) => image(handle)
                .width(SIDEBAR_USER_AVATAR_SIZE)
                .height(SIDEBAR_USER_AVATAR_SIZE)
                .border_radius(100)
                .into(),
            Image::None => {
                let placeholder: Element<'_> = match self.user.display_name() {
                    Some(name) if name.len() > 0 => {
                        let first_letter = name.chars().next().unwrap();
                        text(first_letter).size(16).into()
                    }
                    _ => space().into(),
                };

                container(placeholder)
                    .style(move |theme: &Theme| {
                        let palette = theme.extended_palette();
                        let mut style = container::Style::default();
                        style.border = style.border.rounded(100);
                        style
                            .background(palette.background.base.color)
                            .color(palette.background.base.text)
                    })
                    .center(SIDEBAR_USER_AVATAR_SIZE)
                    .clip(true)
                    .into()
            }
        };

        let name = match &self.user.display_name() {
            Some(name) => name.clone(),
            None => "Unknown".to_string(),
        };
        let name = text(name).size(16);
        let id = text(self.user.id().to_string()).size(12);
        let user_info = column![name, id].spacing(5).width(Length::Fill);

        row![avatar, user_info]
            .padding(10)
            .spacing(10)
            .align_y(Alignment::Center)
            .into()
    }

    fn space<'a>(&'a self, space: &'a Room, depth: u8) -> Element<'a> {
        let mut content = column![];

        let mut child_ids = space.children().iter().collect::<Vec<_>>();
        child_ids.sort_by(|a, b| {
            let room_a = self.rooms.get(*a).expect("Failed to find child room");
            let room_b = self.rooms.get(*b).expect("Failed to find child room");

            let name_a = room_a
                .display_name()
                .unwrap_or_else(|| room_a.id().to_string())
                .to_lowercase();
            let name_b = room_b
                .display_name()
                .unwrap_or_else(|| room_b.id().to_string())
                .to_lowercase();
            name_a.cmp(&name_b)
        });

        for child_id in space.children() {
            let Some(child) = self.rooms.values().find(|room| room.id() == *child_id) else {
                tracing::error!("Failed to find child room with id {}", child_id);
                continue;
            };

            if child.is_space() {
                content = content.push(self.space(child, depth + 1));
            } else {
                content = content.push(self.room(child, depth + 1));
            }
        }

        let space_name = match space.display_name() {
            Some(name) => name,
            None => format!("Space {}", space.id()),
        };
        let open = self
            .collapsible_spaces
            .get(&space.id())
            .unwrap_or(&false)
            .to_owned();

        let space_name = text(space_name);
        let space_image = self.sidebar_room_avatar(space, false);

        let toggler = row![space_image, space_name]
            .align_y(Alignment::Center)
            .spacing(5);

        let collapsible = collapsible(toggler)
            .width(Length::Fill)
            .on_close(Message::SpaceClosed(space.id()))
            .on_open(Message::SpaceOpened(space.id()))
            .content(content)
            .open(open)
            .style(move |theme: &Theme, status| sidebar_room_button_style(theme, status, false))
            .padding(Padding {
                left: (DEPTH_PADDING * depth as f32) + HORIZONTAL_PADDING,
                ..SIDEBAR_ROOM_PADDING
            });

        collapsible.into()
    }

    fn room<'a>(&'a self, room: &'a Room, depth: u8) -> Element<'a> {
        let room_name = match room.display_name() {
            Some(name) => name,
            None => format!("Room {}", room.id()),
        };
        let focused = match &self.focused_room {
            Some(id) if *id == room.id() => true,
            _ => false,
        };

        let room_name = text(room_name);
        let room_image = self.sidebar_room_avatar(room, focused);

        let content = row![room_image, room_name]
            .align_y(Alignment::Center)
            .spacing(5);

        button(content)
            .on_press(Message::FocusRoom(room.id()))
            .padding(Padding {
                left: (DEPTH_PADDING * depth as f32) + HORIZONTAL_PADDING,
                ..SIDEBAR_ROOM_PADDING
            })
            .width(Length::Fill)
            .style(move |theme: &Theme, status| sidebar_room_button_style(theme, status, focused))
            .into()
    }

    fn sidebar_room_avatar<'a>(&'a self, room: &'a Room, focused: bool) -> Element<'a> {
        match self.room_avatar_cache.get(&room.id()) {
            Some(img) => match img {
                Image::Ready(handle) => image(handle)
                    .width(SIDEBAR_ROOM_AVATAR_SIZE)
                    .height(SIDEBAR_ROOM_AVATAR_SIZE)
                    .border_radius(100)
                    .content_fit(ContentFit::Cover)
                    .into(),
                Image::None => {
                    let placeholder: Element<'_> = match room.display_name() {
                        Some(name) if name.len() > 0 => {
                            let first_letter = name.chars().next().unwrap();
                            text(first_letter).size(12).into()
                        }
                        _ => space().into(),
                    };

                    container(placeholder)
                        .style(move |theme: &Theme| {
                            let palette = theme.extended_palette();
                            let mut style = container::Style::default();
                            style.border = style.border.rounded(100);
                            if focused {
                                style
                                    .background(palette.primary.base.color)
                                    .color(palette.primary.base.text)
                            } else {
                                style
                                    .background(palette.background.base.color)
                                    .color(palette.background.base.text)
                            }
                        })
                        .center(SIDEBAR_ROOM_AVATAR_SIZE)
                        .clip(true)
                        .into()
                }
            },
            None => {
                let placeholder = container(space())
                    .width(SIDEBAR_ROOM_AVATAR_SIZE)
                    .height(SIDEBAR_ROOM_AVATAR_SIZE)
                    .style(move |theme: &Theme| {
                        let palette = theme.extended_palette().background;
                        let mut style = container::Style::default();
                        style.border = style.border.rounded(100);

                        if focused {
                            style
                                .background(palette.stronger.color)
                                .color(palette.stronger.text)
                        } else {
                            style
                                .background(palette.base.color)
                                .color(palette.base.text)
                        }
                    });

                sensor(placeholder)
                    .key(room.id())
                    .on_show(|_| Message::LoadRoomAvatar(room.id()))
                    .into()
            }
        }
    }

    fn spaces(&self) -> Element<'_> {
        let mut root_parents = self
            .rooms
            .values()
            .filter(|room| room.parents().is_empty() || room.is_direct())
            .collect::<Vec<_>>();

        root_parents.sort_by(|a, b| {
            let name_a = a
                .display_name()
                .unwrap_or_else(|| a.id().to_string())
                .to_lowercase();
            let name_b = b
                .display_name()
                .unwrap_or_else(|| b.id().to_string())
                .to_lowercase();
            name_a.cmp(&name_b)
        });

        let direct_rooms = root_parents.iter().filter(|room| room.is_direct());
        let mut dms = column![];
        for dm in direct_rooms {
            dms = dms.push(self.room(dm, 1));
        }
        let dm_collapsible = collapsible(text("Direct Messages"))
            .on_close(Message::DirectMessagesClosed)
            .on_open(Message::DirectMessagesOpened)
            .open(self.collapsible_dms_open)
            .style(|theme: &Theme, status| sidebar_room_button_style(theme, status, false))
            .padding(SIDEBAR_ROOM_PADDING)
            .width(Length::Fill)
            .content(dms);

        let space_rooms = root_parents.iter().filter(|room| !room.is_direct());
        let mut spaces = column![];
        for space in space_rooms {
            spaces = spaces.push(self.space(space, 0))
        }

        column([dm_collapsible.into(), spaces.into()]).into()
    }
}

fn sidebar_room_button_style(
    theme: &Theme,
    status: button::Status,
    focused: bool,
) -> button::Style {
    let palette = theme.extended_palette().background;
    let mut style = button::Style::default();
    match (status, focused) {
        (button::Status::Active | button::Status::Disabled, false) => {
            style.text_color = palette.base.text;
            style.with_background(Color::TRANSPARENT)
        }
        (button::Status::Hovered, false) => {
            style.text_color = palette.weaker.text;
            style.with_background(palette.weaker.color)
        }
        (button::Status::Pressed, false) => {
            style.text_color = palette.weakest.text;
            style.with_background(palette.weakest.color)
        }
        (button::Status::Active | button::Status::Disabled, true) => {
            style.text_color = palette.strong.text;
            style.with_background(palette.strong.color)
        }
        (button::Status::Hovered, true) => {
            style.text_color = palette.stronger.text;
            style.with_background(palette.stronger.color)
        }
        (button::Status::Pressed, true) => {
            style.text_color = palette.strongest.text;
            style.with_background(palette.strongest.color)
        }
    }
}
