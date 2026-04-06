use std::time::Duration;
use futures_util::StreamExt;
use log::{info, warn};
use matrix_sdk::encryption::CrossSigningResetAuthType;
use matrix_sdk::encryption::recovery::RecoveryState;
use matrix_sdk::ruma::api::client::uiaa;
use matrix_sdk::{sliding_sync, Client, SlidingSync, SlidingSyncList, SlidingSyncListBuilder, SlidingSyncMode};
use matrix_sdk::encryption::verification::{SasState, SasVerification, Verification, VerificationRequest, VerificationRequestState};
use matrix_sdk::ruma::api::client::sync::sync_events::v5::request::{ListFilters, ToDevice, E2EE};
use matrix_sdk::ruma::directory::RoomTypeFilter;
use matrix_sdk::ruma::events::key::verification::request::ToDeviceKeyVerificationRequestEvent;
use matrix_sdk::sliding_sync::Range;
use tokio::time::sleep;

pub async fn check_device_is_crossed_signed(client: &matrix_sdk::Client) -> Result<bool, anyhow::Error> {

    let user_id = client.user_id().unwrap();
    client.encryption().request_user_identity(&user_id).await?;

    let own_device = client.encryption().get_own_device().await?;

    match own_device {
        None => Ok(false),
        Some(device) => Ok(device.is_cross_signed_by_owner()),
    }

}

//There is no way to check the recover status yet without syncing. So we do this instead.
pub async fn check_secret_storage_state(client: &matrix_sdk::Client) -> Result<bool, anyhow::Error> {
    client.encryption().secret_storage().is_enabled().await.map_err(|e| anyhow::anyhow!("Failed to check secret storage state: {}", e))
}

pub async fn restore_from_recovery_key(
    client: &matrix_sdk::Client,
    recovery_key: &str,
) -> Result<(), anyhow::Error> {
    let recovery = client.encryption().recovery();

    info!("Attempting to recover secrets from secret storage…");

    recovery.recover(recovery_key).await?;

    info!("Recovery complete – secrets have been imported");

    let status = client
        .encryption()
        .cross_signing_status()
        .await
        .expect("Should be able to query cross-signing status");
    info!("Cross-signing status after recovery: {status:?}");

    Ok(())
}

pub async fn cross_sign_device(
    client: &matrix_sdk::Client,
    password: Option<String>,
) -> Result<(), anyhow::Error> {
    let encryption = client.encryption();
    let user_id = client.user_id().unwrap().to_owned();

    info!("Bootstrapping cross-signing…");

    // First attempt – will fail with a UIA challenge if keys already exist.
    match encryption.bootstrap_cross_signing_if_needed(None).await {
        Ok(()) => {
            info!("Cross-signing bootstrapped (no UIA required)");
            return Ok(());
        }
        Err(err) => {
            // If the server returned a UIA challenge, handle it.
            if let Some(response) = err.as_uiaa_response() {
                let password = password.ok_or_else(|| {
                    anyhow::anyhow!(
                        "Server requires user-interactive auth but no password was provided"
                    )
                })?;

                let mut auth = uiaa::Password::new(user_id.into(), password);
                auth.session = response.session.clone();

                encryption
                    .bootstrap_cross_signing_if_needed(Some(uiaa::AuthData::Password(auth)))
                    .await?;

                info!("Cross-signing bootstrapped (UIA completed)");
            } else {
                return Err(err.into());
            }
        }
    }

    Ok(())
}

pub async fn reset_cross_signing_identity(
    client: &matrix_sdk::Client,
    password: Option<String>,
) -> Result<(), anyhow::Error> {
    let encryption = client.encryption();
    let user_id = client.user_id().unwrap().to_owned();

    warn!("Resetting cross-signing identity – all existing verifications will be invalidated");

    if let Some(handle) = encryption.recovery().reset_identity().await? {
        match handle.auth_type() {
            CrossSigningResetAuthType::Uiaa(uiaa_info) => {
                let password = password.ok_or_else(|| {
                    anyhow::anyhow!(
                        "Server requires user-interactive auth but no password was provided"
                    )
                })?;

                let mut auth = uiaa::Password::new(user_id.into(), password);
                auth.session = uiaa_info.session.clone();

                handle
                    .reset(Some(uiaa::AuthData::Password(auth)))
                    .await?;
            }
            CrossSigningResetAuthType::OAuth(oauth_info) => {
                return Err(anyhow::anyhow!(
                    "OAuth approval required – visit: {}",
                    oauth_info.approval_url
                ));
            }
        }
    }

    info!("Cross-signing identity has been reset");
    Ok(())
}

pub async fn enable_backup_and_recovery(
    client: &matrix_sdk::Client,
    passphrase: Option<&str>,
) -> Result<String, anyhow::Error> {
    let recovery = client.encryption().recovery();

    info!("Enabling backup and recovery…");

    let mut builder = recovery.enable().wait_for_backups_to_upload();

    if let Some(passphrase) = passphrase {
        builder = builder.with_passphrase(passphrase);
    }

    let recovery_key = builder.await?;

    info!("Backup and recovery enabled – store the recovery key safely");

    Ok(recovery_key)
}

fn no_room_data_list() -> SlidingSyncListBuilder {

    let selective_mode = SlidingSyncMode::new_selective()
        .add_range(Range::new(0, 0));

    let mut list_filters = ListFilters::default();
    list_filters.not_room_types = vec![RoomTypeFilter::Default, RoomTypeFilter::Space];

    SlidingSyncList::builder("only_to_device")
        .sync_mode(selective_mode)
        .filters(Some(list_filters))
}

pub async fn build_to_device_only_sliding_sync(matrix_client: &Client) -> Result<SlidingSync, anyhow::Error> {
    let mut e2ee = E2EE::default();
    e2ee.enabled = Some(true);

    let mut to_device = ToDevice::default();
    to_device.enabled = Some(true);

    let sliding_sync_builder = matrix_client
        .sliding_sync("no_room_data_list")?
        .add_list(no_room_data_list())
        .share_pos()
        .with_e2ee_extension(e2ee)
        .with_to_device_extension(to_device);
    Ok(sliding_sync_builder.build().await?)
}


pub async fn cross_sign_sync_task(
    client: &matrix_sdk::Client,
    mut kill_signal_rcv: tokio::sync::broadcast::Receiver<()>,

) -> Result<(), anyhow::Error> {
    let client = client.clone();
    tokio::spawn(async move {
        let sliding_sync = build_to_device_only_sliding_sync(&client).await.unwrap();

        sliding_sync.add_list(no_room_data_list()).await.unwrap();
        let mut sync_stream = Box::pin(sliding_sync.sync());

        loop {
            tokio::select! {
                _ = kill_signal_rcv.recv() => {
                    sliding_sync.stop_sync().unwrap();
                    info!("Gracefully exit cross_sign sync loop...");
                    break;
                }
                sync_response = sync_stream.next()=> {
                    match sync_response {

                        Some(Ok(update_summary)) => {

                        }

                        Some(Err(err)) => {

                        }

                        None => {

                        }
                    }
                }
            }
        }
    });

    todo!()
}