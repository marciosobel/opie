use std::sync::{Arc, Mutex};

use futures::StreamExt;
use matrix_sdk::encryption::verification::{
    SasState, SasVerification, Verification, VerificationRequest, VerificationRequestState,
};

mod action;
pub use action::Action;

mod error;
pub use error::Error;

mod event;
pub use event::Event;

mod listener;
pub use listener::Listener;

use crate::matrix::bridge::{ActionSender, EventSender};

pub type Bridge = ActionSender<Action>;

/// Creates a channel that sends events about the verification state
pub async fn request_verification(
    request: VerificationRequest,
    mut sender: EventSender<Event>,
    sas: Arc<Mutex<Option<SasVerification>>>,
) -> Result<(), matrix_sdk::Error> {
    request.accept().await?;

    tracing::info!("Received verification request");
    let mut stream = request.changes();
    tokio::spawn(async move {
        while let Some(state) = stream.next().await {
            match state {
                VerificationRequestState::Transitioned { verification } => {
                    if let Verification::SasV1(created) = verification {
                        let mut sas = sas.lock().expect("Should not be poisoned");
                        *sas = Some(created.clone());
                        tokio::spawn(sas_verification_handler(created, sender.clone()));
                    }
                }
                VerificationRequestState::Done => {
                    tracing::info!("DONEE");
                    sender.send(Event::Done).await;
                    break;
                }
                VerificationRequestState::Cancelled(info) => {
                    sender.send(Event::Cancelled(info)).await;
                    break;
                }
                VerificationRequestState::Ready { .. } => {
                    if let Err(error) = request.start_sas().await {
                        sender.send(Event::Error(error.into())).await;
                    }
                }
                _ => {}
            }
        }
    });

    Ok(())
}

async fn sas_verification_handler(sas: SasVerification, mut sender: EventSender<Event>) {
    if let Err(error) = sas.accept().await {
        sender.send(Event::Error(error.into())).await;
        return;
    }

    sender.send(Event::Started).await;

    let mut stream = sas.changes();
    while let Some(state) = stream.next().await {
        match state {
            SasState::KeysExchanged { emojis, .. } => {
                let emojis = match emojis {
                    Some(auth) => auth.emojis,
                    None => {
                        sender
                            .send(Event::Error(Error::InvalidVerificationType))
                            .await;
                        break;
                    }
                };
                sender.send(Event::VerifyEmojis(emojis)).await;
            }
            SasState::Done { .. } => {
                tracing::info!("Sending event: Done");
                sender.send(Event::Done).await;
                break;
            }
            SasState::Cancelled(info) => {
                sender.send(Event::Cancelled(info)).await;
                break;
            }
            _ => {}
        }
    }
}
