use super::{Image, Message, State, DEPTH_PADDING, HORIZONTAL_PADDING};
use iced::{
    widget::{
        button, center, column, container, image, row, scrollable, sensor, space, stack, text,
    },
    Alignment, Color, ContentFit, Length, Padding, Theme,
};

use components::{collapsible::Collapsible, separator};
use lucide_icons::Icon;
use matrix::services::Room;

type Element<'a> = iced::Element<'a, Message>;

const SIDEBAR_ROOM_PADDING: Padding = Padding {
    left: HORIZONTAL_PADDING,
    right: HORIZONTAL_PADDING,
    top: 2.5,
    bottom: 2.5,
};
const SIDEBAR_ROOM_AVATAR_SIZE: u32 = 20;
const SIDEBAR_USER_AVATAR_SIZE: u32 = 40;
const COLLAPSIBLE_ARROW_SIZE: f32 = 12.0;
const COLLAPSIBLE_CONTENT_SPACING: f32 = 5.0;

impl State {
    pub(super) fn sidebar(&self) -> Element<'_> {
        let rooms: Element<'_> = if self.is_fetching.rooms {
            center(text("Loading rooms...")).into()
        } else {
            self.spaces()
        };

        let sidebar_column = column![rooms, self.user_info()];

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
        let user = self.own_user();

        let avatar: Element<'_> = match &user.avatar {
            Image::Ready(handle) => image(handle)
                .width(SIDEBAR_USER_AVATAR_SIZE)
                .height(SIDEBAR_USER_AVATAR_SIZE)
                .border_radius(100)
                .into(),
            _ => {
                let placeholder: Element<'_> = match user.display_name() {
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

        let name = match &user.display_name() {
            Some(name) => name.clone(),
            None => "Unknown".to_string(),
        };
        let name = text(name).size(16);
        let id = text(user.id().to_string()).style(text::secondary).size(12);

        let user_info_text = column![name, id].width(Length::Fill);
        let user_info_with_avatar = row![avatar, user_info_text]
            .spacing(10)
            .align_y(Alignment::Center);
        let user_info = button(user_info_with_avatar)
            .on_press(Message::ToggleSettingsPopupOpen)
            .padding(5)
            .style(|theme: &Theme, status: button::Status| {
                let palette = theme.extended_palette();
                let mut style = button::background(theme, status);

                match status {
                    button::Status::Active => {
                        style.background = None;
                    }
                    button::Status::Hovered => {
                        style.text_color = palette.background.strong.text;
                        style = style.with_background(palette.background.strong.color);
                    }
                    button::Status::Pressed => {
                        style.text_color = palette.background.stronger.text;
                        style = style.with_background(palette.background.stronger.color);
                    }
                    _ => {}
                }

                style
            });

        container(user_info).padding(5).into()
    }

    fn space<'a>(&'a self, space: &'a Room, depth: u8) -> Element<'a> {
        let mut content = column![];
        let left_padding = (DEPTH_PADDING * depth as f32) + HORIZONTAL_PADDING;

        let mut child_ids = space.children().iter().collect::<Vec<_>>();
        child_ids.sort_by(|a, b| {
            let room_a = self.rooms.get(*a).expect("Failed to find child room");
            let room_b = self.rooms.get(*b).expect("Failed to find child room");

            // show spaces above normal rooms
            match (room_a.is_space(), room_b.is_space()) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                (_, _) => room_a.cmp(room_b),
            }
        });

        for child_id in child_ids {
            let Some(child) = self.rooms.values().find(|room| room.id() == *child_id) else {
                tracing::error!("Failed to find child room with id {}", child_id);
                continue;
            };

            let element = if child.is_space() {
                self.space(child, depth + 1)
            } else {
                self.room(child, depth + 1)
            };

            content = content.push(element);
        }

        let space_name = match space.display_name() {
            Some(name) => name,
            None => format!("Space {}", space.id()),
        };
        let open = self
            .collapsibles
            .spaces
            .get(&space.id())
            .unwrap_or(&false)
            .to_owned();

        let space_name = text(space_name);
        let space_image = self.sidebar_room_avatar(space, false);
        let open_icon = collapsible_arrow_icon(open);

        let toggler = row![open_icon, space_image, space_name]
            .align_y(Alignment::Center)
            .spacing(COLLAPSIBLE_CONTENT_SPACING);
        let content = indentation_line(content, depth);

        collapsible(toggler, content, open, Message::ToggleSpaceOpen(space.id()))
            .padding(SIDEBAR_ROOM_PADDING.left(left_padding))
            .into()
    }

    fn room<'a>(&'a self, room: &'a Room, depth: u8) -> Element<'a> {
        let room_name = match room.display_name() {
            Some(name) => name,
            None => format!("Room {}", room.id()),
        };
        let focused = self.focused_rooms.contains_key(&room.id());
        let room_name = text(room_name);
        let room_image = self.sidebar_room_avatar(room, focused);

        let content = row![room_image, room_name]
            .align_y(Alignment::Center)
            .spacing(COLLAPSIBLE_CONTENT_SPACING);

        let mut padding = padding_for_depth(depth);
        padding.left -= 3.0; // center room avatar with indentation line

        button(content)
            .on_press(Message::OpenTimeline(room.id()))
            .padding(padding)
            .width(Length::Fill)
            .style(move |theme: &Theme, status| sidebar_room_button_style(theme, status, focused))
            .into()
    }

    fn sidebar_room_avatar<'a>(&'a self, room: &'a Room, focused: bool) -> Element<'a> {
        match self.image_cache.rooms.get(&room.id()) {
            Some(img) => {
                if let Image::Ready(handle) = img {
                    image(handle)
                        .width(SIDEBAR_ROOM_AVATAR_SIZE)
                        .height(SIDEBAR_ROOM_AVATAR_SIZE)
                        .border_radius(100)
                        .content_fit(ContentFit::Cover)
                        .into()
                } else {
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
            }
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
            .filter(|room| room.parents().is_empty() || room.is_direct() || room.is_group())
            .collect::<Vec<_>>();
        root_parents.sort();

        let direct_rooms = root_parents.iter().filter(|room| room.is_direct());
        let dms_open = self.collapsibles.dms;
        let mut dms = column![];
        for dm in direct_rooms {
            dms = dms.push(self.room(dm, 1));
        }

        let dms = collapsible(
            icon_label(Icon::Mail, "Direct Messages", dms_open),
            indentation_line(dms, 0),
            dms_open,
            Message::ToggleDirectMessagesOpen,
        );

        let group_rooms = root_parents.iter().filter(|room| room.is_group());
        let groups_open = self.collapsibles.groups;
        let mut groups = column![];
        for group in group_rooms {
            groups = groups.push(self.room(group, 1));
        }
        let groups = collapsible(
            icon_label(Icon::MessagesSquare, "Groups", groups_open),
            indentation_line(groups, 0),
            groups_open,
            Message::ToggleGroupMessagesOpen,
        );

        let space_rooms = root_parents.iter().filter(|room| room.is_space());
        let mut spaces = column![];
        for space in space_rooms {
            spaces = spaces.push(self.space(space, 0))
        }

        scrollable(column([dms.into(), groups.into(), spaces.into()]))
            .height(Length::Fill)
            .into()
    }
}

fn collapsible<'a>(
    trigger_content: impl Into<Element<'a>>,
    content: impl Into<Element<'a>>,
    open: bool,
    on_toggle: Message,
) -> components::collapsible::Collapsible<'a, Message> {
    Collapsible::new(trigger_content)
        .on_close(on_toggle.clone())
        .on_open(on_toggle)
        .open(open)
        .style(|theme: &Theme, status| sidebar_room_button_style(theme, status, false))
        .padding(SIDEBAR_ROOM_PADDING)
        .width(Length::Fill)
        .content(content)
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

