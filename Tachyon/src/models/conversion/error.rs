use thiserror::Error;

#[derive(Error, Debug)]

pub enum ConversionError {

    #[error("FFMPEG returned with error output - stdout: {}", .message)]
    FFMPEG_OUTPUT { message: String },

    #[error(transparent)]
    FFMPEG_IO(#[from] std::io::Error),

    #[error(transparent)]
    FFMPEG_UTF8(#[from] std::str::Utf8Error)


}