use std::{collections::HashMap, sync::Arc};

use iced::{
    Alignment, Color, ContentFit, Element, Length, Padding, Theme,
    widget::{button, center, column, container, image, row, sensor, space, text},
};
use matrix_sdk::ruma::OwnedRoomId;

use crate::{
    Action,
    components::collapsible,
    matrix::{
        bridge::{Action as MatrixAction, Event as MatrixEvent, MatrixBridgeSender},
        services::Room,
    },
};

const DEPTH_PADDING: f32 = 24.0;
const HORIZONTAL_PADDING: f32 = 5.0;
const SIDEBAR_ROOM_PADDING: Padding = Padding {
    left: HORIZONTAL_PADDING,
    right: HORIZONTAL_PADDING,
    top: 2.5,
    bottom: 2.5,
};
const SIDEBAR_ROOM_AVATAR_SIZE: u32 = 20;

#[derive(Debug, Clone)]
pub struct State {
    // bridge: MatrixBridgeSender,
    rooms: Arc<HashMap<OwnedRoomId, Room>>,
    collapsible_spaces: HashMap<OwnedRoomId, bool>,
    collapsible_dms_open: bool,
    focused_room: Option<OwnedRoomId>,
    room_avatar_cache: HashMap<OwnedRoomId, Image>,
}

#[derive(Debug, Clone)]
pub enum Message {
    MatrixEvent(MatrixEvent),
    SpaceOpened(OwnedRoomId),
    SpaceClosed(OwnedRoomId),
    DirectMessagesOpened,
    DirectMessagesClosed,
    FocusRoom(OwnedRoomId),
    LoadRoomAvatar(OwnedRoomId),
}

#[derive(Debug, Clone)]
pub enum Instruction {}

impl State {
    pub fn new(mut bridge: MatrixBridgeSender) -> Self {
        _ = bridge.try_send(MatrixAction::ListAllRooms);

        Self {
            // bridge,
            rooms: Arc::default(),
            collapsible_spaces: HashMap::new(),
            collapsible_dms_open: true,
            focused_room: None,
            room_avatar_cache: HashMap::new(),
        }
    }

    pub fn update(&mut self, message: Message) -> Action<Instruction, Message> {
        match message {
            Message::MatrixEvent(event) => {
                return self.matrix_event(event);
            }
            Message::SpaceOpened(id) => {
                self.collapsible_spaces.insert(id, true);
            }
            Message::SpaceClosed(id) => {
                self.collapsible_spaces.insert(id, false);
            }
            Message::DirectMessagesOpened => self.collapsible_dms_open = true,
            Message::DirectMessagesClosed => self.collapsible_dms_open = false,
            Message::FocusRoom(id) => self.focused_room = Some(id),
            Message::LoadRoomAvatar(id) => {
                let Some(room) = self.rooms.get(&id) else {
                    tracing::error!(
                        "Received LoadRoomAvatar for room id {} but it was not found in the rooms list",
                        id
                    );
                    return Action::none();
                };

                let image = match room.avatar() {
                    Some(bytes) => {
                        let handle = image::Handle::from_bytes(bytes.clone());
                        Image::Ready(handle)
                    }
                    None => Image::None,
                };

                self.room_avatar_cache.insert(id, image);
            }
        }

        Action::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        row![self.sidebar(), self.main_view()].into()
    }

    pub fn main_view(&self) -> Element<'_, Message> {
        if let Some(room_id) = &self.focused_room {
            let room = self.rooms.get(room_id).expect(&format!(
                "Something went wrong getting the room with id {}",
                room_id
            ));

            let name = room.display_name().unwrap_or("Unknown".into());
            center(text!("Focused on room {}", name)).into()
        } else {
            center(text("Welcome to the chat screen")).into()
        }
    }

    pub fn sidebar(&self) -> Element<'_, Message> {
        let mut sidebar = column![];

        let root_parents = self
            .rooms
            .values()
            .filter(|room| room.parents().is_empty() || room.is_direct())
            .collect::<Vec<_>>();

        let direct_rooms = root_parents.iter().filter(|room| room.is_direct());
        let mut dms = column![];
        for dm in direct_rooms {
            dms = dms.push(self.render_room(dm, 1));
        }
        let dm_collapsible = collapsible(text("Direct Messages"))
            .on_close(Message::DirectMessagesClosed)
            .on_open(Message::DirectMessagesOpened)
            .open(self.collapsible_dms_open)
            .style(|theme: &Theme, status| sidebar_room_button_style(theme, status, false))
            .padding(SIDEBAR_ROOM_PADDING)
            .width(Length::Fill)
            .content(dms);
        sidebar = sidebar.push(dm_collapsible);

        let space_rooms = root_parents.iter().filter(|room| !room.is_direct());
        let mut spaces = column![];
        for space in space_rooms {
            spaces = spaces.push(self.render_space(space, 0))
        }
        sidebar = sidebar.push(spaces);

        container(sidebar)
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

    fn render_space<'a>(&'a self, space: &'a Room, depth: u8) -> Element<'a, Message> {
        let mut content = column![];

        for child_id in space.children() {
            let Some(child) = self.rooms.values().find(|room| room.id() == *child_id) else {
                tracing::error!("Failed to find child room with id {}", child_id);
                continue;
            };

            if child.is_space() {
                content = content.push(self.render_space(child, depth + 1));
            } else {
                content = content.push(self.render_room(child, depth + 1));
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
        let space_image = self.render_sidebar_room_avatar(space, false);

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

    fn render_room<'a>(&'a self, room: &'a Room, depth: u8) -> Element<'a, Message> {
        let room_name = match room.display_name() {
            Some(name) => name,
            None => format!("Room {}", room.id()),
        };
        let focused = match &self.focused_room {
            Some(id) if *id == room.id() => true,
            _ => false,
        };

        let room_name = text(room_name);
        let room_image: Element<'_, Message> = self.render_sidebar_room_avatar(room, focused);

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

    fn matrix_event(&mut self, event: MatrixEvent) -> Action<Instruction, Message> {
        match event {
            MatrixEvent::RoomList(rooms) => self.rooms = rooms,
            _ => {}
        }
        Action::none()
    }

    fn render_sidebar_room_avatar<'a>(
        &'a self,
        room: &'a Room,
        focused: bool,
    ) -> Element<'a, Message> {
        match self.room_avatar_cache.get(&room.id()) {
            Some(img) => match img {
                Image::Ready(handle) => image(handle)
                    .width(SIDEBAR_ROOM_AVATAR_SIZE)
                    .height(SIDEBAR_ROOM_AVATAR_SIZE)
                    .border_radius(100)
                    .content_fit(ContentFit::Cover)
                    .into(),
                Image::None => {
                    let placeholder: Element<'_, Message> = match room.display_name() {
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
                let placeholder = container(
                    space()
                        .width(SIDEBAR_ROOM_AVATAR_SIZE)
                        .height(SIDEBAR_ROOM_AVATAR_SIZE),
                )
                .style(move |theme: &Theme| {
                    let palette = theme.extended_palette().background;
                    let style = container::Style::default();
                    if focused {
                        style
                            .background(palette.strong.color)
                            .color(palette.strong.text)
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

#[derive(Debug, Clone)]
enum Image {
    Ready(image::Handle),
    None,
}
