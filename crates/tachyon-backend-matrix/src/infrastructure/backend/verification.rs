use std::pin::pin;
use std::time::Duration;

use anyhow::anyhow;
use futures_util::{Stream, StreamExt};
use matrix_sdk::encryption::identities::Device;
use matrix_sdk::encryption::verification::{
    SasState, SasVerification, VerificationRequest, VerificationRequestState,
};
use matrix_sdk::ruma::api::client::sync::sync_events::v5::request::{E2EE, ListFilters, ToDevice};
use matrix_sdk::ruma::directory::RoomTypeFilter;
use matrix_sdk::ruma::events::key::verification::VerificationMethod;
use matrix_sdk::sliding_sync::Range;
use matrix_sdk::{Client, SlidingSync, SlidingSyncList, SlidingSyncListBuilder, SlidingSyncMode};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

use tachyon_core::application::error::{BackendError, VerificationError};
use tachyon_core::domain::verification::{SasEmoji, VerificationFlowState};

/// Drives one SAS request against another of the user's devices. The to-device traffic it
/// rides on is the session's `sync_until_verified`, which runs for as long as the device is
/// untrusted.
pub struct VerificationFlowMatrix {
    request: VerificationRequest,
    driver: JoinHandle<()>,
}

impl VerificationFlowMatrix {
    pub(crate) async fn start(device: Device) -> Result<Self, VerificationError> {
        let request = device
            .request_verification_with_methods(vec![VerificationMethod::SasV1])
            .await
            .map_err(|e| VerificationError::Backend(BackendError::Technical(anyhow!("{e}"))))?;

        let driver = tokio::spawn(drive(request.clone()));

        Ok(Self { request, driver })
    }

    pub(crate) fn state(&self) -> VerificationFlowState {
        map_request_state(&self.request.state())
    }

    pub(crate) fn request(&self) -> &VerificationRequest {
        &self.request
    }

    pub(crate) async fn cancel(&self) {
        if let Err(e) = self.request.cancel().await {
            log::warn!("Could not cancel the verification request: {e}");
        }
    }
}

impl Drop for VerificationFlowMatrix {
    fn drop(&mut self) {
        self.driver.abort();
    }
}

pub(crate) fn sas_of(request: &VerificationRequest) -> Option<SasVerification> {
    match request.state() {
        VerificationRequestState::Transitioned { verification } => verification.sas(),
        _ => None,
    }
}

pub(crate) fn map_request_state(state: &VerificationRequestState) -> VerificationFlowState {
    match state {
        VerificationRequestState::Created { .. } | VerificationRequestState::Requested { .. } => {
            VerificationFlowState::Requested
        }
        VerificationRequestState::Ready { .. } => VerificationFlowState::Ready,
        VerificationRequestState::Transitioned { verification } => {
            match verification.clone().sas() {
                Some(sas) => map_sas_state(&sas.state()),
                None => VerificationFlowState::Cancelled {
                    reason: "the other device chose a verification method this bridge does not support"
                        .into(),
                },
            }
        }
        VerificationRequestState::Done => VerificationFlowState::Done,
        VerificationRequestState::Cancelled(info) => VerificationFlowState::Cancelled {
            reason: info.reason().to_owned(),
        },
    }
}

pub(crate) fn map_sas_state(state: &SasState) -> VerificationFlowState {
    match state {
        SasState::Created { .. } | SasState::Started { .. } | SasState::Accepted { .. } => {
            VerificationFlowState::Started
        }
        SasState::KeysExchanged {
            emojis: Some(short_auth_string),
            ..
        } => VerificationFlowState::CompareEmojis {
            emojis: short_auth_string
                .emojis
                .iter()
                .map(|emoji| SasEmoji {
                    symbol: emoji.symbol.to_owned(),
                    description: emoji.description.to_owned(),
                })
                .collect(),
        },
        SasState::KeysExchanged { emojis: None, .. } => VerificationFlowState::Cancelled {
            reason: "the other device does not support emoji verification".into(),
        },
        SasState::Confirmed => VerificationFlowState::AwaitingOtherConfirmation,
        SasState::Done { .. } => VerificationFlowState::Done,
        SasState::Cancelled(info) => VerificationFlowState::Cancelled {
            reason: info.reason().to_owned(),
        },
    }
}

async fn drive(request: VerificationRequest) {
    // eyeball's `subscribe()` only yields versions newer than the one current at subscribe
    // time, so reading `state()` before subscribing would lose a transition landing in between.
    let mut request_changes = pin!(request.changes());
    let mut observed = request.state();
    let mut sas_started = false;

    loop {
        match observed {
            VerificationRequestState::Created { .. } | VerificationRequestState::Requested { .. } => {}
            VerificationRequestState::Ready { their_methods, .. } => {
                if !their_methods.contains(&VerificationMethod::SasV1) {
                    cancel_request(&request).await;
                } else if !sas_started {
                    sas_started = true;
                    if let Err(e) = request.start_sas().await {
                        log::warn!("Could not start the SAS verification: {e}");
                    }
                }
            }
            VerificationRequestState::Transitioned { verification } => match verification.sas() {
                Some(sas) => {
                    drive_sas(&mut request_changes, sas).await;
                    break;
                }
                None => cancel_request(&request).await,
            },
            VerificationRequestState::Done | VerificationRequestState::Cancelled(_) => break,
        }

        match request_changes.next().await {
            Some(next) => observed = next,
            None => break,
        }
    }

}

