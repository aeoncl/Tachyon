use anyhow::anyhow;
use base64::Engine;
use base64::engine::general_purpose;
use crate::msnp::error::PayloadError;
use crate::shared::models::b64_string::Base64String;
use crate::shared::payload::msg::raw_msg_payload::{MsgContentType, RawMsgPayload};
use crate::shared::traits::{IntoBytes, IntoRawMsgPayload, TryFromRawMsgPayload};
/*

Packet was chunked

SB << | MSG 30 N 1311MIME-Version: 1.0
Content-Type: image/gif
Message-ID: {CDDB9913-F90B-4162-B63C-7918D3AE3953}
Chunks: 3

base64:R0lGODlhxwBBAPcAAAAAAAAAMwAAZgAAmQAAzAAA/wArAAArMwArZgArmQArzAAr/wBVAABVMwBVZgBVmQBVzABV/wCAAACAMwCAZgCAmQCAzACA/wCqAACqMwCqZgCqmQCqzACq/wDVAADVMwDVZgDVmQDVzADV/wD/AAD/MwD/ZgD/mQD/zAD//zMAADMAMzMAZjMAmTMAzDMA/zMrADMrMzMrZjMrmTMrzDMr/zNVADNVMzNVZjNVmTNVzDNV/zOAADOAMzOAZjOAmTOAzDOA/zOqADOqMzOqZjOqmTOqzDOq/zPVADPVMzPVZjPVmTPVzDPV/zP/ADP/MzP/ZjP/mTP/zDP//2YAAGYAM2YAZmYAmWYAzGYA/2YrAGYrM2YrZmYrmWYrzGYr/2ZVAGZVM2ZVZmZVmWZVzGZV/2aAAGaAM2aAZmaAmWaAzGaA/2aqAGaqM2aqZmaqmWaqzGaq/2bVAGbVM2bVZmbVmWbVzGbV/2b/AGb/M2b/Zmb/mWb/zGb//5kAAJkAM5kAZpkAmZkAzJkA/5krAJkrM5krZpkrmZkrzJkr/5lVAJlVM5lVZplVmZlVzJlV/5mAAJmAM5mAZpmAmZmAzJmA/5mqAJmqM5mqZpmqmZmqzJmq/5nVAJnVM5nVZpnVmZnVzJnV/5n/AJn/M5n/Zpn/mZn/zJn//8wAAMwAM8wAZswAmcwAzMwA/8wrAMwrM8wrZswrmcwrzMwr/8xVAMxVM8xVZsxVmcxVzMxV/8yAAMyAM8yAZsyAmcyAzMyA/8yqAMyqM8yqZsyqmcyqzMyq/8zVAMzVM8zVZszVmczVzMzV/8z/AMz/M8z/Zsz/mcz/zMz///8AAP8AM/8AZv8Amf8AzP8A//8rAP8rM/8rZv8rmf8rzP8r//9VAP9VM/9VZv9Vmf9VzP9V//+AAP+AM/+AZv+Amf+AzP+A//+qAP+qM/+qZv+qmf+qzP+q///VAP/VM//VZv/Vmf/VzP/V////AP//M///Zv//mf//zP///wAAAAAAAAAAAAAAACH5BAEAAPsALAAAAADHAEEAAAj/APcJHEiwoMGB7w4qXMiwocOHECNKnEixokWF3a51u8ixo8ePIEOGfHetZEKRKFOqXMmSIUmTLWNSrCezJsuX107a3Hkw4zWeQD3i1BmU59CiSCceTboT5zimUBsujRo
 */

//Gif only render in WLM2009 using the original WLM encoder for now. I wasn't able to find what exactly is preventing the gifs to appear but, the originals are GIF89a with no extensions. Color encoding might be weird.
pub struct GifMsgPayload {
    pub gif_bytes: Vec<u8>
}

impl GifMsgPayload {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            gif_bytes: bytes,
        }
    }
}

impl TryFromRawMsgPayload for GifMsgPayload {
    type Err = PayloadError;

    fn try_from_raw(raw_msg_payload: RawMsgPayload) -> Result<Self, Self::Err>
    where
        Self: Sized
    {
        let str = raw_msg_payload.get_body_as_string()?;
        let base64 = str.strip_prefix("base64:").ok_or(PayloadError::StringPayloadParsingError { payload: str.to_string(), source: anyhow!("image/gif body did not start with `base64:` prefix") })?;
        let decoded = general_purpose::STANDARD
            .decode(&base64).map_err(|e| PayloadError::AnyError(anyhow!("Could not base64 decoded `image/gif` body: {}", e)))?;

        Ok(Self {
            gif_bytes: decoded,
        })
    }
}

impl IntoRawMsgPayload for GifMsgPayload {
    fn into_raw(self) -> RawMsgPayload {
        let mut out = RawMsgPayload::new(MsgContentType::Gif, false);
        let encoded = general_purpose::STANDARD.encode(self.gif_bytes);
        out.set_body_string(format!("base64:{}", &encoded));
        out
    }
}

impl IntoBytes for GifMsgPayload {
    fn into_bytes(self) -> Vec<u8> {
        self.into_raw().into_bytes()
    }
}