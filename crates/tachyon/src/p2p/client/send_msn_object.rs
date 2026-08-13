use crate::audio::{self, siren_wav_duration};
use crate::matrix::extensions::msn_user_resolver::ToMsnUser;
use crate::p2p::client::session::{SendMsnObjectContent, SessionId, SessionType};
use crate::tachyon::client::tachyon_client::TachyonClient;
use anyhow::anyhow;
use log::{info, warn};
use matrix_sdk::attachment::{AttachmentConfig, AttachmentInfo, BaseAudioInfo};
use matrix_sdk::Room;
use mime::Mime;
use msnp::p2p::v2::raw_p2p_payload::RawP2PPayload;
use msnp::shared::models::msn_object::{MsnObject, MsnObjectType};
use ruma::{RoomId, UInt};
use std::str::FromStr;

impl TachyonClient {

    pub async fn request_msn_object(&self, room: &Room, msn_object: MsnObject) -> Result<(), anyhow::Error> {
        let requester = room.to_msn_user_lazy().await?;
        let owner = self.own_user();

        let transport = self.get_or_create_transport(room.room_id(), &requester);
        let (session_id, session) = self.create_session_with_random_id(transport, SessionType::SendMsnObject(SendMsnObjectContent {
            room_id: room.room_id().to_owned(),
            requester,
            owner,
            msn_object,
        }));

        info!("Requesting MSNObject from the client on session {}", session_id);
        session.receive_invite().await;

        Ok(())
    }

    pub(crate) async fn send_msn_object_buffered(&self, session_id: SessionId, p2p_payload: RawP2PPayload, content: &SendMsnObjectContent) -> Result<(), anyhow::Error> {
        if p2p_payload.tf.is_metadata() {
            return Ok(());
        }

        let expected_size = content.msn_object.size;

        let received_len: usize = {
            let mut chunks = self.inner.chunked_uploads.entry(session_id).or_default();
            chunks.push(p2p_payload);
            chunks.iter().map(|chunk| chunk.payload.len()).sum()
        };

        if received_len < expected_size {
            return Ok(());
        }

        let (_, chunks) = self.inner.chunked_uploads.remove(&session_id).ok_or(anyhow!("Missing chunks in map. SessionId: {}", session_id))?;

        let mut voice_clip = Vec::with_capacity(received_len);
        for mut chunk in chunks {
            voice_clip.append(&mut chunk.payload);
        }

        if voice_clip.len() != expected_size {
            warn!("MSNObject session {} received {} bytes but the invite announced {}", session_id, voice_clip.len(), expected_size);
        }

        match content.msn_object.obj_type {
            MsnObjectType::VoiceClip => self.send_voice_clip_to_matrix(session_id, &content.room_id, voice_clip).await,
            ref obj_type => Err(anyhow!("Received an MSNObject we don't know how to forward to Matrix: {:?}", obj_type)),
        }
    }

    async fn send_voice_clip_to_matrix(&self, session_id: SessionId, room_id: &RoomId, voice_clip: Vec<u8>) -> Result<(), anyhow::Error> {
        let duration = siren_wav_duration(voice_clip.len());
        let opus = audio::from_siren_wav_to_opus_ogg(voice_clip).await?;

        let room = self.matrix_client().get_room(room_id).ok_or(anyhow!("Could not find room to send voice clip to. RoomId: {} SessionId: {}", room_id, session_id))?;

        let info = AttachmentInfo::Voice(BaseAudioInfo {
            duration: Some(duration),
            size: UInt::new(opus.len() as u64),
            waveform: None,
        });

        let mime = Mime::from_str("audio/ogg")?;

        let len = opus.len();
        let resp = room.send_attachment("Voice Message.ogg", &mime, opus, AttachmentConfig::new().info(info)).await.map_err(|e| anyhow!(e))?;
        info!("Uploaded voice clip session {}: {} bytes ({:?}) -> event {}", session_id, len, duration, resp.event_id);

        Ok(())
    }
}
