use std::any::Any;
use matrix_sdk::Client;
use matrix_sdk::ruma::api::client::error::ErrorKind;
use tokio_util::sync::CancellationToken;
use tachyon_core::application::error::BackendError;
use tachyon_core::application::ports::BackendSession;
use crate::domain::auth::SessionRestoreData;

pub struct BackendSessionMatrix {
    client: Client,
    tasks_cancellation_token: CancellationToken,
}

impl BackendSessionMatrix {

    pub(crate) fn new(client: Client, cancellation_token: CancellationToken) -> Self {
        Self {
            client,
            tasks_cancellation_token: cancellation_token,
        }
    }

    /// You need to subscribe to Session Token change before restoring Auth
    pub async fn restore(
        client: matrix_sdk::Client,
        cancellation_token: CancellationToken,
        session_restore_data: SessionRestoreData,
    ) -> Result<Self, BackendError> {
        if let Err(err) = client.restore_session(session_restore_data).await {
            cancellation_token.cancel();

            return Err(BackendError::CannotRestoreLogin(format!("{}", err)));
        }

        match client.whoami().await {
            Ok(_) => Ok(BackendSessionMatrix {
                client,
                tasks_cancellation_token: cancellation_token.clone(),
            }),
            Err(e) => {
                cancellation_token.cancel();

                let Some(api_error) = e.client_api_error_kind() else {
                    return Err(BackendError::Technical(anyhow::anyhow!(e)));
                };

                match api_error {
                    ErrorKind::Forbidden { .. } => Err(BackendError::LoggedOut),
                    ErrorKind::Unauthorized => Err(BackendError::LoggedOut),
                    ErrorKind::UnknownToken { soft_logout } => {
                        if *soft_logout {
                            Err(BackendError::SoftLoggedOut)
                        } else {
                            Err(BackendError::LoggedOut)
                        }
                    }
                    _ => Err(BackendError::Technical(anyhow::anyhow!(e))),
                }
            }
        }
    }

    /// FIXME: Remove this after the refactor is done
    pub fn matrix_client(&self) -> &Client {
        &self.client
    }
}

impl BackendSession for BackendSessionMatrix {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for BackendSessionMatrix {
    fn drop(&mut self) {
        self.tasks_cancellation_token.cancel();
    }
}
