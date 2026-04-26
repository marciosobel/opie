use std::sync::{Arc, Mutex};

use futures::channel::mpsc::Receiver;
use matrix_sdk::{Client, encryption::verification::SasVerification, ruma::OwnedDeviceId};

use crate::Channel;

use super::{Action, ActionSender, Error, Event, EventSender, request_verification};

/// A channel that can receive actions and send events for the sas verification flow.
pub struct Listener {
    pub event: EventListener,
    pub action_sender: ActionSender<Action>,
    pub sas: Arc<Mutex<Option<SasVerification>>>,
}

pub struct EventListener {
    pub rx: Receiver<Event>,
    pub tx: EventSender<Event>,
}

impl Listener {
    /// Creates a new channel that will listen to sas verification actions and will emit events of any responses
    pub fn new(client: Client) -> Self {
        let (event_rx, action_sender, channel) = Channel::new();
        let event_tx = channel.sender();
        let sas: Arc<Mutex<Option<SasVerification>>> = Arc::default();

        tokio::spawn(handle_actions(client, sas.clone(), channel));

        Self {
            event: EventListener {
                rx: event_rx,
                tx: event_tx,
            },
            action_sender,
            sas,
        }
    }
}

async fn handle_actions(
    client: Client,
    sas: Arc<Mutex<Option<SasVerification>>>,
    mut channel: Channel<Action, Event>,
) {
    loop {
        match channel.recv().await {
            Action::VerifyDevice(device_id) => {
                let client = client.clone();
                let sas = sas.clone();
                let tx = channel.sender();

                if let Err(error) = verify_device(device_id, client, sas, tx).await {
                    channel.send(Event::Error(error)).await;
                }
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
}

async fn verify_device(
    device_id: OwnedDeviceId,
    client: Client,
    sas: Arc<Mutex<Option<SasVerification>>>,
    tx: EventSender<Event>,
) -> Result<(), Error> {
    let user_id = match client.user_id() {
        Some(id) => id,
        None => {
            return Err(Error::NotAuthenticated);
        }
    };

    let maybe_device = client
        .encryption()
        .get_device(user_id, &device_id)
        .await
        .map_err(|e| {
            let e = Box::new(e);
            let matrix_err = Arc::new(matrix_sdk::Error::CryptoStoreError(e));
            Error::from(matrix_err)
        })?;

    let device = match maybe_device {
        Some(device) => device,
        None => {
            return Err(Error::DeviceNotFound);
        }
    };

    let request = device
        .request_verification()
        .await
        .map_err(|e| Error::from(Arc::new(e)))?;

    request_verification(request, tx, sas.clone())
        .await
        .map_err(|e| Error::from(Arc::new(e)))
}

fn get_sas_from_mutex(mutex: &Mutex<Option<SasVerification>>) -> Option<SasVerification> {
    let guard = mutex.lock().expect("Should not be poisoned");
    (*guard).clone()
}
