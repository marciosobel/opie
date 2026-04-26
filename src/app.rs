use std::sync::Arc;

use iced::{Subscription, Task, event, window};
use matrix::bridge;

use crate::{
    Settings,
    screen::{auth, home},
};

mod error;
pub use error::AppError;

mod update;
mod verification_state;
mod view;

pub(self) use verification_state::VerificationState;

pub struct App {
    bridge: Option<bridge::Bridge>,
    error: Arc<Option<AppError>>,
    screen: Screen,
    verification_state: VerificationState,
    // settings: Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Window has opened
    WindowOpened(window::Id),
    /// Window has closed
    WindowClosed(window::Id),
    /// A matrix event.
    MatrixEvent(bridge::Event),
    /// An error that occurred in the app
    Error(AppError),
    /// Close the sas verification modal
    CloseVerificationModal,
    /// The emojis in the current verification flow matches with the other device
    AcceptEmojiVerification,
    /// The emojis in the current verification flow does not match with the other device
    CancelEmojiVerification,

    // Screen messages
    /// Messages from the authentication screen
    Auth(auth::Message),
    /// Messages from the main screen
    Home(home::Message),
}

#[derive(Debug, Clone)]
pub enum Screen {
    Auth(auth::State),
    Home(home::State),
    Loading(String),
}

#[derive(Debug, Clone)]
enum Instruction {
    Auth(auth::Instruction),
    Main(home::Instruction),
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let mut tasks = vec![];

        let (_, open) = window::open(window::Settings {
            position: window::Position::Centered,
            ..Default::default()
        });
        tasks.push(open.map(Message::WindowOpened));

        let _settings = Settings::load().expect("Failed to load settings");

        (
            Self {
                bridge: None,
                screen: Screen::Loading("Initializing app...".to_string()),
                error: Arc::new(None),
                verification_state: VerificationState::Stale,
                // settings,
            },
            Task::batch(tasks),
        )
    }

    pub fn theme(&self, _: window::Id) -> iced::theme::Theme {
        iced::theme::Theme::GruvboxDark
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            event::listen_with(handle_event),
            window::close_events().map(Message::WindowClosed),
            bridge::subscribe().map(Message::MatrixEvent),
        ])
    }

    fn error(&self) -> Option<&AppError> {
        (*self.error).as_ref()
    }
}

fn handle_event(event: event::Event, _: event::Status, _: iced::window::Id) -> Option<Message> {
    match event {
        _ => None,
    }
}
