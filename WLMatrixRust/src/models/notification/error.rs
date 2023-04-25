use matrix_sdk::{HttpResult, HttpError};

use crate::models::errors::Errors;

#[derive(Debug, Clone)]

pub enum MsnpErrorCode {

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

#[derive(Debug, Clone)]
pub struct MsnpError{
    pub code: MsnpErrorCode,
    pub tr_id: String
}

impl MsnpError {
    pub fn new(code: MsnpErrorCode, tr_id: String) -> Self {
        return MsnpError {
            code,
            tr_id,
        }
    }

    pub fn internal_server_error(tr_id: &str) -> Self {
        MsnpError::new(MsnpErrorCode::InternalServerError, tr_id.to_string())
    }

    pub fn auth_fail(tr_id: &str) -> Self {
        MsnpError::new(MsnpErrorCode::AuthFail, tr_id.to_string())
    }
}

impl From<HttpError> for MsnpErrorCode {
	fn from(err: HttpError) -> MsnpErrorCode {
        match err {
            HttpError::AuthenticationRequired => {
                return MsnpErrorCode::AuthFail;
            },
            _=> {
                return MsnpErrorCode::InternalServerError;
            }
            }
        }
    }


	impl From<uuid::Error> for MsnpErrorCode {
		fn from(err: uuid::Error) -> MsnpErrorCode {
			MsnpErrorCode::InvalidParameter
		}
	}

	impl From<Errors> for MsnpErrorCode {
    fn from(value: Errors) -> Self {
		return MsnpErrorCode::InternalServerError;
    }
}	