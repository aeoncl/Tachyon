use byteorder::{ByteOrder, LittleEndian};

use crate::{p2p::v2::p2p_transport_packet::P2PTransportPacket, shared::models::msn_user::MsnUser};

#[derive(Clone, Debug)]
pub struct MessageEventContent {
    pub packets: Vec<P2PTransportPacket>,
    pub sender: MsnUser,
    pub receiver: MsnUser,
}

impl MessageEventContent {
    pub fn as_directs_p2p(&self) -> Vec<Vec<u8>> {

        let mut out2: Vec<Vec<u8>> = Vec::with_capacity(self.packets.len());

        for packet in &self.packets {
            let mut msg_bytes: Vec<u8> = packet.to_vec();

            let size = msg_bytes.len();
            let mut buffer: [u8; 4] = [0, 0, 0, 0];
            LittleEndian::write_u32(&mut buffer, size as u32);

            let mut out = buffer.to_vec();
            out.append(&mut msg_bytes);

            out2.push(out);
        }



        return out2;
    }
}
