use anyhow::anyhow;
use log::{info, warn};
use matrix_sdk::attachment::{AttachmentConfig, AttachmentInfo, BaseAudioInfo, BaseFileInfo, BaseImageInfo, BaseVideoInfo};
use mime::Mime;
use ruma::{RoomId, UInt};
use msnp::p2p::v2::raw_p2p_payload::RawP2PPayload;
use crate::msn::application::p2p::client::session::SessionId;
use crate::tachyon::client::tachyon_client::TachyonClient;

impl TachyonClient {
    pub(crate) async fn send_file_buffered(&self, session_id: SessionId, p2p_payload: RawP2PPayload, room_id: &RoomId, file_name: &str, file_size: usize) -> Result<(), anyhow::Error> {
        //Data preparation packets (T=0) carry no file bytes.
        if p2p_payload.tf.is_metadata() {
            return Ok(());
        }

        let received_len: usize = {
            let mut chunks = self.inner.chunked_uploads.entry(session_id).or_default();
            chunks.push(p2p_payload);
            chunks.iter().map(|chunk| chunk.payload.len()).sum()
        };

        if received_len < file_size {
            return Ok(());
        }

        let (_, chunks) = self.inner.chunked_uploads.remove(&session_id).ok_or(anyhow!("Missing chunks in map. SessionId: {}", session_id))?;

        let mut reformed_bytes = Vec::with_capacity(received_len);
        for mut chunk in chunks {
            reformed_bytes.append(&mut chunk.payload);
        }

        if reformed_bytes.len() != file_size {
            warn!("File transfer session {} received {} bytes but the invite announced {}", session_id, reformed_bytes.len(), file_size);
        }

        let room = self.matrix_client().get_room(room_id).ok_or(anyhow!("Could not find room to send file to. RoomId: {} SessionId: {}", room_id, session_id))?;

        let mime = mime_guess::from_path(file_name).first().unwrap_or(mime::APPLICATION_OCTET_STREAM);
        let config = AttachmentConfig::new().info(attachment_info(&mime, reformed_bytes.len()));

        let len = reformed_bytes.len();
        let resp = room.send_attachment(file_name, &mime, reformed_bytes, config).await.map_err(|e| anyhow!(e))?;
        info!("Uploaded file transfer session {}: {} bytes ({}) -> event {}", session_id, len, mime, resp.event_id);

        Ok(())
    }
}

/// The sdk only keeps `info` fields whose `AttachmentInfo` variant matches the
/// event type it derives from the mime, so build the matching variant.
fn attachment_info(mime: &Mime, size: usize) -> AttachmentInfo {
    let size = UInt::new(size as u64);
    match mime.type_() {
        mime::IMAGE => AttachmentInfo::Image(BaseImageInfo { size, ..Default::default() }),
        mime::AUDIO => AttachmentInfo::Audio(BaseAudioInfo { size, ..Default::default() }),
        mime::VIDEO => AttachmentInfo::Video(BaseVideoInfo { size, ..Default::default() }),
        _ => AttachmentInfo::File(BaseFileInfo { size }),
    }
}
