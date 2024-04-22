use axum::http::header::ToStrError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;
use msnp::soap::error::SoapMarshallError;
use crate::notification::client_store::ClientStoreError;
use crate::shared::error::MatrixConversionError;
use crate::web::soap::error::ABError;
use crate::web::soap::shared::build_soap_response;

//TODO
#[derive(Error, Debug)]

pub enum RSIError {
    
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
    #[error(transparent)]
    InternalServerError(#[from] anyhow::Error),
    #[error(transparent)]
    MatrixError(#[from] matrix_sdk::Error),
    #[error(transparent)]
    MatrixStoreError(#[from] matrix_sdk::StoreError),
    #[error("Unsupported Soap Action: {}", .0)]
    UnsupportedSoapAction(String)

}

impl IntoResponse for RSIError {
    fn into_response(self) -> Response {
        build_soap_response("".into(), StatusCode::INTERNAL_SERVER_ERROR)
    }
}