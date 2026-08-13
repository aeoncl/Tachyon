use crate::audio::{self, siren_wav_duration, MAX_VOICE_CLIP_SIZE};
use crate::tachyon::client::tachyon_client::TachyonClient;
use anyhow::anyhow;
use log::debug;
use matrix_sdk::media::{MediaFormat, MediaRequestParameters};
use matrix_sdk::ruma::events::room::message::AudioMessageEventContent;
use msnp::shared::models::msn_object::{FriendlyName, MSNObjectFactory, MsnObject};
use msnp::shared::models::msn_user::MsnUser;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

const MAX_VOICE_CLIP_DURATION: Duration = siren_wav_duration(MAX_VOICE_CLIP_SIZE);

#[derive(Default)]
pub struct VoiceClipStore {
    clips: DashMap<String, (u64, Vec<u8>)>,
    next_sequence: AtomicU64,
}

impl VoiceClipStore {
    fn insert(&self, sha1d: String, clip: Vec<u8>) {
        let sequence = self.next_sequence.fetch_add(1, Ordering::Relaxed);
        self.clips.insert(sha1d, (sequence, clip));

/*        while self.clips.len() > MAX_PENDING_VOICE_CLIPS {
            let stalest = self.clips.iter().min_by_key(|entry| entry.value().0).map(|entry| entry.key().clone());

            let Some(stalest) = stalest else {
                break;
            };

            self.clips.remove(&stalest);
        }*/
    }

    fn take(&self, sha1d: &str) -> Option<Vec<u8>> {
        self.clips.remove(sha1d).map(|(_, (_, clip))| clip)
    }
}

impl TachyonClient {

    pub async fn prepare_voice_clip(
        &self,
        creator: &MsnUser,
        content: &AudioMessageEventContent,
    ) -> Result<MsnObject, anyhow::Error> {

        if let Some(duration) = content.info.as_ref().and_then(|info| info.duration) {
            if duration > MAX_VOICE_CLIP_DURATION {
                return Err(anyhow!(
                    "Audio message is {:?} long, over the {:?} that fits in a voice clip",
                    duration,
                    MAX_VOICE_CLIP_DURATION
                ));
            }
        }

        let media = self
            .matrix_client()
            .media()
            .get_media_content(
                &MediaRequestParameters { source: content.source.clone(), format: MediaFormat::File },
                false,
            )
            .await?;

        let clip = audio::to_siren_wav(media).await?;

        if clip.len() > MAX_VOICE_CLIP_SIZE {
            return Err(anyhow!(
                "Transcoded voice clip is {} bytes, over the {} bytes WLM accepts",
                clip.len(),
                MAX_VOICE_CLIP_SIZE
            ));
        }

        let msn_object = MSNObjectFactory::get_voice_message(
            &clip,
            creator.get_email_address().to_string(),
            FriendlyName::default(),
        );

        debug!("Prepared voice clip: {} bytes, sha1d {}", clip.len(), &msn_object.sha1d);
        self.inner.voice_clips.insert(msn_object.sha1d.clone(), clip);

        Ok(msn_object)
    }

    pub fn take_voice_clip(&self, sha1d: &str) -> Option<Vec<u8>> {
        self.inner.voice_clips.take(sha1d)
    }
}
