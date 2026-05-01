use iced::{
    Alignment, Element,
    Length::{self, FillPortion},
    widget::{button, column, row, text},
};
use lucide_icons::Icon;
use matrix::services::device::{Device, DeviceId};

use crate::Action;

mod tab;
use tab::Tab;

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
    VerifyDevice(DeviceId),
    RefreshDeviceList,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    GetDeviceList,
    VerifyDevice(DeviceId),
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
                                style.with_background(palette.background.neutral.color)
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
        let tab_title = text(self.selected_tab.to_string()).size(28);
        let mut view = column![tab_title];

        let tab = self.selected_tab.render(self);
        view = view.push(tab);
        view.height(Length::Fill)
            .width(Length::FillPortion(4))
            .spacing(10)
            .padding(10)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Action<Instruction, Message> {
        match message {
            Message::ChangeTab(tab) => {
                self.selected_tab = tab;
            }
            Message::DeviceList(devices) => self.device_state = DeviceState::Ready(devices),
            Message::OpenUrl(url) => {
                _ = open::that(url);
            }
            Message::VerifyDevice(id) => {
                return Action::instruction(Instruction::VerifyDevice(id));
            }
            Message::RefreshDeviceList => {
                self.device_state = DeviceState::Loading;
                return Action::instruction(Instruction::GetDeviceList);
            }
        }

        Action::none()
    }
}
