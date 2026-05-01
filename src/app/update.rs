use std::sync::Arc;

use iced::{Task, widget::operation::focus_next};
use rand::{RngExt, distr::Alphanumeric};

use crate::{
    Action,
    screen::{auth, home},
};

use matrix::{
    bridge::{
        Event,
        action::{Action as MatrixAction, AuthAction},
    },
    services::{SasAction, sas_verification},
};

use super::{App, Instruction, Message, Screen, VerificationState};

impl App {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Error(app_error) => {
                tracing::error!("Error: {:?}", app_error);
                self.error = Some(app_error).into();
                Task::none()
            }
            Message::WindowOpened(_) => focus_next(),
            Message::WindowClosed(_) => iced::exit(),
            Message::MatrixEvent(event) => self.handle_matrix_event(event),
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
            Message::CloseVerificationModal => match &mut self.verification_state {
                VerificationState::Emoji(_)
                | VerificationState::Ongoing
                | VerificationState::Confirmed
                | VerificationState::Created => {
                    if let Some(bridge) = &mut self.bridge {
                        bridge.send(SasAction::Cancel);
                    }
                    Task::none()
                }
                VerificationState::Done | VerificationState::Cancelled(_) => {
                    self.verification_state = VerificationState::Stale;
                    Task::none()
                }
                VerificationState::Stale => Task::none(),
                VerificationState::Errored(error) => {
                    let error = matrix::Error::SasVerificationError(error.clone());
                    self.error = Arc::new(Some(error.into()));
                    Task::none()
                }
            },
            Message::AcceptEmojiVerification => {
                if let Some(bridge) = &mut self.bridge {
                    bridge.send(SasAction::Accept);
                }
                Task::none()
            }
            Message::CancelEmojiVerification => {
                if let Some(bridge) = &mut self.bridge {
                    bridge.send(SasAction::Mismatch);
                }
                Task::none()
            }
        }
    }

    fn handle_matrix_event(&mut self, event: Event) -> Task<Message> {
        match event.clone() {
            Event::Stale(mut bridge) => {
                self.bridge = Some(bridge.clone());
                tracing::info!("Matrix bridge stored in the app state");

                self.screen = Screen::Loading("Checking session...".to_string());
                bridge.send(AuthAction::RestoreSession);
            }
            Event::Error(error) => {
                tracing::error!("Received error event from matrix bridge: {:?}", error);
                return Task::done(Message::Error(error.into()));
            }
            Event::Authenticated(user_info) => {
                if let Some(bridge) = self.bridge.clone() {
                    tracing::info!("Authentication successful, showing the main screen");
                    let state = home::State::new(bridge, user_info);
                    self.screen = Screen::Home(state);
                } else {
                    tracing::error!("Received Authenticated event without a bridge");
                }
            }
            Event::Syncing => {
                self.screen = Screen::Loading("Syncing the client...".to_string());
            }
            Event::SessionRestoreFailed => {
                tracing::info!("Session restore failed, showing auth screen");
                let state = auth::State::new();
                self.screen = Screen::Auth(state);
            }
            Event::Ready => {
                tracing::info!("Bridge created successfully");
            }
            Event::SasVerificationEvent(event) => match event {
                sas_verification::Event::Error(error) => tracing::error!("{}", error),
                e => self.verification_state = e.into(),
            },

            // Not doing a catch-all arm because I think it's good to
            // make the top-most wrapper of our app "aware" of all events.
            Event::RoomList(_) => {}
            Event::TimelineEvent(_) => {}
            Event::DeviceList(_) => {}
            Event::UserAvatarFetched(_, _) => {}
            Event::TimelineImageFetched(_, _) => {}
            Event::GetUserResponse(_) => {}
        };

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

        Task::none()
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
                        .send(AuthAction::Authenticate { username, password });

                    Task::none()
                }
            },
            Instruction::Main(instruction) => match instruction {},
        }
    }
}
