use crate::models::{slp_payload::P2PTransportPacket, msn_user::MSNUser};


#[derive(Clone, Debug)]
pub struct PendingPacket {
    pub packet: P2PTransportPacket,
    pub awaited_seq_num: u32,
    pub sender: MSNUser,
    pub receiver: MSNUser

}

impl PendingPacket {
    
    pub fn new(packet: P2PTransportPacket, sender: MSNUser, receiver: MSNUser) -> Self {
        return PendingPacket{ packet, sender, receiver, awaited_seq_num: 0 };
    }
    
}

