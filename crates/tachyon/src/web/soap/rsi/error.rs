use axum::http::header::ToStrError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use log::error;
use thiserror::Error;
use msnp::soap::rsi::faults::SoapFaultResponseEnvelope;
use msnp::soap::error::SoapMarshallError;
use msnp::soap::traits::xml::ToXml;
use crate::notification::client_store::ClientStoreError;
use crate::shared::error::MatrixConversionError;
use crate::web::soap::error::ABError;
use crate::web::soap::shared::build_soap_response;

#[derive(Error, Debug)]

pub enum RSIError {
    
    #[error("Couldn't authenticate client")]
    AuthenticationFailed {source: anyhow::Error, service_url: String},
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
    UnsupportedSoapAction(String),
    #[error("Throttle Limit Exceed")]
    ThrottleError,
    #[error("System not available")]
    SystemNotAvailable

}

impl IntoResponse for RSIError {
    fn into_response(self) -> Response {
        error!("SOAP|RSI: {:?}", &self);

        let soap_resp_body = match self {
            RSIError::AuthenticationFailed { source, service_url} => {
                SoapFaultResponseEnvelope::new_authentication_failed(&service_url, None, None)
            }
            RSIError::MissingHeader(_) | RSIError::HeaderParseError(_) | RSIError::MatrixConversionError(_) => {
                SoapFaultResponseEnvelope::new_schema_validator_error("https://rsi.hotmail.com/rsi/rsi.asmx")

            }
            RSIError::UnsupportedSoapAction(soap_action) => {
                SoapFaultResponseEnvelope::new_unknown_soap_action(soap_action)
            }
            RSIError::SoapMarshallError(cause) => {
                match cause {
                    SoapMarshallError::DeserializationError { .. } => {
                        SoapFaultResponseEnvelope::new_schema_validator_error("https://rsi.hotmail.com/rsi/rsi.asmx")
                    }
                    SoapMarshallError::SerializationError { .. } => {
                        SoapFaultResponseEnvelope::new_generic("Failed to marshall response".into())
                    }
                }
            },
            RSIError::ThrottleError => {
                SoapFaultResponseEnvelope::new_send_throttle_limit_exceed()
            },
            RSIError::SystemNotAvailable => {
                SoapFaultResponseEnvelope::new_system_unavailable()
            },
            _ => {
                SoapFaultResponseEnvelope::new_generic("An error has occured".into())
            }
        };

        let body = match soap_resp_body.to_xml() {
            Ok(response_body) => {
                response_body
            }
            Err(err) => {
                error!("SOAP|RSI: Couldn't marshall error response: {:?}", err);
                crate::web::soap::error::MARSHALL_ERROR.to_string()
            }
        };

        build_soap_response(body, StatusCode::INTERNAL_SERVER_ERROR)
    }
}