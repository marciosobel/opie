use components::separator;
use iced::{
    Alignment, Element, Font, font,
    widget::{button, column, container, rich_text, row, space, span, text},
};
use lucide_icons::Icon;
use matrix::services::Device;

use super::{super::DeviceState, Message, State, Tab};

impl Tab {
    pub(super) fn render_device_tab(state: &State) -> Element<'_, Message> {
        device_list(&state.device_state)
    }
}

fn device_list(state: &DeviceState) -> Element<'_, Message> {
    let connected_devices: Element<'_, Message> = {
        let refresh_button = button(
            row![Icon::RefreshCcw.widget().size(12), text("Refresh")]
                .spacing(5)
                .align_y(Alignment::Center),
        )
        .on_press(Message::RefreshDeviceList);

        let manage_devices_text = rich_text![
            "You can manage your devices ",
            span("here.")
                .font(Font {
                    weight: font::Weight::Bold,
                    ..Font::DEFAULT
                })
                .size(16)
                .link("https://account.matrix.org/account/sessions")
        ]
        .on_link_click(Message::OpenUrl);

        let header = column![text("Connected devices").size(24), manage_devices_text].spacing(5);
        if matches!(state, DeviceState::Ready(_)) {
            row![header, space::horizontal(), refresh_button].into()
        } else {
            header.into()
        }
    };

    let device_list: Element<'_, Message> = match &state {
        DeviceState::None => text("No action taken").into(),
        DeviceState::Loading => text("Loading devices...").into(),
        DeviceState::Ready(devices) => {
            let mut device_list = column![];

            if let Some(device) = devices.iter().find(|device| device.is_self) {
                device_list = device_list
                    .push(text("This device").size(16).font(Font {
                        weight: font::Weight::Semibold,
                        ..Font::DEFAULT
                    }))
                    .push(device_list_item(device))
                    .push(space::vertical().height(10));
            }

            device_list = device_list.push(text("Other devices").size(16).font(Font {
                weight: font::Weight::Semibold,
                ..Font::DEFAULT
            }));

            let mut devices = devices.iter().filter(|device| !device.is_self).peekable();
            while let Some(device) = devices.next() {
                let verify_button = button(row![Icon::Shield.widget(), text("Verify")].spacing(5))
                    .on_press(Message::VerifyDevice(device.id.clone()));

                device_list = device_list.push(
                    row![device_list_item(device), space::horizontal(), verify_button]
                        .align_y(Alignment::Center),
                );

                let is_last = devices.peek().is_none();
                if !is_last {
                    device_list = device_list.push(separator::horizontal())
                }
            }

            device_list.spacing(10).into()
        }
    };

    column![connected_devices, device_list].spacing(10).into()
}

fn device_list_item(device: &Device) -> Element<'_, Message> {
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

    let badge = {
        let icon = if device.verified {
            Icon::CircleCheck.widget().style(text::success)
        } else {
            Icon::CircleAlert.widget().style(text::danger)
        };

        icon.size(14)
    };

    let info = column![row![name, badge].spacing(5).align_y(Alignment::Center), id].spacing(2.5);
    row![icon, info].spacing(10).into()
}
