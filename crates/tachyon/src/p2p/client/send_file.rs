use anyhow::anyhow;
use log::info;
use matrix_sdk::attachment::{AttachmentConfig, AttachmentInfo, BaseFileInfo};
use mime::Mime;
use ruma::{RoomId, UInt};
use msnp::p2p::v2::raw_p2p_payload::RawP2PPayload;
use crate::p2p::client::session::SessionId;
use crate::tachyon::client::tachyon_client::TachyonClient;

impl TachyonClient {
    pub(crate) async fn send_file_buffered(&self, session_id: SessionId, mut p2p_payload: RawP2PPayload, room_id: &RoomId, file_name: &str, file_size: usize) -> Result<(), anyhow::Error> {
        let contains_key =  self.inner.chunked_uploads.contains_key(&session_id);
        let is_chunked_packet = p2p_payload.is_chunked_packet();
        let is_file_chunked = self.inner.chunked_uploads.contains_key(&session_id) || p2p_payload.is_chunked_packet();

        info!("DEBUG CHUNKK: contains_key: {} is_chunked_packet: {} ", contains_key, is_chunked_packet);


        if is_file_chunked {

            if p2p_payload.get_missing_bytes_count() > 0 {
                //Still chunked
                self.inner.chunked_uploads.entry(session_id).or_default().push(p2p_payload);

            } else {
                let (_, mut chunks) = self.inner.chunked_uploads.remove(&session_id).ok_or(anyhow!("Missing chunks in map. SessionId: {}", session_id))?;
                let mut reformed_bytes = {
                    let mut out = Vec::new();
                    for mut chunk in chunks {
                        out.append(&mut chunk.payload);
                    }
                    out.append(&mut p2p_payload.payload);
                    out
                };

                let room = self.matrix_client().get_room(room_id).ok_or(anyhow!("Could not find room to send file to. RoomId: {} SessionId: {}", room_id, session_id))?;
                let mime: mime::Mime = "audio/wave".parse().unwrap();
                let attachment = room.send_attachment(file_name, &mime, reformed_bytes, AttachmentConfig::default());
                attachment.await.map_err(|e| anyhow!(e))?;
            }
        } else {

            let room = self.matrix_client().get_room(room_id).ok_or(anyhow!("Could not find room to send file to. RoomId: {} SessionId: {}", room_id, session_id))?;
            let mime: mime::Mime = "audio/wave".parse().unwrap();
            let config = AttachmentConfig::new().info(AttachmentInfo::File(BaseFileInfo{ size:  Some(UInt::new(file_size as u64).unwrap()) }));

            let attachment = room.send_attachment(file_name,&mime, p2p_payload.payload, config);

            attachment.await.map_err(|e| anyhow!(e))?;
        }

        Ok(())
    }
}
