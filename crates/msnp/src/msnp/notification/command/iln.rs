use crate::msnp::error::CommandError;
use crate::msnp::notification::command::uum::{UumClient, UumPayload};
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::models::capabilities::ClientCapabilities;
use crate::shared::models::email_address::EmailAddress;
use crate::shared::models::msn_object::MsnObject;
use crate::shared::models::network_id::NetworkId;
use crate::shared::models::presence_status::PresenceStatus;
use crate::shared::traits::MSNPCommand;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::shared::models::capabilities::ClientCapabilities;
    use crate::shared::models::email_address::EmailAddress;
    use crate::shared::models::msn_object::{FriendlyName, MSNObjectFactory};
    use crate::shared::models::network_id::NetworkId;
    use crate::shared::models::presence_status::PresenceStatus;
    use crate::shared::traits::MSNPCommand;

    use super::IlnServer;

    #[test]
    pub fn test_iln_ser_msn_obj() {

        let iln = IlnServer {
            tr_id: 1,
            presence_status: PresenceStatus::BSY,
            network_id: NetworkId::WindowsLive,
            email_address: EmailAddress::from_str("test@shlasouf.local").unwrap(),
            display_name: "Testo".to_string(),
            client_capabilities: ClientCapabilities::new(0,0),
            avatar: Some(MSNObjectFactory::get_contact_display_picture(&Vec::new(), "test@shlasouf.local".into(), "blabla.tmp".into(), FriendlyName::new("blabla.jpg"))),
            badge_url: None,
        };


        let bytes = iln.into_bytes();

        let iln_deser = String::from_utf8(bytes).unwrap();

        assert_eq!("ILN 1 BSY 1:test@shlasouf.local Testo 0:0 <msnobj Creator=\"test@shlasouf.local\" Type=\"3\" SHA1D=\"2jmj7l5rSw0yVb/vlWAYkK/YBwk=\" Size=\"0\" Location=\"blabla.tmp\" Friendly=\"YgBsAGEAYgBsAGEALgBqAHAAZwAAAA==\" contenttype=\"D\" />\r\n", &iln_deser);
    }

    #[test]
    pub fn test_iln_ser_no_msn_obj() {

        let iln = IlnServer {
            tr_id: 1,
            presence_status: PresenceStatus::BSY,
            network_id: NetworkId::WindowsLive,
            email_address: EmailAddress::from_str("test@shlasouf.local").unwrap(),
            display_name: "Testo".to_string(),
            client_capabilities: ClientCapabilities::new(0,0),
            avatar: None,
            badge_url: None,
        };


        let bytes = iln.into_bytes();

        let iln_deser = String::from_utf8(bytes).unwrap();

        assert_eq!("ILN 1 BSY 1:test@shlasouf.local Testo 0:0 0\r\n", &iln_deser);
    }

    #[test]
    pub fn test_iln_ser_no_msn_obj_badge() {

        let iln = IlnServer {
            tr_id: 1,
            presence_status: PresenceStatus::BSY,
            network_id: NetworkId::WindowsLive,
            email_address: EmailAddress::from_str("test@shlasouf.local").unwrap(),
            display_name: "Testo".to_string(),
            client_capabilities: ClientCapabilities::new(0,0),
            avatar: None,
            badge_url: Some("http://badge.jpg".into()),
        };


        let bytes = iln.into_bytes();

        let iln_deser = String::from_utf8(bytes).unwrap();

        assert_eq!("ILN 1 BSY 1:test@shlasouf.local Testo 0:0 0 http://badge.jpg\r\n", &iln_deser);
    }


}


pub struct IlnServer {
    pub tr_id: u128,
    pub presence_status: PresenceStatus,
    pub network_id: NetworkId,
    pub email_address: EmailAddress,
    pub display_name: String,
    pub client_capabilities: ClientCapabilities,
    pub avatar: Option<MsnObject>,
    pub badge_url: Option<String>
}

impl MSNPCommand for IlnServer {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> where Self: Sized {
        todo!()
    }

    fn into_bytes(self) -> Vec<u8> {
        let mut out = format!("ILN {tr_id} {presence_status} {network_id}:{email_addr} {display_name} {capab} {avatar}",
                tr_id = self.tr_id,
                presence_status = self.presence_status,
                network_id = self.network_id as u32,
                email_addr = self.email_address,
                display_name = self.display_name,
                capab = self.client_capabilities,
                avatar = self.avatar.map(|a| a.to_string()).unwrap_or("0".into())
        );


        match self.badge_url {
            None => {
                out.push_str("\r\n");
            }
            Some(badge_url) => {
                out.push_str(&format!(" {}\r\n", badge_url));
            }
        }

        out.into_bytes()
    }
}
