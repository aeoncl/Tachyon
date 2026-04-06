use std::time::Duration;
use futures_util::StreamExt;
use matrix_sdk::event_handler::Ctx;
use matrix_sdk::{Client, Room};
use matrix_sdk::encryption::verification::{SasState, SasVerification, Verification, VerificationRequest, VerificationRequestState};
use ruma::events::presence::PresenceEvent;
use tokio::time::sleep;
use crate::matrix::handlers::context::TachyonContext;

pub async fn request_verification_handler(client: Client, request: VerificationRequest) {
    println!("Accepting verification request from {}", request.other_user_id());


    request.accept().await.expect("Can't accept verification request");

    let mut stream = request.changes();

    while let Some(state) = stream.next().await {
        match state {
            VerificationRequestState::Created { .. }
            | VerificationRequestState::Requested { .. }
            | VerificationRequestState::Ready { .. } => (),
            VerificationRequestState::Transitioned { verification } => {
                // We only support SAS verification.
                if let Verification::SasV1(sas) = verification {
                    tokio::spawn(sas_verification_handler(client, sas));
                    break;
                }
            }
            VerificationRequestState::Done | VerificationRequestState::Cancelled(_) => break,
        }
    }
}

async fn sas_verification_handler(client: Client, sas: SasVerification) {
    println!(
        "Starting verification with {} {}",
        &sas.other_device().user_id(),
        &sas.other_device().device_id()
    );
    sas.accept().await.unwrap();

    let mut stream = sas.changes();

    while let Some(state) = stream.next().await {
        match state {
            SasState::KeysExchanged { emojis, decimals: _ } => {
                //TODO: Wait for confirmation

                //FIXME: Remove this
                println!("{:?}", emojis);
                sleep(Duration::from_millis(5000)).await;
                sas.confirm().await.unwrap();

            }
            SasState::Done { .. } => {
                let device = sas.other_device();

                println!(
                    "Successfully verified device {} {} {:?}",
                    device.user_id(),
                    device.device_id(),
                    device.local_trust_state()
                );

                break;
            }
            SasState::Cancelled(cancel_info) => {
                println!("The verification has been cancelled, reason: {}", cancel_info.reason());

                break;
            }
            SasState::Created { .. }
            | SasState::Started { .. }
            | SasState::Accepted { .. }
            | SasState::Confirmed => (),
        }
    }
}