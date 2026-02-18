use std::sync::Arc;

use iced::{
    Subscription, Task, event,
    widget::{center, operation::focus_next, text},
    window,
};
use matrix_sdk::{Client, ClientBuildError};
use screen_macro::screen;
use thiserror::Error;

use crate::{
    Action, Settings,
    async_dropper::AsyncDropper,
    matrix,
    screen::{auth, main},
};

pub struct App {
    /// Reference to the main SDK client.
    client: Option<AsyncDropper<Client>>,
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
    /// The matrix client has been built and is ready to use
    MatrixClientBuilt(AsyncDropper<Client>),
    /// Attempt to restore a session from disk
    RestoreSession,
    /// Failed to restore a session from disk
    RestoreSessionFailed(AppError),
    /// Store the matrix server provider in the settings
    SaveMatrixServer(String),
    /// Attempt to authenticate
    Authenticate { username: String, password: String },
    /// User authenticated
    Authenticated,
    /// An error that occurred in the app
    Error(AppError),

    //Screen messages
    /// Messages from the authentication screen
    Auth(auth::Message),
    /// Messages from the main screen
    Main(main::Message),
}

pub enum Screen {
    Auth(auth::State),
    Main(main::State),
    Loading,
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

        if let Some(server) = settings.server() {
            tasks.push(restore_session(server));
        }

        (
            Self {
                client: None,
                screen: Screen::Loading,
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
            Message::MatrixClientBuilt(client) => {
                self.client = Some(client);
                Task::none()
            }
            Message::RestoreSession => {
                let Some(client) = self.client.clone() else {
                    return Task::none();
                };

                Task::future(matrix::restore_session(client)).then(|result| match result {
                    Ok(status) => match status {
                        matrix::RestoreStatus::Restored => Task::done(Message::Authenticated),
                        matrix::RestoreStatus::NoSession => {
                            let error = AppError::MatrixSessionRestoreNotFound;
                            Task::done(Message::RestoreSessionFailed(error))
                        }
                    },
                    Err(error) => {
                        let error = AppError::MatrixError(Arc::new(error.into()));
                        Task::done(Message::Error(error))
                    }
                })
            }
            Message::RestoreSessionFailed(error) => {
                eprintln!("Failed to restore session: {:?}", error);
                self.screen = Screen::Auth(auth::State::new());
                self.error = Arc::new(Some(error));
                Task::none()
            }
            Message::SaveMatrixServer(server) => match self.settings.set_server(server).save() {
                Ok(_) => Task::none(),
                Err(error) => {
                    let error = AppError::SettingsSaveError(Arc::new(error.into()));
                    Task::done(Message::Error(error))
                }
            },
            Message::Error(app_error) => {
                eprintln!("Error: {:?}", app_error);
                self.error = Arc::new(Some(app_error));
                Task::none()
            }
            Message::Authenticated => {
                let Some(client) = self.client.clone() else {
                    eprintln!("Authentication complete but no client found");
                    return Task::none();
                };

                self.screen = Screen::Main(main::State::new(client));
                Task::none()
            }
            Message::Authenticate { username, password } => {
                let Some(client) = self.client.clone() else {
                    return Task::none();
                };

                Task::perform(matrix::authenticate(client, username, password), |result| {
                    match result {
                        Ok(_) => Message::Authenticated,
                        Err(error) => Message::Error(Arc::new(error).into()),
                    }
                })
            }
            Message::WindowOpened(_) => focus_next(),
            Message::WindowClosed(_) => {
                let Some(client) = self.client.take() else {
                    return iced::exit();
                };

                Task::future(async move { client })
                    .discard()
                    .chain(iced::exit())
            }

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
            Screen::Loading => center(text("Loading...")).into(),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            event::listen_with(handle_event),
            window::close_events().map(Message::WindowClosed),
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
        match instruction {
            Instruction::Auth(instruction) => match instruction {
                auth::Instruction::Authenticate {
                    server,
                    username,
                    password,
                } => Task::perform(
                    matrix::init_client(server.clone()),
                    move |result| match result {
                        Err(error) => Message::Error(Arc::new(error).into()),
                        Ok(client) => Message::MatrixClientBuilt(AsyncDropper::new(client)),
                    },
                )
                .chain(Task::batch([
                    Task::done(Message::Authenticate { username, password }),
                    Task::done(Message::SaveMatrixServer(server)),
                ])),
            },
            Instruction::Main(instruction) => match instruction {},
        }
    }

    fn error(&self) -> Option<&AppError> {
        (*self.error).as_ref()
    }
}

#[derive(Error, Debug, Clone)]
pub enum AppError {
    #[error("Failed to build matrix client")]
    MatrixClientBuildError(#[from] Arc<ClientBuildError>),

    #[error("Error from the matrix client")]
    MatrixError(#[from] Arc<matrix_sdk::Error>),

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

fn restore_session(server: String) -> Task<Message> {
    Task::future(matrix::init_client(server)).then(move |result| match result {
        Err(error) => {
            let error = Arc::new(error).into();
            Task::done(Message::RestoreSessionFailed(error))
        }
        Ok(client) => {
            let create_client = Task::done(Message::MatrixClientBuilt(AsyncDropper::new(client)));
            let restore_session = Task::done(Message::RestoreSession);

            create_client.chain(restore_session)
        }
    })
}
