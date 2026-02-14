use std::sync::Arc;

use iced::{
    Subscription, Task, event,
    widget::{center, operation::focus_next, text},
    window,
};
use matrix_sdk::{Client, ClientBuildError};
use screen_macro::screen;
use thiserror::Error;

use crate::{Action, matrix, screen::auth};

pub struct App {
    /// Reference to the main SDK client.
    client: Option<Client>,
    error: Arc<Option<AppError>>,
    screen: Screen,
}

#[derive(Debug, Clone)]
pub enum Message {
    WindowOpened(window::Id),
    WindowClosed(window::Id),
    MatrixClientBuilt(Client),
    Authenticate { username: String, password: String },
    Authenticated,
    Error(AppError),
    Auth(auth::Message),
}

pub enum Screen {
    Auth(auth::State),
    Chat,
}

#[derive(Debug, Clone)]
enum Instruction {
    Auth(auth::Instruction),
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let (_, open) = window::open(window::Settings {
            position: window::Position::Centered,
            ..Default::default()
        });

        (
            Self {
                client: None,
                screen: Screen::Auth(auth::State::new()),
                error: Arc::new(None),
            },
            open.map(Message::WindowOpened),
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
            Message::Auth(message) => {
                let auth = screen!(self, Screen::Auth);

                let action = auth
                    .update(message)
                    .map(Message::Auth)
                    .map_instruction(Instruction::Auth);

                self.handle_action(action)
            }
            Message::Error(app_error) => {
                eprintln!("Error: {:?}", app_error);
                self.error = Arc::new(Some(app_error));
                Task::none()
            }
            Message::Authenticated => {
                self.screen = Screen::Chat;
                Task::none()
            }
            Message::Authenticate { username, password } => {
                let Some(client) = &self.client else {
                    return Task::none();
                };

                Task::perform(
                    matrix::authenticate(client.clone(), username, password),
                    |result| match result {
                        Ok(_) => Message::Authenticated,
                        Err(error) => Message::Error(Arc::new(error).into()),
                    },
                )
            }
            Message::WindowOpened(_) => focus_next(),
            Message::WindowClosed(_) => {
                let mut tasks = vec![];

                // since the app is closing, we can take the client off of our state
                let client = self.client.take();

                // gracefully shutdown the matrix client.
                // since it needs to be in an async runtime context,
                // we create a task that simply calls the `drop` function.
                if let Some(client) = client {
                    let task = Task::future(async move {
                        drop(client);
                    })
                    .discard();

                    tasks.push(task);
                }

                Task::batch(tasks).chain(iced::exit())
            }
        }
    }

    pub fn view(&self, _: window::Id) -> iced::Element<'_, Message> {
        match &self.screen {
            Screen::Auth(auth) => auth.view(&self.error).map(Message::Auth),
            Screen::Chat => center(text("Você autenticado pabens")).into(),
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
                } => Task::perform(matrix::init_client(server), move |result| match result {
                    Err(error) => Message::Error(Arc::new(error).into()),
                    Ok(client) => Message::MatrixClientBuilt(client),
                })
                .chain(Task::done(Message::Authenticate { username, password })),
            },
        }
    }
}

fn handle_event(event: event::Event, _: event::Status, _: iced::window::Id) -> Option<Message> {
    match event {
        _ => None,
    }
}

#[derive(Error, Debug, Clone)]
pub enum AppError {
    #[error("Failed to build matrix client: {0:?}")]
    MatrixClientBuildError(#[from] Arc<ClientBuildError>),

    #[error("Error from the matrix client: {0:?}")]
    MatrixError(#[from] Arc<matrix_sdk::Error>),
}
