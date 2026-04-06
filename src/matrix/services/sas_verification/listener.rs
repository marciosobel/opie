use std::sync::{Arc, Mutex};

use futures::channel::mpsc::Receiver;
use matrix_sdk::{Client, encryption::verification::SasVerification};

use crate::matrix::bridge::Channel;

use super::{Action, ActionSender, Error, Event, EventSender, request_verification};

/// A channel that can receive actions and send events for the sas verification flow.
pub struct Listener {
    pub event_rx: Receiver<Event>,
    pub event_tx: EventSender<Event>,
    pub action_tx: ActionSender<Action>,
    pub sas: Arc<Mutex<Option<SasVerification>>>,
}

impl Listener {
    /// Creates a new channel that will listen to sas verification actions and will emit events of any responses
    pub fn new(client: Client) -> Self {
        let (event_rx, action_tx, mut channel) = Channel::new();
        let event_tx = channel.sender();
        let sas: Arc<Mutex<Option<SasVerification>>> = Arc::default();

        let sas_clone = sas.clone();
        tokio::spawn(async move {
            let sas = sas_clone;
            loop {
                match channel.recv().await {
                    Action::VerifyDevice(device_id) => {
                        let user_id = match client.user_id() {
                            Some(id) => id,
                            None => {
                                channel.send(Error::NotAuthenticated).await;
                                continue;
                            }
                        };

                        let result = client
                            .encryption()
                            .get_device(user_id, &device_id)
                            .await
                            .map_err(Box::new)
                            .map_err(matrix_sdk::Error::CryptoStoreError);

                        let device = match result {
                            Ok(maybe_device) => match maybe_device {
                                Some(device) => device,
                                None => {
                                    channel.send(Error::DeviceNotFound).await;
                                    continue;
                                }
                            },
                            Err(error) => {
                                channel.send(Event::Error(error.into())).await;
                                continue;
                            }
                        };

                        let request = match device.request_verification().await {
                            Ok(request) => request,
                            Err(error) => {
                                channel.send(Event::Error(error.into())).await;
                                continue;
                            }
                        };

                        match request_verification(request, channel.sender(), sas.clone()).await {
                            Ok(_) => {}
                            Err(error) => {
                                channel.send(Event::Error(error.into())).await;
                                continue;
                            }
                        };
                    }
                    Action::Cancel => {
                        let sas = get_sas_from_mutex(&sas);
                        if let Some(sas) = sas {
                            if let Err(error) = sas.cancel().await {
                                channel.send(Event::Error(error.into())).await;
                            }
                        }
                    }
                    Action::Accept => {
                        let sas = get_sas_from_mutex(&sas);
                        if let Some(sas) = sas {
                            if let Err(error) = sas.confirm().await {
                                tracing::error!("failed to confirm sas: {:?}", error);
                                channel.send(Event::Error(error.into())).await;
                            }
                        }
                    }
                    Action::Mismatch => {
                        let sas = get_sas_from_mutex(&sas);
                        if let Some(sas) = sas {
                            if let Err(error) = sas.mismatch().await {
                                channel.send(Event::Error(error.into())).await;
                            }
                        }
                    }
                }
            }
        });

        Self {
            event_rx,
            event_tx,
            action_tx,
            sas,
        }
    }
}

fn get_sas_from_mutex(mutex: &Mutex<Option<SasVerification>>) -> Option<SasVerification> {
    let guard = mutex.lock().expect("Should not be poisoned");
    (*guard).clone()
}
