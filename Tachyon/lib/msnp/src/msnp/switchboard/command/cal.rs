// Invite someone to join the SB
// >>> CAL 58 aeontest@shl.local
// <<< CAL 58 RINGING 4324234

pub struct CalClient {
    tr_id: u128,
    email_addr: String
}

pub struct CalServer {
    tr_id: u128,

    session_id: u64
}