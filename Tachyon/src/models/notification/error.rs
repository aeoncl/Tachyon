use std::fmt::{Display, Formatter, Write};

use thiserror::Error;

use crate::models::tachyon_error::{MatrixError, PayloadError, TachyonError};

#[derive(Debug, Clone)]
pub enum MSNPErrorCode {

    AlreadyInMode = 218,
	AuthFail = 911,
	ChallengeResponseFailed = 540,
	CommandDisabled = 502,
	ContactListError = 403,
	ContactListUnavailable = 402,
	DuplicateSession = 207,
	GroupAlreadyExists = 228,
	GroupInvalid = 224,
	GroupNameTooLong = 229,
	GroupZeroUnremovable = 230,
	InternalServerError = 500,
	InvalidCircleMembership = 933,
	InvalidNetworkID = 204,
	InvalidParameter = 201,
	InvalidUser = 205,
	InvalidUser2 = 208,
	ListLimitReached = 210,
	NotAllowedWhileHDN = 913,
	NotExpected = 715,
	PrincipalNotInGroup = 225,
	PrincipalNotOnList = 216,
	PrincipalNotOnline = 217,
	PrincipalOnList = 215,
	XXLEmptyDomain = 240,
	XXLInvalidPayload = 241
}

impl Display for  MSNPErrorCode {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", self.to_owned() as u16))
	}
}


#[derive(Error, Debug)]
#[error("A MSNP error has occured - code: {} msg: {:?} kind: {:?}", .code, .msg, .kind)]
pub struct MSNPServerError {
	pub tr_id: Option<String>,
	pub code: MSNPErrorCode,
	pub msg: Option<String>,
	pub source: Option<TachyonError>,
	pub kind: MSNPServerErrorType
}

#[derive(Debug)]
pub enum MSNPServerErrorType {
	FatalError ,
	RecoverableError
}

impl From<bool> for MSNPServerErrorType {
	fn from(value: bool) -> Self {
		if value { Self::FatalError } else { Self::RecoverableError }
	}
}


impl MSNPServerError {
	pub fn new(fatal: bool, tr_id: Option<String>, code: MSNPErrorCode, source: TachyonError) -> Self {
			Self {
				tr_id,
				code,
				msg: None,
				source: Some(source),
				kind: fatal.into(),
			}

	}

	pub fn new_with_msg(fatal: bool, tr_id: Option<String>, code: MSNPErrorCode, message: String) -> Self {
		Self {
			tr_id,
			code,
			msg: Some(message),
			source: None,
			kind: fatal.into(),
		}
	}

	pub fn fatal_error_no_source(message: String) -> Self {
		Self::new_with_msg(true, None, MSNPErrorCode::InternalServerError, message)
	}

	pub fn fatal_error_no_source_with_trid(tr_id: String, message: String) -> Self {
		Self::new_with_msg(true, Some(tr_id), MSNPErrorCode::InternalServerError, message)
	}

	pub fn fatal_error(source: TachyonError) -> Self {
		Self::new(true, None, MSNPErrorCode::InternalServerError, source.into())
	}

	pub fn fatal_error_with_trid(tr_id: String, source: TachyonError) -> Self {
		Self::new(true, Some(tr_id), MSNPErrorCode::InternalServerError, source.into())

	}

	pub fn from_source_with_trid(tr_id: String, source: TachyonError) -> Self {
		Self::from_source_internal(Some(tr_id), source)
	}

	pub fn from_source(source: TachyonError) -> Self {
		Self::from_source_internal(None, source)
	}

