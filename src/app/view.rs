use iced::{
    Alignment, Color, Element,
    widget::{center, column, container, stack, text},
    window,
};

use super::{App, Message, Screen, VerificationState};

impl App {
    pub fn view(&self, _: window::Id) -> Element<'_, Message> {
        let base = match &self.screen {
            Screen::Auth(screen) => screen.view(self.error()).map(Message::Auth),
            Screen::Home(screen) => screen.view().map(Message::Home),
            Screen::Loading(msg) => center(
                column![text("Loading...").size(24), text(msg)]
                    .spacing(10)
                    .align_x(Alignment::Center),
            )
            .into(),
        };

        if self.verification_state.is_validating() {
            stack![base, self.verification_modal()].into()
        } else {
            base
        }
    }

    fn verification_modal(&self) -> Element<'_, Message> {
        let content: Element<'_, Message> = match &self.verification_state {
            VerificationState::Ongoing => components::ongoing(),
            VerificationState::Cancelled(info) => components::cancelled(info.reason()),
            VerificationState::Done => components::done(),
            VerificationState::Emoji(emojis) => components::emojis(emojis),
            VerificationState::Errored(error) => text(error.to_string()).into(),
            VerificationState::Confirmed => components::confirmed(),
            VerificationState::Created => components::created(),
            VerificationState::Stale => {
                unreachable!("Function should not be called when verification state is not stale")
            }
        };

        container(center(container(content).padding(24).style(|theme| {
            container::Style::default().background(theme.extended_palette().background.base.color)
        })))
        .style(|_| {
            container::Style::default().background(Color {
                a: 0.8,
                ..Color::BLACK
            })
        })
        .padding(24)
        .into()
    }
}

mod components {

    use super::Message;
    use lucide_icons::Icon;
    use matrix_sdk::encryption::verification::Emoji;

    use iced::{
        Alignment, Element, Font,
        font::Weight,
        widget::{button, column, container, row, text},
    };

    pub fn emojis(emojis: &[Emoji; 7]) -> Element<'_, Message> {
        let emojis = {
            let sets = emojis.iter().map(|emoji| {
                column![text(emoji.symbol).size(28), text(emoji.description)]
                    .align_x(Alignment::Center)
                    .spacing(2.5)
                    .into()
            });
            row(sets)
                .spacing(16)
                .align_y(Alignment::Center)
                .wrap()
                .align_x(Alignment::Center)
        };

        let actions = {
            row![
                button("They match").on_press(Message::AcceptEmojiVerification),
                button("They don't match")
                    .on_press(Message::CancelEmojiVerification)
                    .style(button::danger)
            ]
            .spacing(10)
            .align_y(Alignment::Center)
            .wrap()
            .align_x(Alignment::Center)
        };

        column![container(emojis).max_width(300), actions]
            .align_x(Alignment::Center)
            .spacing(20)
            .into()
    }

    pub fn cancelled(reason: &str) -> Element<'_, Message> {
        let header = column![
            Icon::CircleX.widget().size(32).style(text::danger),
            text("Authentication cancelled").size(24).font(Font {
                weight: Weight::Semibold,
                ..Font::DEFAULT
            }),
        ]
        .align_x(Alignment::Center)
        .spacing(2.5);

        let content = column![
            text!(
                "Authentication has been cancelled with the following reason: {}",
                reason
            ),
            button("Got it").on_press(Message::CloseVerificationModal)
        ]
        .align_x(Alignment::Center)
        .spacing(2.5);

        column![header, content]
            .align_x(Alignment::Center)
            .spacing(10)
            .into()
    }

    pub fn done<'a>() -> Element<'a, Message> {
        let header = column![
            Icon::CircleCheck.widget().size(32).style(text::success),
            text("Authentication completed!")
                .font(Font {
                    weight: Weight::Semibold,
                    ..Font::DEFAULT
                })
                .size(24)
        ]
        .spacing(2.5)
        .align_x(Alignment::Center);

        column![
            header,
            button("Good!").on_press(Message::CloseVerificationModal)
        ]
        .align_x(Alignment::Center)
        .spacing(5)
        .into()
    }

    pub fn ongoing<'a>() -> Element<'a, Message> {
        column![
            Icon::Clock.widget().size(32),
            text("Loading verification flow...").size(16)
        ]
        .align_x(Alignment::Center)
        .spacing(2.5)
        .into()
    }

    pub fn confirmed<'a>() -> Element<'a, Message> {
        column![
            Icon::Clock.widget().size(32),
            text("Waiting for the other device's response...").size(16)
        ]
        .align_x(Alignment::Center)
        .spacing(2.5)
        .into()
    }

    pub fn created<'a>() -> Element<'a, Message> {
        column![
            Icon::Clock.widget().size(32),
            text("Waiting for the other device to accept the request...").size(16)
        ]
        .align_x(Alignment::Center)
        .spacing(2.5)
        .into()
    }
}
