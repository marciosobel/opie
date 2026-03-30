use iced::{
    Alignment, Element,
    Length::{self, FillPortion},
    widget::{button, column, container, rich_text, row, space, span, text},
};
use lucide_icons::Icon;
use matrix_sdk::ruma::api::client::device::Device;

use crate::{Action, components::separator};

#[derive(Debug, Clone)]
pub struct State {
    pub open: bool,
    selected_tab: Tab,
    device_state: DeviceState,
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeTab(Tab),
    DeviceList(Vec<Device>),
    OpenUrl(String),
}

#[derive(Debug, Clone)]
pub enum Instruction {
    GetDeviceList,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    Devices,
}

#[derive(Debug, Clone)]
pub enum DeviceState {
    None,
    Loading,
    Ready(Vec<Device>),
}

impl State {
    pub fn new() -> Self {
        Self {
            open: false,
            selected_tab: Tab::Devices,
            device_state: DeviceState::None,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        row![self.tabs(), self.tab_content()].into()
    }

    pub fn tabs(&self) -> Element<'_, Message> {
        let tabs = vec![(Tab::Devices, Icon::Monitor)]
            .into_iter()
            .map(|(tab, icon)| {
                let selected = self.selected_tab == tab;

                let content = row![icon.widget(), text(tab.to_string())]
                    .align_y(Alignment::Center)
                    .spacing(5);

                button(content)
                    .on_press(Message::ChangeTab(tab))
                    .width(Length::Fill)
                    .padding(10)
                    .style(move |theme, status| {
                        let palette = theme.extended_palette();
                        let mut style = button::background(theme, status);
                        style.border = style.border.rounded(0);

                        match status {
                            button::Status::Active if selected => {
                                style.with_background(palette.background.strong.color)
                            }
                            _ => style,
                        }
                    })
                    .into()
            })
            .collect::<Vec<_>>();

        column(tabs)
            .spacing(5)
            .padding(5)
            .width(FillPortion(1))
            .into()
    }

    pub fn tab_content(&self) -> Element<'_, Message> {
        let mut view = column![text(self.selected_tab.to_string()).size(28)]
            .spacing(10)
            .padding(5);

        let tab_content = match self.selected_tab {
            Tab::Devices => self.render_device_tab(),
        };

        view = view.push(tab_content);
        view.height(Length::Fill)
            .width(Length::FillPortion(4))
            .into()
    }

    pub fn update(&mut self, message: Message) -> Action<Instruction, Message> {
        match message {
            Message::ChangeTab(tab) => {
                self.selected_tab = tab;
                match self.selected_tab {
                    Tab::Devices => {
                        if matches!(self.device_state, DeviceState::None) {
                            self.device_state = DeviceState::Loading;
                            return Action::instruction(Instruction::GetDeviceList);
                        }
                    }
                }
            }
            Message::DeviceList(devices) => self.device_state = DeviceState::Ready(devices),
            Message::OpenUrl(url) => {
                _ = open::that(url);
            }
        }

        Action::none()
    }

    fn render_device_tab(&self) -> Element<'_, Message> {
        let device_list: Element<'_, Message> = match &self.device_state {
            DeviceState::None => return text("No action taken").into(),
            DeviceState::Loading => return text("Loading devices...").into(),
            DeviceState::Ready(devices) => {
                let mut device_list = column![].spacing(10);
                let mut devices = devices.iter().peekable();
                while let Some(device) = devices.next() {
                    let device_name = device
                        .display_name
                        .clone()
                        .unwrap_or(String::from("Unknown device"));
                    let device_icon = get_device_icon(&device_name);

                    let device_name = text(device_name).size(16);
                    let device_id = text(device.device_id.to_string()).size(12);
                    let device_icon = container(device_icon.widget().size(20).center())
                        .center(36)
                        .style(|theme| {
                            let palette = theme.extended_palette();
                            let mut style = container::Style::default();
                            style.border = style.border.rounded(100);
                            style.background(palette.background.stronger.color)
                        });

                    let device_info = row![device_icon, column![device_name, device_id]].spacing(5);
                    device_list = device_list.push(container(device_info).padding(5));

                    let is_last = devices.peek().is_none();
                    if !is_last {
                        device_list = device_list.push(separator::horizontal())
                    }
                }

                row![device_list, space::horizontal()].into()
            }
        };

        let device_list = {
            column![
                text("Connected devices").size(24),
                rich_text![
                    "You can manage your devices ",
                    span("here.").link("https://account.matrix.org/account/sessions")
                ]
                .on_link_click(Message::OpenUrl),
                device_list,
            ]
        };
        column![device_list].into()
    }
}

impl std::fmt::Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tab::Devices => write!(f, "Devices"),
        }
    }
}

fn get_device_icon(display_name: &str) -> Icon {
    let name = display_name.to_lowercase();
    if name.contains("web") || name.contains("browser") {
        Icon::AppWindowMac
    } else if name.contains("android") || name.contains("ios") || name.contains("iphone") {
        Icon::Smartphone
    } else {
        Icon::Monitor
    }
}
