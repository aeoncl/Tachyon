use crate::msnp::switchboard::models::b64_string::Base64String;
use crate::shared::command::ok::OkCommand;
use crate::shared::models::endpoint_id::EndpointId;

// Answers an XFR command from the Notification Sever, joining a Switchboard
// >>> ANS 3 aeontest@shl.local;{F52973B6-C926-4BAD-9BA8-7C1E840E4AB0} base64token 4060759068338340280
// <<< ANS 3 OK
pub struct AnsClient {
    tr_id: u128,
    endpoint_id: EndpointId,
    token: Base64String,
    session_id: u64
}

impl AnsClient {

    pub fn get_ok_response(&self) -> OkCommand {
        OkCommand{ operand: "ANS".to_string(), tr_id: self.tr_id}
    }

}
