use thiserror::Error;

#[derive(Error, Debug)]
pub enum SoapError {
    #[error("An error has occured while deserializing XML: {}", .message)]
    DeserializationError { message: String }
}