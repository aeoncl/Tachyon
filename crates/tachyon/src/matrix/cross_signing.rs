use log::{info, warn};
use matrix_sdk::encryption::CrossSigningResetAuthType;
use matrix_sdk::encryption::recovery::RecoveryState;
use matrix_sdk::ruma::api::client::uiaa;

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