use iced::{
    Alignment, Color, Element,
    widget::{button, center, column, container, stack, text},
    window,
};
use matrix_sdk::encryption::verification::format_emojis;

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
            VerificationState::Ongoing => text("Loading verification flow...").size(16).into(),
            VerificationState::Cancelled(info) => column![
                text("Authentication cancelled").size(20),
                text("Authentication has been cancelled with the following reason:"),
                text(info.reason()),
                button("Ok").on_press(Message::CloseVerificationModal)
            ]
            .into(),
            VerificationState::Done => column![
                text("Authentication completed!"),
                button("Ok").on_press(Message::CloseVerificationModal)
            ]
            .into(),
            VerificationState::Emoji(emojis) => column![
                text(format_emojis(emojis.clone())),
                button("They match").on_press(Message::AcceptEmojiVerification),
                button("Does not match").on_press(Message::CancelEmojiVerification)
            ]
            .into(),
            VerificationState::Errored(error) => text(error.to_string()).into(),
            VerificationState::Stale => {
                unreachable!("Function should not be called when verification state is not stale")
            }
        };

        container(center(content))
            .style(|_| {
                container::Style::default().background(Color {
                    a: 0.8,
                    ..Color::BLACK
                })
            })
            .into()
    }
}
