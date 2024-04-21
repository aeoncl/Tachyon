use anyhow::anyhow;
use axum::http::header::ToStrError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use log::error;
use msnp::soap::error::SoapMarshallError;
use thiserror::Error;
use msnp::soap::abch::msnab_faults::SoapFaultResponseEnvelope;
use msnp::soap::passport::rst2::response::factory::RST2ResponseFactory;
use msnp::soap::traits::xml::ToXml;
use crate::notification::client_store::ClientStoreError;
use crate::shared::error::MatrixConversionError;
use crate::web::soap::error::ABError::InternalServerError;
use crate::web::soap::shared;
use crate::web::soap::shared::build_soap_response;


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
    #[error(transparent)]
    InternalServerError(#[from] anyhow::Error),
    #[error(transparent)]
    MatrixError(#[from] matrix_sdk::Error),
    #[error(transparent)]
    MatrixStoreError(#[from] matrix_sdk::StoreError),
    #[error("Unsupported Soap Action: {}", .0)]
    UnsupportedSoapAction(String)
}


const MARSHALL_ERROR : &str = r#"<?xml version="1.0" encoding="utf-8"?>
<soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
    <soap:Body>
        <soap:Fault>
            <faultcode>soap:Client</faultcode>
            <faultstring>Marshall Error</faultstring>
        </soap:Fault>
    </soap:Body>
</soap:Envelope>
"#;

impl IntoResponse for ABError {
    fn into_response(self) -> Response {
        error!("SOAP|ABCH: {:?}", &self);

        let body = match self {
            ABError::AuthenticationFailed { .. } => {
                SoapFaultResponseEnvelope::new_generic("Authentication failed".into())
            }
            ABError::ClientStoreError(_) => {
                SoapFaultResponseEnvelope::new_generic("Error with the client store".into())
            }
            ABError::MissingHeader(header) => {
                SoapFaultResponseEnvelope::new_generic(format!("Missing header in request: {}", header))
            }
            ABError::HeaderParseError(_) => {
                SoapFaultResponseEnvelope::new_generic("Could not parse header".into())

            }
            ABError::SoapMarshallError(_) => {
                SoapFaultResponseEnvelope::new_generic("Bad request".into())
            }
            ABError::MatrixConversionError(_) => {
                SoapFaultResponseEnvelope::new_generic("Could not convert to a matrix identifier".into())
            }
            InternalServerError { .. } => {
                SoapFaultResponseEnvelope::new_generic("General Error".into())
            }
            ABError::MatrixError(_) => {
                SoapFaultResponseEnvelope::new_generic("Fatal Matrix Error".into())
            }
            ABError::MatrixStoreError(_) => {
                SoapFaultResponseEnvelope::new_generic("Matrix Store Error".into())
            }
            ABError::UnsupportedSoapAction(soap_action) => {
                SoapFaultResponseEnvelope::new_unknown_soap_action(soap_action)
            }
        };


        let body = match body.to_xml() {
            Ok(response_body) => {
                response_body
            }
            Err(err) => {
                error!("SOAP|ABCH: Couldn't marshall error: {:?}", err);
                MARSHALL_ERROR.to_string()
            }
        };


        build_soap_response(body, StatusCode::INTERNAL_SERVER_ERROR)
    }
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

impl IntoResponse for RST2Error {
    fn into_response(self) -> Response {
        error!("SOAP|RST2: {:?}", &self);
        match self {
            RST2Error::AuthenticationFailed { .. } => {
                shared::build_soap_response(RST2ResponseFactory::get_auth_error_response(), StatusCode::OK)
            },
            RST2Error::SoapMarshallError(_) => {
                shared::build_soap_response(RST2ResponseFactory::get_bad_request(), StatusCode::OK)
            }
            RST2Error::MatrixConversionError(_) => {
                shared::build_soap_response(RST2ResponseFactory::get_bad_request(), StatusCode::OK)
            }
            RST2Error::InternalServerError { .. } => {
                shared::build_soap_response(RST2ResponseFactory::get_bad_request(), StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}