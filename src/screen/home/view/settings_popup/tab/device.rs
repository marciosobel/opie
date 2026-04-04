use iced::{
    Alignment, Element, Font, font, padding,
    widget::{button, column, container, rich_text, row, space, span, text},
};
use lucide_icons::Icon;
use matrix_sdk::encryption::identities::Device;

use crate::components::separator;

use super::{super::DeviceState, Message, State, Tab};

impl Tab {
    pub(super) fn render_device_tab(&self, state: &State) -> Element<'_, Message> {
        let device_list: Element<'_, Message> = match &state.device_state {
            DeviceState::None => return text("No action taken").into(),
            DeviceState::Loading => return text("Loading devices...").into(),
            DeviceState::Ready(devices) => {
                let mut device_list = column![];
                let mut devices = devices.iter().peekable();
                while let Some(device) = devices.next() {
                    let name = match device.display_name() {
                        Some(name) => name.to_string(),
                        None => String::from("Unknown device"),
                    };

                    let name = text(name).size(16);
                    let id = text(device.device_id().to_string())
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

                    let badge = {
                        let content = if device.is_verified_with_cross_signing() {
                            row![Icon::CircleCheck.widget(), "Verified"]
                        } else {
                            row![Icon::CircleAlert.widget(), "Not verified"]
                        };
                        container(content).padding(padding::horizontal(10))
                    };

                    let verify_button =
                        button(row![Icon::Shield.widget(), text("Verify")].spacing(5))
                            .on_press(Message::VerifyDevice(device.device_id().to_owned()));

                    let info_text = column![row![name, badge].spacing(5), id].spacing(2.5);
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
        let name = match self.display_name() {
            Some(display_name) => display_name.to_lowercase(),
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