async fn drive_sas(
    request_changes: &mut (impl Stream<Item = VerificationRequestState> + Unpin),
    sas: SasVerification,
) {
    let mut sas_changes = pin!(sas.changes());
    let mut observed = sas.state();

    loop {
        match observed {
            SasState::Started { .. } => {
                if let Err(e) = sas.accept().await {
                    log::warn!("Could not accept the SAS verification: {e}");
                }
            }
            SasState::KeysExchanged { emojis: None, .. } => {
                if let Err(e) = sas.cancel().await {
                    log::warn!("Could not cancel the emoji-less SAS verification: {e}");
                }
            }
            SasState::Created { .. }
            | SasState::Accepted { .. }
            | SasState::KeysExchanged { emojis: Some(_), .. }
            | SasState::Confirmed => {}
            SasState::Done { .. } | SasState::Cancelled(_) => return,
        }

        loop {
            tokio::select! {
                sas_state = sas_changes.next() => match sas_state {
                    Some(next) => {
                        observed = next;
                        break;
                    }
                    None => return,
                },
                request_state = request_changes.next() => match request_state {
                    Some(VerificationRequestState::Done)
                    | Some(VerificationRequestState::Cancelled(_))
                    | None => return,
                    Some(_) => {}
                },
            }
        }
    }
}

async fn cancel_request(request: &VerificationRequest) {
    if let Err(e) = request.cancel().await {
        log::warn!("Could not cancel the verification request: {e}");
    }
}

/// A sync that carries only to-device events and device-list changes. Its outgoing-request
/// step is also what uploads this device's own keys, which nothing else does before the
/// bridge starts its full sync.
pub(crate) async fn run_to_device_sync(client: Client, cancel: CancellationToken) {
    // The SDK's sync stream terminates on any error, so the sync is rebuilt until cancelled.
    loop {
        tokio::select! {
            _ = cancel.cancelled() => break,
            result = sync_to_device_once(&client) => {
                if let Err(e) = result {
                    log::warn!("To-device sync stopped, restarting: {e}");
                }
            }
        }

        tokio::select! {
            _ = cancel.cancelled() => break,
            _ = tokio::time::sleep(Duration::from_secs(1)) => {}
        }
    }
}

async fn sync_to_device_once(client: &Client) -> matrix_sdk::Result<()> {
    let sliding_sync = build_to_device_only_sliding_sync(client).await?;
    let mut stream = pin!(sliding_sync.sync());

    while let Some(item) = stream.next().await {
        item?;
    }

    Ok(())
}

async fn build_to_device_only_sliding_sync(client: &Client) -> matrix_sdk::Result<SlidingSync> {
    let mut e2ee = E2EE::default();
    e2ee.enabled = Some(true);

    let mut to_device = ToDevice::default();
    to_device.enabled = Some(true);

    client
        .sliding_sync("crypto_list")?
        .add_list(no_room_data_list())
        .share_pos()
        .with_e2ee_extension(e2ee)
        .with_to_device_extension(to_device)
        .build()
        .await
}

fn no_room_data_list() -> SlidingSyncListBuilder {
    let selective_mode = SlidingSyncMode::new_selective().add_range(Range::new(0, 0));

    let mut list_filters = ListFilters::default();
    list_filters.not_room_types = vec![RoomTypeFilter::Default, RoomTypeFilter::Space];

    SlidingSyncList::builder("only_to_device")
        .sync_mode(selective_mode)
        .filters(Some(list_filters))
}

#[cfg(test)]
mod tests {
    use super::*;
    use matrix_sdk::encryption::verification::{Emoji, EmojiShortAuthString};

    #[test]
    fn a_freshly_created_request_is_requested() {
        let state = VerificationRequestState::Created {
            our_methods: vec![VerificationMethod::SasV1],
        };

        assert_eq!(map_request_state(&state), VerificationFlowState::Requested);
    }

    #[test]
    fn a_done_request_is_done() {
        assert_eq!(
            map_request_state(&VerificationRequestState::Done),
            VerificationFlowState::Done
        );
    }

    #[test]
    fn a_confirmed_sas_awaits_the_other_confirmation() {
        assert_eq!(
            map_sas_state(&SasState::Confirmed),
            VerificationFlowState::AwaitingOtherConfirmation
        );
    }

    #[test]
    fn a_done_sas_is_done() {
        let state = SasState::Done {
            verified_devices: vec![],
            verified_identities: vec![],
        };

        assert_eq!(map_sas_state(&state), VerificationFlowState::Done);
    }

    #[test]
    fn exchanged_keys_without_emojis_are_a_cancel() {
        let state = SasState::KeysExchanged {
            emojis: None,
            decimals: (1, 2, 3),
        };

        assert!(matches!(
            map_sas_state(&state),
            VerificationFlowState::Cancelled { .. }
        ));
    }

    #[test]
    fn exchanged_keys_with_emojis_are_compared() {
        let state = SasState::KeysExchanged {
            emojis: Some(EmojiShortAuthString {
                indices: [0; 7],
                emojis: std::array::from_fn(|_| Emoji {
                    symbol: "🐶",
                    description: "Dog",
                }),
            }),
            decimals: (1, 2, 3),
        };

        let VerificationFlowState::CompareEmojis { emojis } = map_sas_state(&state) else {
            panic!("expected CompareEmojis");
        };

        assert_eq!(emojis.len(), 7);
        assert!(emojis.iter().all(|e| e.symbol == "🐶" && e.description == "Dog"));
    }
}
