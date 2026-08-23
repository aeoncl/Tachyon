use matrix_sdk::{
    AuthSession, SessionMeta, SessionTokens,
    authentication::{
        matrix::MatrixSession,
        oauth::{ClientId, UserSession},
    },
    ruma::{OwnedDeviceId, OwnedUserId},
};

pub struct SessionRestoreData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub user_id: OwnedUserId,
    pub device_id: OwnedDeviceId,
    pub auth_kind: AuthKind,
}

pub enum AuthKind {
    Matrix,
    OAuth(OAuthMetadata),
}

pub struct OAuthMetadata {
    pub client_id: String,
    pub client_secret: String,
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