fn icon_label<'a>(icon: Icon, label: &'a str, open: bool) -> Element<'a> {
    row![collapsible_arrow_icon(open), icon.widget(), text(label)]
        .spacing(COLLAPSIBLE_CONTENT_SPACING)
        .align_y(Alignment::Center)
        .into()
}

#[inline]
fn padding_for_depth(depth: u8) -> Padding {
    SIDEBAR_ROOM_PADDING.left((DEPTH_PADDING * depth as f32) + HORIZONTAL_PADDING)
}

fn indentation_line<'a>(base: impl Into<Element<'a>>, depth: u8) -> Element<'a> {
    let line = separator::vertical().style(|theme: &iced::Theme| {
        let palette = theme.extended_palette();
        container::background(palette.background.strongest.color)
    });

    let mut padding = padding_for_depth(depth).vertical(0).right(0);
    padding.left += COLLAPSIBLE_ARROW_SIZE / 2.0;

    let indentation_line = container(line).padding(padding).height(Length::Fill);
    stack![base.into(), indentation_line].into()
}

fn collapsible_arrow_icon<'a>(open: bool) -> Element<'a> {
    let icon = if open {
        Icon::ChevronDown
    } else {
        Icon::ChevronRight
    };

    icon.widget().size(COLLAPSIBLE_ARROW_SIZE).into()
}
