use log::debug;
use msnp::p2p::v2::p2p_transport_packet::P2PTransportPacket;
use crate::tachyon::client::tachyon_client::TachyonClient;

pub async fn handle_p2p(tachyon_client: TachyonClient, p2p_packet: P2PTransportPacket) {


    if p2p_packet.is_slp_msg() {
        let payload = p2p_packet.get_payload().unwrap();
        let slp = payload.get_payload_as_slp().unwrap();

    } else {

    }

    if let Some(data_packet) = p2p_packet.get_payload() {



        debug!("Get session: {}", data_packet.session_id);

        let session = tachyon_client.get_session(data_packet.session_id).unwrap();

        if let Ok(slp) = data_packet.get_payload_as_slp() {

            println!("{}", slp);


        }


    }




}

