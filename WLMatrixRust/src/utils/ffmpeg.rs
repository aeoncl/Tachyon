use std::io::Write;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub async fn convert_audio_message(audio: Vec<u8>) -> Vec<u8> {

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
        .arg("wav")
        .arg("pipe:1")
        .kill_on_drop(true)
        .spawn().unwrap();

    let mut stdin = child.stdin.take().expect("Failed to open stdin");


    tokio::spawn(async move{
        stdin.write_all(&audio).await.expect("Failed to write to stdin");
    });

    let output = child.wait_with_output().await.expect("Failed to read stdout");

    return output.stdout;
}


mod tests {
    use std::io::Write;
    use std::process::{Command, Stdio};
    use std::thread;
    use ffmpeg_cli::FfmpegBuilder;

    lazy_static_include_bytes! {
            DOOR_OGG => "assets/sound/door.ogg"
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