use iced::{
    Alignment, Element,
    Length::{self, FillPortion},
    widget::{button, center, column, row, text},
};
use lucide_icons::Icon;

use crate::Action;

#[derive(Debug, Clone)]
pub struct State {
    pub open: bool,
    pub selected_tab: Tab,
}

#[derive(Debug, Clone)]
pub enum Message {
    ChangeTab(Tab),
}

#[derive(Debug, Clone)]
pub enum Instruction {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    Devices,
}

impl State {
    pub fn new() -> Self {
        Self {
            open: false,
            selected_tab: Tab::Devices,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        row![self.tabs(), self.main_view()].into()
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

    pub fn main_view(&self) -> Element<'_, Message> {
        center(text(self.selected_tab.to_string()))
            .width(FillPortion(4))
            .into()
    }

    pub fn update(&mut self, message: Message) -> Action<Instruction, Message> {
        match message {
            Message::ChangeTab(tab) => self.selected_tab = tab,
        }

        Action::none()
    }
}

impl std::fmt::Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tab::Devices => write!(f, "Devices"),
        }
    }
}
