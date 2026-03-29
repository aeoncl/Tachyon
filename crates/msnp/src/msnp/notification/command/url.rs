use crate::msnp::error::CommandError;
use crate::msnp::notification::command::usr::{AuthPolicy, OperationTypeServer, SsoPhaseServer, UsrClient, UsrServer};
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::models::email_address::EmailAddress;
use crate::shared::traits::{IntoBytes, TryFromRawCommand};
use std::fmt::{Display, Formatter};
use std::str::FromStr;


/*


   client: URL 14 INBOX\r\n
   server: URL 14 /cgi-bin/HoTMaiL https://login.live.com/ppsecure/md5auth.srf?lc=1033 0\r\n

 */

pub struct UrlClient {
    pub tr_id: u128,
    pub url_type: UrlType
}

#[derive(Clone, Eq, PartialEq, Debug)]

pub enum UrlType {
    Inbox,
    Folders,
    Compose,
    ComposeFor {
        email_addr: EmailAddress
    },
    Profile {
        locale_id: String
    },
    Person {
        locale_id: String
    },
    Chat {
        locale_id: String
    },
    Mobile,
    AddrBook,
    AdvSearch,
    IntSearch,
    UnknownYet(String)
}

impl TryFromRawCommand for UrlClient {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err>
    where
        Self: Sized
    {
        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let raw_url_type = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "url_type".into(), 2))?;
        let url_type = match raw_url_type.as_str() {
            "INBOX" => {
                UrlType::Inbox
            }
            "FOLDERS" => {
                UrlType::Folders
            }
            "COMPOSE" => {
                if let Some(email) =split.pop_front().map(|s| EmailAddress::from_str(&s)) {
                    let email = email?;
                    UrlType::ComposeFor { email_addr: email }
                } else {
                    UrlType::Compose
                }
            }
            "PROFILE" => {
                let locale_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "profile_locale_id".into(), 3))?;
                UrlType::Profile {
                    locale_id
                }
            }
            "PERSON" => {
                let locale_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "profile_locale_id".into(), 3))?;
                UrlType::Person {
                    locale_id
                }
            }
            "CHAT" => {
                let locale_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "profile_locale_id".into(), 3))?;
                UrlType::Chat {
                    locale_id
                }
            }
            //TODO support all types
            _ => {

                let remaining =split.drain(..).reduce(|acc, current| format!("{} {}", acc, current) );
                UrlType::UnknownYet(format!("{} {:?}", raw_url_type, remaining))
            }
        };

        Ok(Self {
            tr_id,
            url_type,
        })

    }
}

pub struct UrlServer {
    tr_id: u128,
    main_url: String,
    post_url: String,
    //2 for windows live
    url_type: u8
}


impl UrlServer {
    pub fn new(tr_id: u128, main_url: String, post_url: String, url_type: u8) -> Self {
        Self {
            tr_id,
            main_url,
            post_url,
            url_type
        }
    }
}

impl Display for UrlServer {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "URL {tr_id} {main_url} {post_url} {url_type}\r\n", tr_id = self.tr_id,  main_url = self.main_url, post_url = self.post_url, url_type = self.url_type)
    }
}

impl IntoBytes for UrlServer {
    fn into_bytes(self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}


#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::msnp::raw_command_parser::RawCommandParser;
    use crate::shared::models::email_address::EmailAddress;
    use crate::shared::traits::TryFromRawCommand;

    use super::{UrlClient, UrlServer, UrlType};

    #[test]
    fn client_inbox_deser() {
       let command_str = "URL 18 INBOX\r\n";
       let raw_command = RawCommandParser::new().parse_message(command_str.as_bytes()).unwrap().remove(0);

       let deser = UrlClient::try_from_raw(raw_command).unwrap();

        assert_eq!(deser.tr_id, 18);
        assert_eq!(deser.url_type, UrlType::Inbox);

    }

    #[test]
    fn client_compose_deser() {
        let command_str = "URL 18 COMPOSE\r\n";
        let raw_command = RawCommandParser::new().parse_message(command_str.as_bytes()).unwrap().remove(0);

        let deser = UrlClient::try_from_raw(raw_command).unwrap();

        assert_eq!(deser.tr_id, 18);
        assert_eq!(deser.url_type, UrlType::Compose);

    }

    #[test]
    fn client_compose_for_deser() {
        let command_str = "URL 18 COMPOSE xx-th3a-xx@hotmail.com\r\n";
        let raw_command = RawCommandParser::new().parse_message(command_str.as_bytes()).unwrap().remove(0);

        let deser = UrlClient::try_from_raw(raw_command).unwrap();

        assert_eq!(deser.tr_id, 18);
        assert_eq!(deser.url_type, UrlType::ComposeFor { email_addr: EmailAddress::from_str("xx-th3a-xx@hotmail.com").unwrap() });

    }

    #[test]
    fn server_url_ser() {
        let command = UrlServer {
            tr_id: 1,
            main_url: "/cgi-bin/HoTMaiL".to_string(),
            post_url: "https://login.live.com/ppsecure/md5auth.srf?lc=1033".to_string(),
            url_type: 0,
        };

       let ser = command.to_string();

        assert_eq!("URL 1 /cgi-bin/HoTMaiL https://login.live.com/ppsecure/md5auth.srf?lc=1033 0\r\n", &ser)

    }

}
