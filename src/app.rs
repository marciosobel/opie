use std::sync::Arc;

use iced::{
    Alignment, Subscription, Task, event,
    futures::SinkExt,
    widget::{center, column, operation::focus_next, text},
    window,
};
use screen_macro::screen;
use thiserror::Error;

use crate::{
    Action, Settings,
    matrix::{
        self,
        bridge::{Action as MatrixAction, MatrixBridgeSender},
    },
    screen::{auth, main},
};

pub struct App {
    bridge: Option<MatrixBridgeSender>,
    error: Arc<Option<AppError>>,
    screen: Screen,
    settings: Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Window has opened
    WindowOpened(window::Id),
    /// Window has closed
    WindowClosed(window::Id),
    /// A matrix event.
    MatrixEvent(matrix::bridge::Event),
    /// An error that occurred in the app
    Error(AppError),

    // Screen messages
    /// Messages from the authentication screen
    Auth(auth::Message),
    /// Messages from the main screen
    Main(main::Message),
}

pub enum Screen {
    Auth(auth::State),
    Main(main::State),
    Loading(String),
}

#[derive(Debug, Clone)]
enum Instruction {
    Auth(auth::Instruction),
    Main(main::Instruction),
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let mut tasks = vec![];

        let (_, open) = window::open(window::Settings {
            position: window::Position::Centered,
            ..Default::default()
        });
        tasks.push(open.map(Message::WindowOpened));

        let settings = Settings::load().expect("Failed to load settings");

        (
            Self {
                bridge: None,
                screen: Screen::Loading("Initializing app...".to_string()),
                error: Arc::new(None),
                settings,
            },
            Task::batch(tasks),
        )
    }

    pub fn theme(&self, _: window::Id) -> iced::theme::Theme {
        iced::theme::Theme::KanagawaDragon
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Error(app_error) => {
                tracing::error!("Error: {:?}", app_error);
                self.error = Arc::new(Some(app_error));
                Task::none()
            }
            Message::WindowOpened(_) => focus_next(),
            Message::WindowClosed(_) => iced::exit(),
            Message::MatrixEvent(event) => self.handle_matrix_event(event),

            // Screen messages
            Message::Auth(message) => {
                let screen = screen!(self, Screen::Auth);

                let action = screen
                    .update(message)
                    .map(Message::Auth)
                    .map_instruction(Instruction::Auth);

                self.handle_action(action)
            }
            Message::Main(message) => {
                let screen = screen!(self, Screen::Main);
                let action = screen
                    .update(message)
                    .map(Message::Main)
                    .map_instruction(Instruction::Main);

                self.handle_action(action)
            }
        }
    }

    pub fn view(&self, _: window::Id) -> iced::Element<'_, Message> {
        match &self.screen {
            Screen::Auth(screen) => screen.view(self.error()).map(Message::Auth),
            Screen::Main(screen) => screen.view().map(Message::Main),
            Screen::Loading(msg) => center(
                column![text("Loading...").size(24), text(msg)]
                    .spacing(10)
                    .align_x(Alignment::Center),
            )
            .into(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            event::listen_with(handle_event),
            window::close_events().map(Message::WindowClosed),
            matrix::bridge::subscribe().map(Message::MatrixEvent),
        ])
    }

    fn handle_action(&mut self, action: Action<Instruction, Message>) -> Task<Message> {
        let instruction_task = match action.instruction {
            Some(instruction) => self.perform(instruction),
            None => Task::none(),
        };

        return instruction_task.chain(action.task);
    }

    fn perform(&mut self, instruction: Instruction) -> Task<Message> {
        tracing::info!("Performing instruction: {:?}", instruction);
        match instruction {
            Instruction::Auth(instruction) => match instruction {
                auth::Instruction::Authenticate {
                    server,
                    username,
                    password,
                } => {
                    let Some(bridge) = &mut self.bridge else {
                        tracing::error!("No bridge available to authenticate");
                        return Task::none();
                    };

                    tracing::info!(
                        "Sending CreateMatrixClient and Authenticate actions to the bridge"
                    );
                    _ = bridge.try_send(MatrixAction::CreateMatrixClient { server });
                    _ = bridge.try_send(MatrixAction::Authenticate { username, password });

                    Task::none()
                }
            },
            Instruction::Main(instruction) => match instruction {},
        }
    }

    fn error(&self) -> Option<&AppError> {
        (*self.error).as_ref()
    }

    fn handle_matrix_event(&mut self, event: matrix::bridge::Event) -> Task<Message> {
        match event {
            matrix::bridge::Event::Stale(mut bridge) => {
                self.bridge = Some(bridge.clone());
                tracing::info!("Matrix bridge stored in the app state");

                self.screen = Screen::Loading("Checking session...".to_string());
                let Some(server) = self.settings.server() else {
                    tracing::info!("No server found in settings, showing auth screen");
                    self.screen = Screen::Auth(auth::State::new());
                    return Task::none();
                };

                tracing::info!("Homeserver found in settings, trying to restore session");

                self.screen = Screen::Loading("Creating matrix client...".to_string());
                _ = bridge.try_send(MatrixAction::CreateMatrixClient { server });
                _ = bridge.try_send(MatrixAction::RestoreSession);

                Task::none()
            }
            matrix::bridge::Event::Error(error) => {
                tracing::error!("Received error event from matrix bridge: {:?}", error);
                Task::done(Message::Error(error.into()))
            }
            matrix::bridge::Event::Authenticated => {
                let Some(bridge) = self.bridge.clone() else {
                    tracing::error!("Received Authenticated event without a bridge");
                    return Task::none();
                };

                tracing::info!("Authentication successful, showing the main screen");
                let state = main::State::new(bridge);
                self.screen = Screen::Main(state);
                Task::none()
            }
            matrix::bridge::Event::SessionRestoreFailed => {
                tracing::info!("Session restore failed, showing auth screen");
                let state = auth::State::new();
                self.screen = Screen::Auth(state);
                Task::none()
            }
            matrix::bridge::Event::Ready => {
                tracing::info!("Bridge created successfully");
                Task::none()
            }
        }
    }
}

#[derive(Error, Debug, Clone)]
pub enum AppError {
    #[error("Matrix bridge error: {0}")]
    MatrixBridgeError(#[from] matrix::bridge::Error),

    #[error("Failed to save settings")]
    SettingsSaveError(#[from] Arc<anyhow::Error>),

    #[error("No session found to restore")]
    MatrixSessionRestoreNotFound,
}

fn handle_event(event: event::Event, _: event::Status, _: iced::window::Id) -> Option<Message> {
    match event {
        _ => None,
    }
}
