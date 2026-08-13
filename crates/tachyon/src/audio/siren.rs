use crate::audio::AudioConversionError;
use std::os::raw::{c_int, c_uchar};
use std::sync::Mutex;

/// Bytes of PCM s16le consumed per Siren7 frame (320 samples, mono).
pub const SIREN_FRAME_SIZE: usize = 640;

/// Bytes produced by the encoder per frame at 16kHz.
pub const SIREN_ENCODED_FRAME_SIZE: usize = 40;

/// Byte size of the WAV header libsiren builds, prepended to the encoded frames.
pub const SIREN_WAV_HEADER_SIZE: usize = std::mem::size_of::<SirenWavHeader>();

/// The only sample rate WLM voice clips use.
const SIREN_SAMPLE_RATE: c_int = 16000;

#[repr(C)]
struct RiffHeader {
    riff_id: u32,
    riff_size: u32,
}

#[repr(C)]
struct FmtChunk {
    format: u16,
    channels: u16,
    sample_rate: u32,
    byte_rate: u32,
    block_align: u16,
    bits_per_sample: u16,
}

#[repr(C)]
struct SirenFmtChunk {
    fmt: FmtChunk,
    extra_size: u16,
    dct_length: u16,
}

#[repr(C)]
pub struct SirenWavHeader {
    riff: RiffHeader,
    wave_id: u32,
    fmt_id: u32,
    fmt_size: u32,
    fmt: SirenFmtChunk,
    fact_id: u32,
    fact_size: u32,
    samples: u32,
    data_id: u32,
    data_size: u32,
}

#[repr(C)]
struct StSirenEncoder {
    sample_rate: c_int,
    wav_header: SirenWavHeader,
    context: [f32; 320],
}

extern "C" {
    fn Siren7_NewEncoder(sample_rate: c_int) -> *mut StSirenEncoder;
    fn Siren7_CloseEncoder(encoder: *mut StSirenEncoder);
    fn Siren7_EncodeFrame(
        encoder: *mut StSirenEncoder,
        data_in: *mut c_uchar,
        data_out: *mut c_uchar,
    ) -> c_int;
}

/// libsiren keeps its lookup tables and per-frame scratch buffers in file scope statics, so no two
/// encoders may run at the same time, not even from different encoder handles.
static ENCODER_LOCK: Mutex<()> = Mutex::new(());

/// Encodes mono 16kHz PCM s16le into a Siren7 WAV voice clip.
///
/// This blocks the calling thread and serializes with every other caller: run it off the async
/// runtime's worker threads.
pub fn encode(pcm_s16le: &[u8]) -> Result<Vec<u8>, AudioConversionError> {
    let _guard = ENCODER_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());

    let encoder = unsafe { Siren7_NewEncoder(SIREN_SAMPLE_RATE) };
    let result = encode_with(encoder, pcm_s16le);
    unsafe { Siren7_CloseEncoder(encoder) };
    result
}

fn encode_with(
    encoder: *mut StSirenEncoder,
    pcm_s16le: &[u8],
) -> Result<Vec<u8>, AudioConversionError> {
    let frame_count = pcm_s16le.len().div_ceil(SIREN_FRAME_SIZE);

    let mut frame_in = [0u8; SIREN_FRAME_SIZE];
    let mut frame_out = [0u8; SIREN_ENCODED_FRAME_SIZE];
    let mut data = Vec::with_capacity(frame_count * SIREN_ENCODED_FRAME_SIZE);

    for chunk in pcm_s16le.chunks(SIREN_FRAME_SIZE) {
        //The encoder always reads a whole 320 sample frame, so the trailing chunk is padded with silence.
        frame_in[..chunk.len()].copy_from_slice(chunk);
        frame_in[chunk.len()..].fill(0);

        let code = unsafe { Siren7_EncodeFrame(encoder, frame_in.as_mut_ptr(), frame_out.as_mut_ptr()) };
        if code != 0 {
            return Err(AudioConversionError::SirenEncodingFailed { code });
        }

        data.extend_from_slice(&frame_out);
    }

    //libsiren keeps the header's RiffSize, Samples and DataSize up to date as frames are encoded,
    //so it is only valid once every frame went through.
    let mut out = Vec::with_capacity(SIREN_WAV_HEADER_SIZE + data.len());
    out.extend_from_slice(unsafe { header_as_bytes(encoder) });
    out.append(&mut data);

    Ok(out)
}

unsafe fn header_as_bytes<'a>(encoder: *mut StSirenEncoder) -> &'a [u8] {
    std::slice::from_raw_parts(
        std::ptr::addr_of!((*encoder).wav_header) as *const u8,
        SIREN_WAV_HEADER_SIZE,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header_layout_matches_libsiren() {
        assert_eq!(SIREN_WAV_HEADER_SIZE, 60);
        assert_eq!(std::mem::size_of::<SirenFmtChunk>(), 20);
    }

    #[test]
    fn encodes_a_whole_number_of_frames() {
        //Half a second of silence.
        let pcm = vec![0u8; SIREN_FRAME_SIZE * 25];

        let encoded = encode(&pcm).unwrap();

        assert_eq!(encoded.len(), SIREN_WAV_HEADER_SIZE + SIREN_ENCODED_FRAME_SIZE * 25);
        assert_eq!(&encoded[0..4], b"RIFF");
        assert_eq!(&encoded[8..12], b"WAVE");
    }

    #[test]
    fn pads_a_partial_trailing_frame() {
        let pcm = vec![0u8; SIREN_FRAME_SIZE + 1];

        let encoded = encode(&pcm).unwrap();

        assert_eq!(encoded.len(), SIREN_WAV_HEADER_SIZE + SIREN_ENCODED_FRAME_SIZE * 2);
    }

    #[test]
    fn header_reports_the_encoded_size() {
        let pcm = vec![0u8; SIREN_FRAME_SIZE * 3];

        let encoded = encode(&pcm).unwrap();

        let data_size = u32::from_le_bytes(encoded[56..60].try_into().unwrap()) as usize;
        let riff_size = u32::from_le_bytes(encoded[4..8].try_into().unwrap()) as usize;
        let samples = u32::from_le_bytes(encoded[48..52].try_into().unwrap()) as usize;

        assert_eq!(data_size, SIREN_ENCODED_FRAME_SIZE * 3);
        assert_eq!(riff_size, encoded.len() - 8);
        assert_eq!(samples, 320 * 3);
    }
}
