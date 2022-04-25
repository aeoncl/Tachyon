use std::{str::Utf8Error, string::FromUtf8Error};

use actix_web::{HttpResponse, ResponseError, body::BoxBody};
use base64::DecodeError;
use http::StatusCode;
use url::ParseError;

#[derive(Debug)]
pub struct WebError {
    pub message: Option<String>,
    pub status_code: StatusCode
}


impl WebError {
    pub fn message(&self) -> String {
        match &self.message {
            Some(c) => c.clone(),
            None => String::from(""),
        }
    }
}

impl ResponseError for WebError {

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(("Content-Type", "application/soap+xml"))
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        return self.status_code;
    }
}

impl std::fmt::Display for WebError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}



impl From<Utf8Error> for WebError {
    fn from(err: Utf8Error) -> WebError {
        WebError {
            message: Some(err.to_string()),
            status_code: StatusCode::BAD_REQUEST
        }
    }
}
impl From<String> for WebError {
    fn from(err: String) -> WebError {
        WebError {
            message: Some(err),
            status_code: StatusCode::BAD_REQUEST
        }
    }
}

impl From<StatusCode> for WebError {
    fn from(err: StatusCode) -> WebError {
        WebError {
            message: None,
            status_code: err
        }
    }
}



impl From<matrix_sdk::Error> for WebError {
    fn from(err: matrix_sdk::Error) -> WebError {
        WebError {
            message: None,
            status_code: StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl From<matrix_sdk::HttpError> for WebError {
    fn from(err: matrix_sdk::HttpError) -> WebError {
        WebError {
            message: None,
            status_code: StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl From<ParseError> for WebError {
    fn from(err: ParseError) -> WebError {
        WebError {
            message: None,
            status_code: StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl From<DecodeError> for WebError {
    fn from(err: DecodeError) -> WebError {
        WebError {
            message: None,
            status_code: StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

impl From<FromUtf8Error> for WebError {
    fn from(err: FromUtf8Error) -> WebError {
        WebError {
            message: None,
            status_code: StatusCode::INTERNAL_SERVER_ERROR
        }
    }  
}

impl From<Result<(), StatusCode>> for WebError {
    fn from(err: Result<(), StatusCode>) -> WebError {
        WebError {
            message: None,
            status_code: err.err().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}




/* 
impl From<validator::ValidationErrors> for WebError {
    fn from(err: validator::ValidationErrors) -> WebError {
        WebError {
            message: Some(err.to_string()),
            err_type: ErrorType::ValidationError,
        }
    }

}impl From<String> for WebError {
    fn from(err: String) -> WebError {
        WebError {
            message: Some(err),
            err_type: ErrorType::UserError,
        }
    }
}
*/