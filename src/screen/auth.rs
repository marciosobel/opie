use iced::{
    Alignment, Element,
    widget::{button, center, column, text, text_input},
};

use crate::{Action, app::AppError};

#[derive(Debug, Clone)]
pub struct State {
    username: String,
    password: String,
    homeserver: String,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Authenticate {
        homeserver: String,
        username: String,
        password: String,
    },
}

#[derive(Debug, Clone)]
pub enum Message {
    UsernameChanged(String),
    PasswordChanged(String),
    ServerChanged(String),
    Auth,
}

impl State {
    pub fn new() -> Self {
        Self {
            username: String::new(),
            password: String::new(),
            homeserver: String::from("matrix.org"),
        }
    }

    pub fn update(&mut self, message: Message) -> Action<Instruction, Message> {
        match message {
            Message::UsernameChanged(username) => {
                self.username = username;
                Action::none()
            }
            Message::PasswordChanged(password) => {
                self.password = password;
                Action::none()
            }
            Message::ServerChanged(server) => {
                self.homeserver = server;
                Action::none()
            }
            Message::Auth => {
                let instruction = Instruction::Authenticate {
                    homeserver: self.homeserver.clone(),
                    username: self.username.clone(),
                    password: self.password.clone(),
                };

                Action::instruction(instruction)
            }
        }
    }

    pub fn view(&self, error: Option<&AppError>) -> Element<'_, Message> {
        let username_input =
            text_input("Username", &self.username).on_input(Message::UsernameChanged);

        let provider_input =
            text_input("Provider", &self.homeserver).on_input(Message::ServerChanged);

        let password_input = text_input("Password", &self.password)
            .secure(true)
            .on_input(Message::PasswordChanged);

        let mut content = column![
            text("OPIE").size(30),
            username_input,
            password_input,
            provider_input,
            button("Login").on_press(Message::Auth)
        ]
        .align_x(Alignment::Center)
        .spacing(10)
        .max_width(300);

        if let Some(error) = error {
            content = content.push(text!("Error: {}", error).style(text::danger));
        }

        center(content).padding(20).into()
    }
}
