use crate::shared::models::endpoint_id::EndpointId;

// Initiate a new SB.
// >>> USR 55 aeontest@shl.local;{F52973B6-C926-4BAD-9BA8-7C1E840E4AB0} token
// <<< USR 55 aeontest@shl.local aeontest@shl.local OK
pub struct UsrClient {

    tr_id: u128,
    endpoint_id: EndpointId,
    token: String

}

pub struct UsrServerOk {
    tr_id: u128,
    email_addr: String,
    display_name: String,
}