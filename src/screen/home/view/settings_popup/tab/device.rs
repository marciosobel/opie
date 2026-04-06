use iced::{
    Alignment, Element, Font, font,
    widget::{button, column, container, rich_text, row, space, span, text},
};
use lucide_icons::Icon;

use crate::components::separator;

use super::{super::DeviceState, Message, State, Tab};

impl Tab {
    pub(super) fn render_device_tab<'a>(&'a self, state: &'a State) -> Element<'a, Message> {
        let device_list: Element<'_, Message> = match &state.device_state {
            DeviceState::None => return text("No action taken").into(),
            DeviceState::Loading => return text("Loading devices...").into(),
            DeviceState::Ready(devices) => {
                let mut device_list = column![];
                let mut devices = devices.iter().peekable();
                while let Some(device) = devices.next() {
                    let name = match &device.display_name {
                        Some(name) => name.to_string(),
                        None => String::from("Unknown device"),
                    };

                    let name = text(name).size(16);
                    let id = text(device.id.to_string()).size(12).style(text::secondary);
                    let icon = container(device.kind.widget().size(20).center())
                        .center(36)
                        .style(|theme| {
                            let palette = theme.extended_palette();
                            let mut style = container::Style::default();
                            style.border = style.border.rounded(100);
                            style.background(palette.background.stronger.color)
                        });

                    let badge = verified_badge(device.verified);

                    let verify_button =
                        button(row![Icon::Shield.widget(), text("Verify")].spacing(5))
                            .on_press(Message::VerifyDevice(device.id.clone()));

                    let info_text =
                        column![row![name, badge].spacing(5).align_y(Alignment::Center), id]
                            .spacing(2.5);
                    let info = row![icon, info_text].spacing(10);

                    device_list = device_list.push(
                        row![info, space::horizontal(), verify_button]
                            .align_y(Alignment::Center)
                            .padding(5),
                    );

                    let is_last = devices.peek().is_none();
                    if !is_last {
                        device_list = device_list.push(separator::horizontal())
                    }
                }

                device_list.spacing(10).into()
            }
        };

        column![
            column![
                text("Connected devices").size(24),
                rich_text![
                    "You can manage your devices ",
                    span("here.")
                        .font(Font {
                            weight: font::Weight::Bold,
                            ..Font::DEFAULT
                        })
                        .size(16)
                        .link("https://account.matrix.org/account/sessions")
                ]
                .on_link_click(Message::OpenUrl),
            ]
            .spacing(5),
            device_list,
        ]
        .spacing(10)
        .into()
    }
}

fn verified_badge<'a, Message: 'a>(verified: bool) -> Element<'a, Message> {
    let badge = if verified {
        Icon::CircleCheck.widget().style(text::success)
    } else {
        Icon::CircleAlert.widget().style(text::danger)
    };

    badge.size(14).into()
}
