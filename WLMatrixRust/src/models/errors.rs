use std::str::Utf8Error;

#[derive(Debug)]

pub enum Errors {

    PayloadDeserializeError

}

impl From<Utf8Error> for Errors {
    fn from(err: Utf8Error) -> Errors {
        Errors::PayloadDeserializeError
    }
}

pub enum MsnpErrorCode {

    InvalidParameter = 201,
	InvalidNetworkID = 204,
	InvalidUser = 205,
	DuplicateSession = 207,
	InvalidUser2 = 208,
	ListLimitReached = 210,
	PrincipalOnList = 215,
	PrincipalNotOnList = 216,
	PrincipalNotOnline = 217,
	AlreadyInMode = 218,
	GroupInvalid = 224,
	PrincipalNotInGroup = 225,
	GroupAlreadyExists = 228,
	GroupNameTooLong = 229,
	GroupZeroUnremovable = 230,
	XXLEmptyDomain = 240,
	XXLInvalidPayload = 241,
	ContactListUnavailable = 402,
	ContactListError = 403,
	InternalServerError = 500,
	CommandDisabled = 502,
	ChallengeResponseFailed = 540,
	NotExpected = 715,
	AuthFail = 911,
	NotAllowedWhileHDN = 913,
	InvalidCircleMembership = 933
}