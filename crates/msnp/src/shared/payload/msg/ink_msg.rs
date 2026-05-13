
/*
 SB << | MSG 18 N 396MIME-Version: 1.0
Content-Type: application/x-ms-ink

base64:APUBHAOAgAQdBNIF3AEDBEgRRWQZFDIIAIAoAjiGK0IzCADAFgK3bStCFauq00GrqtNBAAAAPgCAqr4KIyaC/gFT+AVVSwILm42ImmbJ2ACC/MH5jaBJSVLCwiTeauaAChYQg/wJH4EXE8uesxKJtICC/FH4plAACiEggv4Du/gO+wMVLKDO5WaTQIP4Y+GVSISmEkzG6HXoXoAKIhuD/CrfhW0ZwIEKmOHLe+Tc8oL8LfhfN5sm+OpbmptbZNAKNz+C/gjj+COaJUBRYRMkxidu7aGywkR6SZ8eeYL8gfkfzNLzu/E31Mkxvjz47xoruje5iwixYCg=

 */
use anyhow::anyhow;
use base64::Engine;
use base64::engine::general_purpose;
use crate::msnp::error::PayloadError;
use crate::shared::models::b64_string::Base64String;
use crate::shared::payload::msg::gif_msg::GifMsgPayload;
use crate::shared::payload::msg::raw_msg_payload::{MsgContentType, RawMsgPayload};
use crate::shared::traits::{IntoBytes, TryFromRawMsgPayload};

//The content is encoded using Microsoft Ink Serialization Format. It's used if the RendersIfs capability is set for the interlocuter.
// Spec: https://download.microsoft.com/download/0/B/E/0BE8BDD7-E5E8-422A-ABFD-4342ED7AD886/InkSerializedFormat(ISF)Specification.pdf
// This binary format seems very annoying to work with, i have found an implementation on github: https://github.com/blu-base/libisf-qt
// emesene implemented this also.
pub struct InkMessagePayload {
    ifs_bytes: Vec<u8>
}

impl TryFromRawMsgPayload for InkMessagePayload{
    type Err = PayloadError;

    fn try_from_raw(raw_msg_payload: RawMsgPayload) -> Result<Self, Self::Err>
    where
        Self: Sized
    {
        let str = raw_msg_payload.get_body_as_string()?;
        let base64 = str.strip_prefix("base64:").ok_or(PayloadError::StringPayloadParsingError { payload: str.to_string(), source: anyhow!("ink body did not start with `base64:` prefix") })?;
        let decoded = general_purpose::STANDARD
            .decode(&base64).map_err(|e| PayloadError::AnyError(anyhow!("Could not base64 decoded `ink` body: {}", e)))?;

        Ok(Self {
            ifs_bytes: decoded,
        })
    }
}

impl IntoBytes for InkMessagePayload {
    fn into_bytes(self) -> Vec<u8> {
        let mut out = RawMsgPayload::new(MsgContentType::Ink, false);
        let encoded = general_purpose::STANDARD.encode(self.ifs_bytes);
        out.set_body_string(format!("base64:{}", &encoded));
        out.into_bytes()
    }
}