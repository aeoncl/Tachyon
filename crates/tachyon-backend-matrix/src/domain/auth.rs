use matrix_sdk::{
    AuthSession, SessionMeta, SessionTokens,
    authentication::{
        matrix::MatrixSession,
        oauth::{ClientId, UserSession},
    },
    ruma::{OwnedDeviceId, OwnedUserId},
};

/// Everything needed to rebuild a matrix session, in a shape the store can hold.
#[derive(Clone)]
pub struct SessionRestoreData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub user_id: OwnedUserId,
    pub device_id: OwnedDeviceId,
    pub auth_kind: AuthKind,
}

#[derive(Clone)]
pub enum AuthKind {
    Matrix,
    OAuth(OAuthMetadata),
}

#[derive(Clone)]
pub struct OAuthMetadata {
    pub client_id: String,
    /// `None` for public clients, which is what a native app registered against MAS is.
    pub client_secret: Option<String>,
}

impl From<SessionRestoreData> for AuthSession {
    fn from(val: SessionRestoreData) -> Self {
        match val.auth_kind {
            AuthKind::Matrix => Self::Matrix(MatrixSession {
                meta: SessionMeta {
                    user_id: val.user_id,
                    device_id: val.device_id,
                },
                tokens: SessionTokens {
                    access_token: val.access_token,
                    refresh_token: val.refresh_token,
                },
            }),
            AuthKind::OAuth(oauth_metadata) => {
                Self::OAuth(Box::new(matrix_sdk::authentication::oauth::OAuthSession {
                    client_id: ClientId::new(oauth_metadata.client_id),
                    user: UserSession {
                        meta: SessionMeta {
                            user_id: val.user_id,
                            device_id: val.device_id,
                        },
                        tokens: SessionTokens {
                            access_token: val.access_token,
                            refresh_token: val.refresh_token,
                        },
                    },
                }))
            }
        }
    }
}

/// `AuthSession` is `#[non_exhaustive]`, so an SDK upgrade can introduce an auth kind this
/// adapter cannot persist. That is a failure, not something to paper over.
impl TryFrom<AuthSession> for SessionRestoreData {
    type Error = anyhow::Error;

    fn try_from(value: AuthSession) -> Result<Self, Self::Error> {
        match value {
            AuthSession::Matrix(session) => Ok(SessionRestoreData {
                access_token: session.tokens.access_token,
                refresh_token: session.tokens.refresh_token,
                user_id: session.meta.user_id,
                device_id: session.meta.device_id,
                auth_kind: AuthKind::Matrix,
            }),
            AuthSession::OAuth(session) => {
                let client_id = session.client_id.as_str().to_owned();
                Ok(SessionRestoreData {
                    access_token: session.user.tokens.access_token,
                    refresh_token: session.user.tokens.refresh_token,
                    user_id: session.user.meta.user_id,
                    device_id: session.user.meta.device_id,
                    auth_kind: AuthKind::OAuth(OAuthMetadata {
                        client_id,
                        client_secret: None,
                    }),
                })
            }
            other => Err(anyhow::anyhow!(
                "Unsupported matrix auth session kind: {:?}",
                other
            )),
        }
    }
}
