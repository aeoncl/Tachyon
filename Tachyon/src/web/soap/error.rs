use anyhow::anyhow;
use axum::http::header::ToStrError;
use msnp::soap::error::SoapMarshallError;
use thiserror::Error;
use crate::notification::client_store::ClientStoreError;
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
pub enum ABError {
    #[error("Couldn't authenticate client")]
    AuthenticationFailed {source: anyhow::Error},
    #[error(transparent)]
    ClientStoreError(#[from] ClientStoreError),
    #[error("Mandatory header: {} was missing from request.", .0)]
    MissingHeader(String),
    #[error(transparent)]
    HeaderParseError(#[from] ToStrError),
    #[error(transparent)]
    SoapMarshallError(#[from] SoapMarshallError),
    #[error(transparent)]
    MatrixConversionError(#[from] MatrixConversionError),
    #[error("An internal server error has occured")]
    InternalServerError{source: anyhow::Error},
    #[error(transparent)]
    MatrixError(#[from] matrix_sdk::Error),
    #[error(transparent)]
    MatrixStoreError(#[from] matrix_sdk::StoreError),
    #[error("Unsupported Soap Action: {}", .0)]
    UnsupportedSoapAction(String)
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
