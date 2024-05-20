use crate::msnp::error::CommandError;
use crate::msnp::notification::command::uum::{UumClient, UumPayload};
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::models::capabilities::ClientCapabilities;
use crate::shared::models::email_address::EmailAddress;
use crate::shared::models::msn_object::MsnObject;
use crate::shared::models::network_id::NetworkId;
use crate::shared::models::network_id_email::NetworkIdEmail;
use crate::shared::models::presence_status::PresenceStatus;
use crate::shared::traits::MSNPCommand;

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::shared::models::capabilities::ClientCapabilities;
    use crate::shared::models::email_address::EmailAddress;
    use crate::shared::models::msn_object::{FriendlyName, MSNObjectFactory};
    use crate::shared::models::network_id::NetworkId;
    use crate::shared::models::network_id_email::NetworkIdEmail;
    use crate::shared::models::presence_status::PresenceStatus;
    use crate::shared::traits::MSNPCommand;

    use super::{NlnServer};

    #[test]
    pub fn test_nln_via_ser_msn_obj() {

        let iln = NlnServer {
            presence_status: PresenceStatus::BSY,
            target_user: NetworkIdEmail::new(NetworkId::WindowsLive, EmailAddress::from_str("test@shlasouf.local").unwrap()),
            via: Some(NetworkIdEmail::new(NetworkId::Circle, EmailAddress::from_str("test@live.fr").unwrap())),
            display_name: "Testo".to_string(),
            client_capabilities: ClientCapabilities::new(0,0),
            avatar: Some(MSNObjectFactory::get_contact_display_picture(&Vec::new(), "test@shlasouf.local".into(), "blabla.tmp".into(), FriendlyName::new("blabla.jpg"))),
            badge_url: None,
        };

        let bytes = iln.into_bytes();

        let nln_deser = String::from_utf8(bytes).unwrap();

        assert_eq!("NLN BSY 1:test@shlasouf.local;via=9:test@live.fr Testo 0:0 <msnobj Creator=\"test@shlasouf.local\" Type=\"3\" SHA1D=\"2jmj7l5rSw0yVb/vlWAYkK/YBwk=\" Size=\"0\" Location=\"blabla.tmp\" Friendly=\"YgBsAGEAYgBsAGEALgBqAHAAZwAAAA==\" contenttype=\"D\" />\r\n", &nln_deser);
    }

    #[test]
    pub fn test_nln_ser_msn_obj() {

        let nln = NlnServer {
            presence_status: PresenceStatus::BSY,
            target_user: NetworkIdEmail::new(NetworkId::WindowsLive, EmailAddress::from_str("test@shlasouf.local").unwrap()),
            via: None,
            display_name: "Testo".to_string(),
            client_capabilities: ClientCapabilities::new(0,0),
            avatar: Some(MSNObjectFactory::get_contact_display_picture(&Vec::new(), "test@shlasouf.local".into(), "blabla.tmp".into(), FriendlyName::new("blabla.jpg"))),
            badge_url: None,
        };

        let bytes = nln.into_bytes();

        let nln_deser = String::from_utf8(bytes).unwrap();

        assert_eq!("NLN BSY 1:test@shlasouf.local Testo 0:0 <msnobj Creator=\"test@shlasouf.local\" Type=\"3\" SHA1D=\"2jmj7l5rSw0yVb/vlWAYkK/YBwk=\" Size=\"0\" Location=\"blabla.tmp\" Friendly=\"YgBsAGEAYgBsAGEALgBqAHAAZwAAAA==\" contenttype=\"D\" />\r\n", &nln_deser);
    }

    #[test]
    pub fn test_nln_ser_no_msn_obj() {

        let nln = NlnServer {
            presence_status: PresenceStatus::BSY,
            target_user: NetworkIdEmail::new(NetworkId::WindowsLive, EmailAddress::from_str("test@shlasouf.local").unwrap()),
            via: None,
            display_name: "Testo".to_string(),
            client_capabilities: ClientCapabilities::new(0,0),
            avatar: None,
            badge_url: None,
        };


        let bytes = nln.into_bytes();

        let nln_deser = String::from_utf8(bytes).unwrap();

        assert_eq!("NLN BSY 1:test@shlasouf.local Testo 0:0 0\r\n", &nln_deser);
    }

    #[test]
    pub fn test_nln_ser_no_msn_obj_badge() {

        let nln = NlnServer {
            presence_status: PresenceStatus::BSY,
            target_user: NetworkIdEmail::new(NetworkId::WindowsLive, EmailAddress::from_str("test@shlasouf.local").unwrap()),
            via: None,
            display_name: "Testo".to_string(),
            client_capabilities: ClientCapabilities::new(0,0),
            avatar: None,
            badge_url: Some("http://badge.jpg".into()),
        };


        let bytes = nln.into_bytes();

        let nln_deser = String::from_utf8(bytes).unwrap();

        assert_eq!("NLN BSY 1:test@shlasouf.local Testo 0:0 0 http://badge.jpg\r\n", &nln_deser);
    }


}


pub struct NlnServer {
    pub presence_status: PresenceStatus,
    pub target_user: NetworkIdEmail,
    pub via: Option<NetworkIdEmail>,
    pub display_name: String,
    pub client_capabilities: ClientCapabilities,
    pub avatar: Option<MsnObject>,
    pub badge_url: Option<String>,
}

impl MSNPCommand for NlnServer {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> where Self: Sized {
        todo!()
    }

    fn into_bytes(self) -> Vec<u8> {

        let mut target_user = match self.via {
            None => {
                self.target_user.to_string()
            }
            Some(via) => {
                format!("{};via={}", self.target_user.to_string(), via.to_string())
            }
        };



        let mut out = format!("NLN {presence_status} {target_user} {display_name} {capab} {avatar}",
                              presence_status = self.presence_status,
                              target_user = target_user,
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
