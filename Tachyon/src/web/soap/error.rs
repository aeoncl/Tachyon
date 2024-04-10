use anyhow::anyhow;
use msnp::soap::error::SoapMarshallError;
use thiserror::Error;
use crate::shared::error::MatrixConversionError;

#[derive(Error, Debug)]
pub enum SoapError {
    #[error("Couldn't authenticate client")]
    AuthenticationFailed {source: anyhow::Error},
    #[error(transparent)]
    SoapMarshallError(#[from] SoapMarshallError),
    #[error(transparent)]
    MatrixConversionError(#[from] MatrixConversionError),
    #[error("An internal server error has occured")]
    InternalServerError{source: anyhow::Error}
}

#[derive(Error, Debug)]
pub enum RST2Error {
    #[error("Couldn't authenticate client")]
    AuthenticationFailed {source: anyhow::Error},
    #[error(transparent)]
    SoapMarshallError(#[from] SoapMarshallError),
    #[error(transparent)]
    MatrixConversionError(#[from] MatrixConversionError),
    #[error("An internal server error has occured")]
    InternalServerError{source: anyhow::Error}
}
