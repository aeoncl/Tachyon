use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::traits::{IntoBytes, TryFromRawCommand};
use std::fmt::{Display, Formatter};
use num_derive::{FromPrimitive, ToPrimitive};
use strum_macros::Display;

pub struct ErrCommand {
    pub tr_id: u128,
    pub msnp_error: MsnpError
}

impl ErrCommand {
    pub fn new(tr_id: u128, error_code: MsnpError) -> Self {
        Self {
            tr_id,
            msnp_error: error_code,
        }
    }
}

impl Display for ErrCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{error_code} {tr_id}\r\n", error_code = self.msnp_error.clone() as u16, tr_id = self.tr_id)

    }
}

impl IntoBytes for ErrCommand {
    fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}
impl TryFromRawCommand for ErrCommand {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {
        todo!()
    }

}
#[repr(u16)]
#[derive(Display, Clone, FromPrimitive, ToPrimitive)]
pub enum MsnpError {
    InvalidSyntax = 200,
    InvalidParameter = 201,
    InvalidPrincipal = 205,
    InvalidPrincipalMsnp10 = 208,
    DomainNameMissing = 206,
    AlreadyLoggedIn = 207,
    NicknameChangeIllegal = 209,
    PrincipalListFull = 210,
    InvalidRenameRequest = 213,
    PrincipalAlreadyOnList = 215,
    PrincipalNotOnList = 216,
    PrincipalNotOnline = 217,
    AlreadyInMode = 218,
    PrincipalIsInTheOppositeList = 219,
    TooManyGroups = 223,
    InvalidGroup = 224,
    PrincipalNotInGroup = 225,
    GroupNotEmpty = 227,
    GroupWithSameNameAlreadyExists = 228,
    GroupNameTooLong = 229,
    CannotRemoveGroupZero = 230,
    InvalidGroup2 = 231,
    EmptyDomain = 240,
    SwitchboardFailed = 280,
    TransferToSwitchboardFailed = 281,
    P2PError = 282,
    RequiredFieldMissing = 300,
    NotLoggedIn = 302,
    ErrorAccessingContactList = 402,
    ErrorAccessingContactList2 = 403,
    InvalidAccountPermission = 420,
    InternalServerError = 500,
    DatabaseServerError = 501,
    CommandDisabled = 502,
    FileOperationFailed = 510,
    Banned = 511,
    MemoryAllocationFailed = 520,
    ChallengeResponseFailed = 540,
    ServerIsBusy = 600,
    ServerIsUnavaillable = 601,
    PeerNameserverIsDown = 602,
    DatabaseConnectionFailed = 603,
    ServerIsGoingDown = 604,
    ServerUnavaillable = 605,
    CouldNotCreateConnection = 700,
    BadCVRParameter = 710,
    WriteIsBlocking = 711,
    SessionIsOverloaded = 712,
    CallingTooRapidly = 713,
    TooManySessions = 714,
    NotExpected = 715,
    BadFriendFile = 717,
    NotExpected2 = 731,
    ChangingTooRapidly = 800,
    ServerTooBusy = 910,
    ServerIsBusy2 = 911,
    ServerTooBusy2 = 912,
    NotAllowedWhenHidden = 913,
    ServerUnavaillable2 = 914,
    ServerUnavaillable3 = 915,
    ServerUnavaillable4 = 916,
    AuthenticationFailed = 917,
    ServerTooBusy3 = 918,
    ServerTooBusy4 = 919,
    NotAcceptingNewPrincipals = 920,
    ServerTooBusy5 = 921,
    ServerTooBusy6 = 922,
    KidsPassportWithoutParentalConsent = 923,
    PassportAccountNotYetVerified = 924,
    BadTicket = 928,
    AccountNotOnThisServer = 931
}