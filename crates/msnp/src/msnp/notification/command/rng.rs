use std::fmt::{Display, Formatter};
use crate::msnp::error::CommandError;
use crate::msnp::notification::models::ip_address::IpAddress;
use crate::msnp::raw_command_parser::RawCommand;
use crate::msnp::switchboard::models::auth_method::AuthenticationMethod;
use crate::msnp::switchboard::models::session_id::SessionId;
use crate::shared::models::email_address::EmailAddress;
use crate::shared::models::ticket_token::TicketToken;
use crate::shared::traits::MSNPCommand;

pub struct RngServer {
    session_id: SessionId,
    address: IpAddress,
    auth_type: AuthenticationMethod,
    ticket_token: TicketToken,
    inviter_passport: EmailAddress,
    inviter_name: String,
}

impl RngServer {
    pub fn new(session_id: SessionId, address: IpAddress, ticket_token: TicketToken, inviter_passport: EmailAddress, inviter_name: String) -> Self {
        Self {
            session_id,
            address,
            auth_type: AuthenticationMethod::default(),
            ticket_token,
            inviter_passport,
            inviter_name,
        }
    }
}

impl MSNPCommand for RngServer {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err>
    where
        Self: Sized
    {
        todo!()
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

impl Display for RngServer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RNG {sess_id} {address} {auth_type} {token} {inviter_passport} {inviter_name}\r\n",
               sess_id = self.session_id,
               address = self.address,
               auth_type = self.auth_type,
               token = self.ticket_token,
               inviter_passport = self.inviter_passport,
               inviter_name = self.inviter_name
        )
    }
}