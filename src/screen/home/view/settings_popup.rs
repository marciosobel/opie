use iced::{
    Element,
    Length::{self, FillPortion},
    widget::{button, center, column, row, text},
};

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

#[derive(Debug, Clone)]
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
        let tabs = Tab::list().into_iter().map(|tab| {
            button(text(tab.to_string()))
                .width(Length::Fill)
                .padding(0)
                .on_press(Message::ChangeTab(tab))
                .into()
        });

        column(tabs).width(FillPortion(1)).into()
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

impl Tab {
    /// Returns a list of tabs to be displayed in the application
    pub fn list() -> Vec<Tab> {
        vec![Tab::Devices]
    }
}

impl std::fmt::Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tab::Devices => write!(f, "Devices"),
        }
    }
}
