use futures::StreamExt;
use matrix_sdk::{
    Client as MatrixClient,
    ruma::events::{
        key::verification::request::ToDeviceKeyVerificationRequestEvent,
        room::message::{MessageType, OriginalSyncRoomMessageEvent},
    },
};

use crate::{Event, channel, services::sas_verification};

/// Creates a new bridge with the sas verification service
pub fn create_sas_bridge(
    client: MatrixClient,
    mut matrix_event_tx: channel::EventSender<Event>,
) -> sas_verification::Bridge {
    let listener = sas_verification::Listener::new(client.clone());

    let tx = listener.event.tx.clone();
    let sas = listener.sas.clone();
    client.add_event_handler(
        |ev: ToDeviceKeyVerificationRequestEvent, client: MatrixClient| async move {
            let request = client
                .encryption()
                .get_verification_request(&ev.sender, &ev.content.transaction_id)
                .await
                .expect("Request object wasn't created");

            tokio::spawn(sas_verification::request_verification(request, tx, sas));
        },
    );

    let tx = listener.event.tx;
    let sas = listener.sas.clone();
    client.add_event_handler(
        |ev: OriginalSyncRoomMessageEvent, client: MatrixClient| async move {
            if let MessageType::VerificationRequest(_) = &ev.content.msgtype {
                let request = client
                    .encryption()
                    .get_verification_request(&ev.sender, &ev.event_id)
                    .await
                    .expect("Request object wasn't created");

                tokio::spawn(sas_verification::request_verification(request, tx, sas));
            }
        },
    );

    let mut rx = listener.event.rx;
    tokio::spawn(async move {
        while let Some(event) = rx.next().await {
            matrix_event_tx.send(event).await;
        }
    });

    listener.action_sender
}
