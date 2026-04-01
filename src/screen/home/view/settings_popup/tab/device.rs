use iced::{
    Element, Font, font,
    widget::{column, container, rich_text, row, space, span, text},
};
use lucide_icons::Icon;
use matrix_sdk::ruma::api::client::device::Device;

use crate::components::separator;

use super::{super::DeviceState, Message, State, Tab};

impl Tab {
    pub(super) fn render_device_tab(&self, state: &State) -> Element<'_, Message> {
        let device_list: Element<'_, Message> = match &state.device_state {
            DeviceState::None => return text("No action taken").into(),
            DeviceState::Loading => return text("Loading devices...").into(),
            DeviceState::Ready(devices) => {
                let mut device_list = column![].spacing(10);
                let mut devices = devices.iter().peekable();
                while let Some(device) = devices.next() {
                    let name = device
                        .display_name
                        .clone()
                        .unwrap_or(String::from("Unknown device"));

                    let name = text(name).size(16);
                    let id = text(device.device_id.to_string())
                        .size(12)
                        .style(text::secondary);
                    let icon = container(device.icon().widget().size(20).center())
                        .center(36)
                        .style(|theme| {
                            let palette = theme.extended_palette();
                            let mut style = container::Style::default();
                            style.border = style.border.rounded(100);
                            style.background(palette.background.stronger.color)
                        });

                    let info = row![icon, column![name, id].spacing(2.5)].spacing(10);
                    device_list = device_list.push(container(info).padding(5));

                    let is_last = devices.peek().is_none();
                    if !is_last {
                        device_list = device_list.push(separator::horizontal())
                    }
                }

                row![device_list, space::horizontal()].into()
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
                            ..Default::default()
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

trait DeviceExt {
    fn icon(&self) -> Icon;
}

impl DeviceExt for Device {
    fn icon(&self) -> Icon {
        let name = match self.display_name {
            Some(ref display_name) => display_name.to_lowercase(),
            None => return Icon::Box,
        };

        if name.contains("web") || name.contains("browser") {
            Icon::AppWindowMac
        } else if name.contains("android") || name.contains("ios") || name.contains("iphone") {
            Icon::Smartphone
        } else {
            Icon::Monitor
        }
    }
}
