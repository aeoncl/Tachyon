use crate::audio::AudioConversionError;
use std::process::Stdio;
use std::str::from_utf8;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// Decodes arbitrary audio into the mono 16kHz PCM s16le libsiren expects.
pub async fn decode_to_siren_pcm(audio: Vec<u8>) -> Result<Vec<u8>, AudioConversionError> {
    run(
        &[
            "-i", "pipe:0",
            "-ac", "1",
            "-ar", "16000",
            "-f", "s16le",
            "-acodec", "pcm_s16le",
            "pipe:1",
        ],
        audio,
    )
    .await
}

//TODO replace this with compiled FFMPEG lib to remove dependency from the PATH

/// Decodes a Siren7 WAV voice clip and re-encodes it as Ogg Opus.
pub async fn encode_siren_to_opus(voice_clip: Vec<u8>) -> Result<Vec<u8>, AudioConversionError> {
    run(
        &[
            "-f", "wav",
            "-c:a", "msnsiren",
            "-i", "pipe:0",
            "-ac", "1",
            "-b:a", "16K",
            "-c:a", "libopus",
            "-f", "ogg",
            "pipe:1",
        ],
        voice_clip,
    )
    .await
}

pub(super) async fn run(args: &[&str], input: Vec<u8>) -> Result<Vec<u8>, AudioConversionError> {
    let mut child = Command::new("ffmpeg")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .args(args)
        .kill_on_drop(true)
        .spawn()
        .map_err(|source| AudioConversionError::FfmpegNotAvailable { source })?;

    let mut stdin = child.stdin.take().expect("ffmpeg stdin to be piped");

    //ffmpeg won't drain stdin until we read stdout, so the write has to run alongside wait_with_output.
    //A broken pipe here just means ffmpeg gave up early: the real error comes back on stderr.
    tokio::spawn(async move {
        let _ = stdin.write_all(&input).await;
        let _ = stdin.shutdown().await;
    });

    let output = child.wait_with_output().await?;

    if !output.status.success() {
        return Err(AudioConversionError::FfmpegFailed {
            message: from_utf8(&output.stderr).unwrap_or("<non utf-8 ffmpeg output>").to_string(),
        });
    }

    Ok(output.stdout)
}
