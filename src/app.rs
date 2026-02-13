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
    WindowClosed(window::Id),
    MatrixClientDropped,
    MatrixClientBuilt(Client),
    Authenticate { email: String, password: String },
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
        (
            Self {
                client: None,
                screen: Screen::Auth(auth::State::new()),
                error: Arc::new(None),
            },
            focus_next(),
        )
    }

    pub fn theme(&self) -> iced::theme::Theme {
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
                Task::none()
            }
            Message::Authenticated => {
                self.screen = Screen::Chat;
                Task::none()
            }
            Message::Authenticate { email, password } => {
                let Some(client) = &self.client else {
                    return Task::none();
                };

                Task::perform(
                    client
                        .matrix_auth()
                        .login_username(&email, &password)
                        .into_future(),
                    |result| match result {
                        Ok(_) => Message::Authenticated,
                        Err(error) => Message::Error(Arc::new(error).into()),
                    },
                )
            }
            Message::WindowClosed(_) => {
                let mut tasks = vec![];

                // since the app is closing, we can take the client off of our state
                let client = self.client.take();

                // gracefully shutdown the matrix client.
                // since it needs to be in a tokio runtime context,
                // we create a task that simply calls the `drop` function.
                if let Some(client) = client {
                    let task = Task::future(async move {
                        drop(client);
                    })
                    .discard();

                    tasks.push(task);
                }

                Task::batch(tasks)
            }
            Message::MatrixClientDropped => Task::none(),
        }
    }

    pub fn view(&self) -> iced::Element<'_, Message> {
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
                    email,
                    password,
                } => Task::perform(matrix::init_client(server), move |result| match result {
                    Err(error) => Message::Error(Arc::new(error).into()),
                    Ok(client) => Message::MatrixClientBuilt(client),
                })
                .chain(Task::done(Message::Authenticate { email, password })),
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
