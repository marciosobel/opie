use iced::{
    Alignment, Element,
    widget::{button, center, column, text, text_input},
};

use crate::{Action, app::AppError};

#[derive(Debug, Clone)]
pub struct State {
    email: String,
    password: String,
    server: String,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Authenticate {
        server: String,
        email: String,
        password: String,
    },
}

#[derive(Debug, Clone)]
pub enum Message {
    EmailChanged(String),
    PasswordChanged(String),
    ServerChanged(String),
    Auth,
}

impl State {
    pub fn new() -> Self {
        Self {
            email: String::new(),
            password: String::new(),
            server: String::from("matrix.org"),
        }
    }

    pub fn update(&mut self, message: Message) -> Action<Instruction, Message> {
        match message {
            Message::EmailChanged(email) => {
                self.email = email;
                Action::none()
            }
            Message::PasswordChanged(password) => {
                self.password = password;
                Action::none()
            }
            Message::ServerChanged(server) => {
                self.server = server;
                Action::none()
            }
            Message::Auth => {
                let instruction = Instruction::Authenticate {
                    server: self.server.clone(),
                    email: self.email.clone(),
                    password: self.password.clone(),
                };

                Action::instruction(instruction)
            }
        }
    }

    pub fn view(&self, error: &Option<AppError>) -> Element<'_, Message> {
        let email_input = text_input("E-email", &self.email).on_input(Message::EmailChanged);

        let provider_input = text_input("Provider", &self.server).on_input(Message::ServerChanged);

        let password_input = text_input("Password", &self.password)
            .secure(true)
            .on_input(Message::PasswordChanged);

        let mut content = column![
            text("OPIE").size(30),
            email_input,
            password_input,
            provider_input,
            button("Login").on_press(Message::Auth)
        ]
        .align_x(Alignment::Center)
        .spacing(10)
        .max_width(300);

        if let Some(error) = error {
            content = content.push(text!("Error: {:?}", error).style(text::danger));
        }

        center(content).padding(20).into()
    }
}
