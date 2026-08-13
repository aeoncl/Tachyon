pub mod ffmpeg;
pub mod siren;

use crate::audio::siren::{SIREN_ENCODED_FRAME_SIZE, SIREN_WAV_HEADER_SIZE};
use std::time::Duration;
use thiserror::Error;

/// WLM 2009 refuses voice clips bigger than this.
pub const MAX_VOICE_CLIP_SIZE: usize = 30_000;

/// Every Siren7 frame carries 320 samples at 16kHz.
const SIREN_FRAME_DURATION_MS: u64 = 20;

#[derive(Error, Debug)]
pub enum AudioConversionError {
    #[error("Could not spawn ffmpeg, is it installed and on the PATH?")]
    FfmpegNotAvailable { source: std::io::Error },
    #[error("ffmpeg exited with a failure: {message}")]
    FfmpegFailed { message: String },
    #[error("libsiren could not encode a frame, error code: {code}")]
    SirenEncodingFailed { code: i32 },
    #[error("The Siren encoding task did not complete")]
    SirenTaskFailed(#[from] tokio::task::JoinError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub async fn to_siren_wav(audio: Vec<u8>) -> Result<Vec<u8>, AudioConversionError> {
    let pcm = ffmpeg::decode_to_siren_pcm(audio).await?;
    //siren::encode blocks and serializes with every other encode, so it stays off the runtime's workers.
    tokio::task::spawn_blocking(move || siren::encode(&pcm)).await?
}

pub async fn from_siren_wav_to_opus_ogg(voice_clip: Vec<u8>) -> Result<Vec<u8>, AudioConversionError> {
    ffmpeg::encode_siren_to_opus(voice_clip).await
}

pub const fn siren_wav_duration(voice_clip_len: usize) -> Duration {
    let frames = voice_clip_len.saturating_sub(SIREN_WAV_HEADER_SIZE) / SIREN_ENCODED_FRAME_SIZE;
    Duration::from_millis(SIREN_FRAME_DURATION_MS * frames as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Round trips a tone through the whole voice clip pipeline. Ignored by default: it needs
    /// ffmpeg on the PATH, built with the msnsiren decoder and libopus.
    /// Run with `cargo test -p tachyon --bin tachyon audio:: -- --ignored`.
    #[tokio::test]
    #[ignore = "requires ffmpeg"]
    async fn voice_clip_round_trip() {
        let tone = ffmpeg::run(
            &["-f", "lavfi", "-i", "sine=frequency=440:duration=3", "-f", "wav", "pipe:1"],
            Vec::new(),
        )
        .await
        .unwrap();

        let clip = to_siren_wav(tone).await.unwrap();

        //3 seconds at 20ms per frame, and small enough to go out as a voice clip.
        assert_eq!(siren_wav_duration(clip.len()), Duration::from_secs(3));
        assert!(clip.len() < MAX_VOICE_CLIP_SIZE, "clip was {} bytes", clip.len());

        //ffmpeg only decodes it if the Siren WAV header we build is well formed.
        let opus = from_siren_wav_to_opus_ogg(clip).await.unwrap();
        assert_eq!(&opus[0..4], b"OggS");
    }
}
