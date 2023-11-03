use std::io::{Write, Cursor};
use std::process::Stdio;
use std::str::from_utf8_unchecked;
use byteorder::{LittleEndian, ByteOrder};
use log::debug;
use serde::de;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::models::uuid::UUID;
use crate::{Siren7_NewEncoder, SirenWavHeader, Siren7_EncodeFrame, Siren7_CloseEncoder};

lazy_static_include_bytes! {
            MSNWAV => "assets/sound/aeoncl_02_11_23@12_45_58.wav"
        }


static SIREN_FRAME_SIZE : usize = 640usize;

pub async fn convert_audio_message(audio: Vec<u8>) -> Vec<u8> {

    let log_file_uuid = UUID::new();


    let mut ogg = std::fs::File::create(format!("C:\\temp\\ogg{}.ogg", &log_file_uuid)).unwrap();
    ogg.write_all(&audio);

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
        .spawn().unwrap();

    let mut stdin = child.stdin.take().expect("Failed to open stdin");


    tokio::spawn(async move{
        stdin.write_all(&audio).await.expect("Failed to write to stdin");
    });

    let output = child.wait_with_output().await.expect("Failed to read stdout");
   return convert_to_siren(output.stdout, &log_file_uuid);
}

pub fn remove_wave_header(wave_pcm16: &mut Vec<u8>) {

    let wave_as_str = unsafe { String::from_utf8_unchecked(wave_pcm16.clone()) };
    let data_index = wave_as_str.find("data").expect("data field to be present in PCM 16 WAVE") + 7;
    debug!("Wave data starts at index: {}", data_index);
   let wave_header = wave_pcm16.drain(0..data_index);

   debug!("DEBUG WAVE HEADER REMOVED: {:?}", wave_header.as_slice())
   //assert_eq!(&wave_header.as_slice()[36..40], "data".as_bytes())
}

pub fn convert_to_siren(mut wave_pcm16: Vec<u8>, log_file_uuid: &UUID) -> Vec<u8> {


    //let mut wave_file = std::fs::File::create(format!("C:\\temp\\wave_{}.wav", log_file_uuid)).unwrap();
    // wave_file.write_all(&wave_pcm16);

    // remove_wave_header(&mut wave_pcm16);
    
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

    
    let mut siren_file = std::fs::File::create(format!("C:\\temp\\siren_{}.wav", log_file_uuid)).unwrap();
    siren_file.write_all(&siren_wave_header_as_bytes);


    return siren_wave_header_as_bytes;
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts(
        (p as *const T) as *const u8,
        ::core::mem::size_of::<T>(),
    )
}


mod tests {
    use std::io::{Cursor, Write};
    use std::process::{Command, Stdio};
    use std::{mem, thread};
    use byteorder::{LittleEndian, ByteOrder};
    use ffmpeg_cli::FfmpegBuilder;
    use wav::BitDepth;
    use crate::utils::ffmpeg::any_as_u8_slice;
    use crate::{PCMWavHeader, RiffHeader, Siren7_CloseEncoder, Siren7_EncodeFrame, Siren7_NewEncoder, SirenEncoder, SirenWavHeader};
    use crate::models::uuid::UUID;

    lazy_static_include_bytes! {
            DOOR_OGG => "assets/sound/door.ogg",
            DOOR_WAV => "assets/sound/door.wav",

            MSN_EXPORTED_WAV => "assets/sound/testmsnaudio.wav"
        }

    #[test]
    fn siren_encoder_test() {

        let frame_size = 640 as usize;

        let encoder: *mut crate::stSirenEncoder = unsafe { Siren7_NewEncoder(16000) };


        let mut raw_wav: Vec<u8> =  MSN_EXPORTED_WAV.to_vec();

        let mut file: Cursor<Vec<u8>> = Cursor::new(raw_wav);

        let (header, data) = wav::read(&mut file).expect("WAV to be valid PCM WAVE");

        let mut data_bytes : Vec<u8> = data.as_sixteen().unwrap().to_owned().into_iter().flat_map(|x|{
            let mut buffer = [0; 2];
            LittleEndian::write_i16(&mut buffer, x);
            buffer.to_owned()

        }).collect();

        let mut buffer: Vec<u8> = vec![0; frame_size / 16];

        let mut result: Vec<u8> = data_bytes.chunks_mut(frame_size).flat_map(|c| {
            buffer.fill(0);
            if c.len() < frame_size {
                let mut last_chunk = c.to_vec();
                last_chunk.resize(frame_size, 0);
                unsafe { Siren7_EncodeFrame(encoder, last_chunk.as_mut_ptr(), buffer.as_mut_ptr()) };
                buffer.to_owned()
            } else {
                unsafe { Siren7_EncodeFrame(encoder, c.as_mut_ptr(), buffer.as_mut_ptr()) };
                buffer.to_owned()
            }
        }).collect();


        let waveHeader: SirenWavHeader = unsafe { *encoder }.WavHeader;

        let mut header : Vec<u8> = unsafe { any_as_u8_slice(&waveHeader).to_vec() };

        header.append(&mut result);

        let mut file = std::fs::File::create(format!("C:\\temp\\out_test_{}.raw", UUID::new())).unwrap();
        file.write_all(&header);

        println!("{:?}", waveHeader);



   //     let result = result[0..(unsafe { *encoder }).WavHeader.DataSize as usize].to_vec();

        let test1 = 3;


        unsafe { Siren7_CloseEncoder(encoder); }
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