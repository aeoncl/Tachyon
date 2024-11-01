use std::io::Write;
use std::process::Stdio;
use std::str::from_utf8;

use byteorder::ByteOrder;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::{Siren7_CloseEncoder, Siren7_EncodeFrame, Siren7_NewEncoder, SirenWavHeader};
use crate::models::conversion::error::ConversionError;

lazy_static_include_bytes! {
            MSNWAV => "assets/sound/aeoncl_02_11_23@12_45_58.wav"
        }


static SIREN_FRAME_SIZE : usize = 640usize;

pub async fn convert_siren_to_opus(audio: Vec<u8>) -> Result<Vec<u8>, ConversionError> {
    let mut child = Command::new("ffmpeg")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg("-f")
        .arg("wav")
        .arg("-c:a")
        .arg("msnsiren")
        .arg("-i")
        .arg("pipe:0")
        .arg("-ac")
        .arg("1")
        .arg("-b:a")
        .arg("16K")
        .arg("-c:a")
        .arg("libopus")
        .arg("-f")
        .arg("ogg")
        .arg("pipe:1")
        .kill_on_drop(true)
        .spawn()?;

    let mut stdin = child.stdin.take().expect("FFMPEG stdin to be present");

    tokio::spawn(async move{
        stdin.write_all(&audio).await.expect("Failed to write to stdin");
    });

    let output = child.wait_with_output().await?;

    if !output.status.success() {
        return Err(ConversionError::FFMPEG_OUTPUT {message: from_utf8(&output.stderr)?.to_string() });
    }
    return Ok(output.stdout);
}

pub async fn convert_incoming_audio_message(audio: Vec<u8>) -> Result<Vec<u8>, ConversionError> {
    let mut child = Command::new("ffmpeg")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg("-f")
        .arg("ogg")
        .arg("-i")
        .arg("pipe:0")
        .arg("-ac")
        .arg("1")
        .arg("-ar")
        .arg("16000")
        .arg("-f")
        .arg("s16le")
        .arg("-acodec")
        .arg("pcm_s16le")
        .arg("pipe:1")
        .kill_on_drop(true)
        .spawn()?;

    let mut stdin = child.stdin.take().expect("FFMPEG stdin to be present");


    tokio::spawn(async move{
        stdin.write_all(&audio).await.expect("Failed to write to stdin");
    });

    let output = child.wait_with_output().await?;

    if !output.status.success() {
        return Err(ConversionError::FFMPEG_OUTPUT {message: from_utf8(&output.stderr)?.to_string() });
    }

    return Ok(convert_to_siren(output.stdout));
}

pub fn convert_to_siren(mut wave_pcm16: Vec<u8>) -> Vec<u8> {
    let mut buffer: Vec<u8> = vec![0; SIREN_FRAME_SIZE / 16];
    let encoder: *mut crate::stSirenEncoder = unsafe { Siren7_NewEncoder(16000) };

    let mut data_part: Vec<u8> = wave_pcm16.chunks_mut(SIREN_FRAME_SIZE).flat_map(|c| {
        buffer.fill(0);
        unsafe { Siren7_EncodeFrame(encoder, c.as_mut_ptr(), buffer.as_mut_ptr()) };
        buffer.to_owned()
    }).collect();

    let siren_wave_header: SirenWavHeader = unsafe { *encoder }.WavHeader;
    let mut siren_wave_header_as_bytes : Vec<u8> = unsafe { any_as_u8_slice(&siren_wave_header).to_vec() };
    siren_wave_header_as_bytes.append(&mut data_part);
    unsafe { Siren7_CloseEncoder(encoder); }

    return siren_wave_header_as_bytes;
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::core::mem::size_of::<T>(),
    )
}


mod tests {
    use std::thread;
    use std::io::Write;
    use std::process::{Command, Stdio};

    use crate::{Siren7_CloseEncoder, Siren7_EncodeFrame, Siren7_NewEncoder};
    use crate::models::uuid::UUID;

    lazy_static_include_bytes! {
            DOOR_OGG => "assets/sound/door.ogg",
            DOOR_WAV => "assets/sound/door.wav",

            MSN_EXPORTED_WAV => "assets/sound/testmsnaudio.wav"
        }

    #[test]
    fn siren_encoder_test() {

    }

    #[test]
    fn siren_decoder_test() {

        let encoder = unsafe { Siren7_NewEncoder(16000) };

        let mut wav: Vec<u8> =  MSN_EXPORTED_WAV[40..MSN_EXPORTED_WAV.len()].to_vec();
        let mut buffer: Vec<u8> = Vec::with_capacity(2048);

        let result: Vec<u8> = wav.chunks_mut(2048).flat_map(|c| {
            buffer.fill(0);
            let result = unsafe { Siren7_EncodeFrame(encoder, c.as_mut_ptr(), buffer.as_mut_ptr()) };
            if result >= 0 {
                return buffer[0..result as usize].to_vec();
            } else {
                panic!("AAAAAH {}", result);
            }
        } ).collect();

        let mut file = std::fs::File::create(format!("C:\\temp\\out_test_{}.raw", UUID::new())).unwrap();
        file.write_all(&result);

        unsafe { Siren7_CloseEncoder(encoder); }
    }
    #[test]
fn ffmpeg() {



   // let mut buffer = AVMem::new(2048);
  //  let avio_context = AVIOContextCustom::alloc_context(buffer, false, media, None, None, None);
   // let mut avio_context_contained = AVIOContextContainer::Custom(avio_context);
 //   let mut input_format_context = AVFormatContextInput::from_io_context(avio_context_contained)?;

        let mut child = Command::new("ffmpeg")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .arg("-f")
            .arg("ogg")
            .arg("-i")
            .arg("pipe:0")
            .arg("-ac")
            .arg("1")
            .arg("-ar")
            .arg("16000")
            .arg("-f")
            .arg("wav")
            .arg("pipe:1")
            .spawn().unwrap();

        let mut stdin = child.stdin.take().expect("Failed to open stdin");

        thread::spawn(move || {
            stdin.write_all(&DOOR_OGG).expect("Failed to write to stdin");
        });

        let output = child.wait_with_output().expect("Failed to read stdout");
        let my_bytes = output.stdout;

        println!("{:?}", my_bytes);
        let test = 0;
}

}