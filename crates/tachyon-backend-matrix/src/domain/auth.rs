use matrix_sdk::{
    AuthSession, SessionMeta, SessionTokens,
    authentication::{
        matrix::MatrixSession,
        oauth::{ClientId, UserSession},
    },
    ruma::{OwnedDeviceId, OwnedUserId},
};
use serde::{Deserialize, Serialize};
use tachyon_core::domain::auth::CredentialBlob;

type CredentialsVersion = u32;
const CREDENTIALS_FORMAT_VERSION: CredentialsVersion = 1;
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionRestoreData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub user_id: OwnedUserId,
    pub device_id: OwnedDeviceId,
    pub auth_kind: AuthKind,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum AuthKind {
    Matrix,
    OAuth(OAuthMetadata),
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct OAuthMetadata {
    pub client_id: String,
    /// `None` for public clients
    pub client_secret: Option<String>,
}


#[derive(Serialize, Deserialize)]
struct CredentialEnvelope {
    version: CredentialsVersion,
    #[serde(flatten)]
    data: SessionRestoreData,
}

impl SessionRestoreData {
    pub fn to_blob(&self) -> Result<CredentialBlob, anyhow::Error> {
        let envelope = CredentialEnvelope {
            version: CREDENTIALS_FORMAT_VERSION,
            data: self.clone(),
        };
        Ok(CredentialBlob::new(serde_json::to_vec(&envelope)?))
    }

    pub fn from_blob(blob: &CredentialBlob) -> Result<Self, anyhow::Error> {
        #[derive(Deserialize)]
        struct VersionProbe {
            version: u32,
        }

        let probe: VersionProbe = serde_json::from_slice(blob.as_bytes())?;
        if probe.version != CREDENTIALS_FORMAT_VERSION {
            anyhow::bail!("unsupported credential blob version {}", probe.version);
        }

        let envelope: CredentialEnvelope = serde_json::from_slice(blob.as_bytes())?;
        Ok(envelope.data)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn oauth_data() -> SessionRestoreData {
        SessionRestoreData {
            access_token: "access".to_string(),
            refresh_token: Some("refresh".to_string()),
            user_id: OwnedUserId::try_from("@aeon:shlasouf.local").unwrap(),
            device_id: OwnedDeviceId::from("DEVICEID"),
            auth_kind: AuthKind::OAuth(OAuthMetadata {
                client_id: "client".to_string(),
                client_secret: None,
            }),
        }
    }

    #[test]
    fn oauth_credentials_round_trip_through_the_blob() {
        let data = oauth_data();

        let restored = SessionRestoreData::from_blob(&data.to_blob().unwrap()).unwrap();

        assert!(restored == data);
    }

    #[test]
    fn matrix_credentials_round_trip_through_the_blob() {
        let data = SessionRestoreData {
            auth_kind: AuthKind::Matrix,
            refresh_token: None,
            ..oauth_data()
        };

        let restored = SessionRestoreData::from_blob(&data.to_blob().unwrap()).unwrap();

        assert!(restored == data);
    }

    #[test]
    fn a_blob_from_the_future_is_rejected() {
        let mut json: serde_json::Value =
            serde_json::from_slice(oauth_data().to_blob().unwrap().as_bytes()).unwrap();
        json["version"] = serde_json::Value::from(2);
        let blob = CredentialBlob::new(serde_json::to_vec(&json).unwrap());

        let Err(err) = SessionRestoreData::from_blob(&blob) else {
            panic!("a version-2 blob must be rejected");
        };
        assert!(err.to_string().contains("version 2"), "{err}");
    }

    #[test]
    fn garbage_is_rejected() {
        let blob = CredentialBlob::new(b"not json".to_vec());

        assert!(SessionRestoreData::from_blob(&blob).is_err());
    }


}