	pub fn from_source_internal(tr_id: Option<String>, source: TachyonError) -> Self {
		match &source {
			TachyonError::AuthenticationError { sauce } => {
				Self::new(true, tr_id, MSNPErrorCode::AuthFail, source.into())
			},
			TachyonError::CommandSplitOutOfBounds { command } => {
				Self::new(true, tr_id, MSNPErrorCode::InvalidParameter, source.into())
			},
			TachyonError::P2PError(err) => {
				Self::new(false, tr_id, MSNPErrorCode::NotExpected, source.into())
			},
			TachyonError::PayloadError(err) => {
				match &err {
					PayloadError::EnumParsingError { payload, sauce } => {
						Self::new(true, tr_id, MSNPErrorCode::InvalidParameter, source.into())
					},
					PayloadError::BinaryPayloadParsingError {payload, sauce} => {
						Self::new(false, tr_id, MSNPErrorCode::InvalidParameter, source.into())
					},
					PayloadError::StringPayloadParsingError {payload, sauce} => {
						Self::new(false, tr_id, MSNPErrorCode::InvalidParameter, source.into())
					},
					PayloadError::MandatoryPartNotFound {name, payload} => {
						Self::new(true, tr_id, MSNPErrorCode::InvalidParameter, source.into())
					},
					PayloadError::PayloadBytesMissing => {
						Self::new(false, tr_id, MSNPErrorCode::InternalServerError, source.into())
					},
					PayloadError::PayloadDoesNotContainsSLP => {
						Self::new(false, tr_id, MSNPErrorCode::NotExpected, source.into())
					},
					PayloadError::AnyError(_err) => {
						Self::new(false, tr_id, MSNPErrorCode::NotExpected, source.into())
					},
					PayloadError::ParseIntError(_err) => {
						Self::new(true, tr_id, MSNPErrorCode::NotExpected, source.into())
					},
					PayloadError::PayloadNotHandled {..} => {
						Self::new(false, tr_id, MSNPErrorCode::NotExpected, source.into())
					}
				}
			},
			TachyonError::MatrixError(err) => {

				match &err {
					MatrixError::WebError(err) => {
						Self::new(true, tr_id, MSNPErrorCode::InternalServerError, source.into())
					}
					MatrixError::IdParseError(err) => {
						Self::new(true, tr_id, MSNPErrorCode::InternalServerError, source.into())
					}
				}
			},
			TachyonError::UUIDConversionError(_err) => {
				Self::new(true, tr_id, MSNPErrorCode::InvalidParameter, source.into())
			}
		}
	}

	fn handle_matrix_error(tr_id: Option<String>, error: &MatrixError, source: TachyonError) -> Self {
		match &error {
			MatrixError::WebError(err) => {
				Self::new(true, tr_id, MSNPErrorCode::InternalServerError, source.into())
			}
			MatrixError::IdParseError(err) => {
				Self::new(true, tr_id, MSNPErrorCode::InternalServerError, source.into())
			}
		}
	}

	fn handle_payload_error(tr_id: Option<String>, error: &PayloadError, source: TachyonError) -> Self{
		match &error {
			PayloadError::EnumParsingError { payload, sauce } => {
				Self::new(true, tr_id, MSNPErrorCode::InvalidParameter, source.into())
			},
			PayloadError::BinaryPayloadParsingError {payload, sauce} => {
				Self::new(false, tr_id, MSNPErrorCode::InvalidParameter, source.into())
			},
			PayloadError::StringPayloadParsingError {payload, sauce} => {
				Self::new(false, tr_id, MSNPErrorCode::InvalidParameter, source.into())
			},
			PayloadError::MandatoryPartNotFound {name, payload} => {
				Self::new(true, tr_id, MSNPErrorCode::InvalidParameter, source.into())
			},
			PayloadError::PayloadBytesMissing => {
				Self::new(false, tr_id, MSNPErrorCode::InternalServerError, source.into())
			},
			PayloadError::PayloadDoesNotContainsSLP => {
				Self::new(false, tr_id, MSNPErrorCode::NotExpected, source.into())
			},
			PayloadError::AnyError(_err) => {
				Self::new(false, tr_id, MSNPErrorCode::NotExpected, source.into())
			},
			PayloadError::ParseIntError(_err) => {
				Self::new(true, tr_id, MSNPErrorCode::NotExpected, source.into())
			},
			PayloadError::PayloadNotHandled {..} => {
				Self::new(false, tr_id, MSNPErrorCode::NotExpected, source.into())
			}
		}
	}



}