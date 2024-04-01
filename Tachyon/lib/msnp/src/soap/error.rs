use thiserror::Error;

#[derive(Error, Debug)]
pub enum SoapMarshallError {
    #[error("An error has occured while deserializing XML: {}", .message)]
    DeserializationError { message: String },

    #[error("An error has occured while serializing XML: {}", .message)]
    SerializationError { message: String }
}