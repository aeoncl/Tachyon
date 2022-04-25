use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    static ref TEMPLATE : String = String::from("MIME-Version: 1.0\r\nContent-Type: ");
    static ref CHARSET : String = String::from("; charset=UTF-8");
}
 
pub struct MsgPayload {

    pub content_type: String,
    pub headers: HashMap<String, String>,
    pub body: String
}

impl MsgPayload {

    pub fn new(content_type: &str) -> Self {
        return MsgPayload{ content_type: content_type.to_string(), headers: HashMap::new(), body: String::new() };
    }

    pub fn add_header(&mut self, name: String, value: String){
        self.headers.insert(name, value);
    }

    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }

    pub fn serialize(&self) -> String {
        let mut headers = String::new();
        for (key, value) in &self.headers {
            headers.push_str(format!("{}: {}\r\n", &key, &value).as_str());
        }
        return format!("{template}{content_type}{charset}\r\n{headers}\r\n{body}", template=TEMPLATE.as_str(), content_type=&self.content_type, charset=CHARSET.as_str(), headers=headers, body=&self.body);
    }
}

pub mod factories {
    use chrono::Local;

    use crate::models::uuid::{PUID};

    use super::MsgPayload;

    pub struct MsgPayloadFactory;

    impl MsgPayloadFactory {
        pub fn get_msmsgs_profile(puid: PUID, msn_addr: String, matrix_token: String) -> MsgPayload {
            let mut out = MsgPayload::new("text/x-msmsgsprofile");
            let now = Local::now().timestamp_millis();
            out.add_header(String::from("LoginTime"), now.to_string());
            out.add_header(String::from("EmailEnabled"), String::from("1"));
            out.add_header(String::from("MemberIdHigh"), puid.get_most_significant_bytes().to_string());
            out.add_header(String::from("MemberIdLow"), puid.get_least_significant_bytes().to_string());
            out.add_header(String::from("lang_preference"), String::from("1033"));
            out.add_header(String::from("preferredEmail"), msn_addr);
            out.add_header(String::from("country"), String::new());
            out.add_header(String::from("PostalCode"), String::new());
            out.add_header(String::from("Gender"), String::new());
            out.add_header(String::from("Kid"), String::from("0"));
            out.add_header(String::from("Age"), String::new());
            out.add_header(String::from("BDayPre"), String::new());
            out.add_header(String::from("Birthday"), String::new());
            out.add_header(String::from("Wallet"), String::new());
            out.add_header(String::from("Flags"), String::from("1610613827"));
            out.add_header(String::from("sid"), String::from("507"));
            out.add_header(String::from("MSPAuth"), matrix_token);
            out.add_header(String::from("ClientIP"), String::new());
            out.add_header(String::from("ClientPort"), String::from("0"));
            out.add_header(String::from("ABCHMigrated"), String::from("1"));
            out.add_header(String::from("MPOPEnabled"), String::from("1"));

            return out;
        }

        pub fn get_initial_mail_data_notification() -> MsgPayload {
            let mut out = MsgPayload::new("text/x-msmsgsinitialmdatanotification");
            out.set_body(String::from("Mail-Data: <MD><E><I>0</I><IU>0</IU><O>0</O><OU>0</OU></E><Q><QTM>409600</QTM><QNM>204800</QNM></Q></MD>\r\nInbox-Unread: 1\r\nFolders-Unread: 0\r\nInbox-URL: /cgi-bin/HoTMaiL\r\nFolders-URL: /cgi-bin/folders\r\nPost-URL: http://127.0.0.1:8080/email\r\n"));
            return out;
        }
    }

}

#[cfg(test)]
mod tests {
    use crate::models::msg_payload::MsgPayload;


    #[test]
    fn test() {
        let mut payload = MsgPayload::new("content-type");
        payload.add_header(String::from("headerName"),String::from("headerValue"));
        let serialized = payload.serialize();
        assert_eq!(serialized,String::from("MIME-Version: 1.0\r\nContent-Type: content-type; charset=UTF-8\r\nheaderName:headerValue\r\n\r\n")); 
    }
}