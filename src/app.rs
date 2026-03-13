use std::sync::Arc;

use iced::{
    Alignment, Subscription, Task, event,
    widget::{center, column, operation::focus_next, text},
    window,
};
use rand::{RngExt, distr::Alphanumeric};
use thiserror::Error;

use crate::{
    Action, Settings,
    matrix::{
        self,
        bridge::{Action as MatrixAction, Bridge},
    },
    screen::{auth, home},
};

pub struct App {
    bridge: Option<Bridge>,
    error: Arc<Option<AppError>>,
    screen: Screen,
    // settings: Settings,
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
                // settings,
            },
            Task::batch(tasks),
        )
    }

    pub fn theme(&self, _: window::Id) -> iced::theme::Theme {
        iced::theme::Theme::KanagawaWave
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
                let Screen::Auth(screen) = &mut self.screen else {
                    return Task::none();
                };

                let action = screen
                    .update(message)
                    .map(Message::Auth)
                    .map_instruction(Instruction::Auth);

                self.handle_action(action)
            }
            Message::Home(message) => {
                let Screen::Home(screen) = &mut self.screen else {
                    return Task::none();
                };

                let action = screen
                    .update(message)
                    .map(Message::Home)
                    .map_instruction(Instruction::Main);

                self.handle_action(action)
            }
        }
    }

    pub fn view(&self, _: window::Id) -> iced::Element<'_, Message> {
        match &self.screen {
            Screen::Auth(screen) => screen.view(self.error()).map(Message::Auth),
            Screen::Home(screen) => screen.view().map(Message::Home),
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
                    homeserver,
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

                    let mut rng = rand::rng();
                    let passphrase: String = (&mut rng)
                        .sample_iter(Alphanumeric)
                        .take(32)
                        .map(char::from)
                        .collect();

                    bridge
                        .send(MatrixAction::CreateMatrixClient {
                            homeserver,
                            passphrase,
                        })
                        .send(MatrixAction::Authenticate { username, password });

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
        match &mut self.screen {
            Screen::Home(state) => {
                let action = state
                    .update(home::Message::MatrixEvent(event))
                    .map(Message::Home)
                    .map_instruction(Instruction::Main);
                return self.handle_action(action);
            }
            _ => {}
        };

        match event {
            matrix::bridge::Event::Stale(mut bridge) => {
                self.bridge = Some(bridge.clone());
                tracing::info!("Matrix bridge stored in the app state");

                self.screen = Screen::Loading("Checking session...".to_string());
                bridge.send(MatrixAction::RestoreSession);
            }
            matrix::bridge::Event::Error(error) => {
                tracing::error!("Received error event from matrix bridge: {:?}", error);
                return Task::done(Message::Error(error.into()));
            }
            matrix::bridge::Event::Authenticated => {
                if let Some(bridge) = self.bridge.clone() {
                    tracing::info!("Authentication successful, showing the main screen");
                    let state = home::State::new(bridge);
                    self.screen = Screen::Home(state);
                } else {
                    tracing::error!("Received Authenticated event without a bridge");
                }
            }
            matrix::bridge::Event::Syncing => {
                self.screen = Screen::Loading("Syncing the client...".to_string());
            }
            matrix::bridge::Event::SessionRestoreFailed => {
                tracing::info!("Session restore failed, showing auth screen");
                let state = auth::State::new();
                self.screen = Screen::Auth(state);
            }
            matrix::bridge::Event::Ready => {
                tracing::info!("Bridge created successfully");
            }
            matrix::bridge::Event::RoomList(_) => {}
            matrix::bridge::Event::TimelineEvent(_) => {}
        };

        Task::none()
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
