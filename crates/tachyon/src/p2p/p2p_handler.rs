use log::info;
use matrix_sdk::media::{MediaFormat, MediaRequestParameters};
use msnp::msnp::error::PayloadError;
use msnp::p2p::v2::factories::P2PPayloadFactory;
use crate::p2p::client::transport::{Transport, UnwrappedP2PPacket};
use crate::tachyon::client::tachyon_client::TachyonClient;
use msnp::p2p::v2::p2p_transport_packet::P2PTransportPacket;
use msnp::p2p::v2::slp::raw_slp_payload::{RawSlpPayload, SlpPayloadFactory};
use msnp::p2p::v2::slp::SlpPayload;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::models::msn_user::MsnUser;
use crate::p2p::client::session::SessionType;

pub async fn handle_p2p_packet(transport: Transport, p2p_packet: P2PTransportPacket, tachyon_client: TachyonClient) {

    let sorted_packet = transport.unwrap_packet(p2p_packet).await.unwrap();

    if let Some(packet) = sorted_packet {

        match packet {
            UnwrappedP2PPacket::Slp(slp_payload, transport_op) => {
                //Handle SLP
                let content_type = slp_payload.get_content_type().unwrap();
                if content_type.as_str() == "application/x-msnmsgr-sessionreqbody" && slp_payload.is_200_ok() {
                    //Start transfering stuff
                    let session_id = slp_payload
                        .get_body_property(&String::from("SessionID"))
                        .ok_or(PayloadError::MandatoryPartNotFound { name: "SessionID".to_string(), payload: slp_payload.to_string() }).unwrap()
                        .parse::<u32>().unwrap();

                    let session = tachyon_client.get_session(session_id).unwrap();
                    session.accept().unwrap();
                    tokio::spawn(async move {

                        let session_type = session.session_type();
                        match session_type {
                            SessionType::ReceiveFile(content) => {
                                let file = tachyon_client.matrix_client().media().get_media_content(
                                    &MediaRequestParameters {
                                        source: content.media_source.clone(),
                                        format: MediaFormat::File,
                                    },
                                    false
                                ).await;

                                match file {
                                    Ok(file) => {
                                        let mut p2p_payload = P2PPayloadFactory::get_file_transfer(session_id);
                                        p2p_payload.payload = file;

                                        session.receive_packet(&content.sender, &content.sender_display_name, &content.receiver, p2p_payload).await;
                                    }
                                    Err(_) => {
                                        //TODO send err 500
                                    }
                                }

                            }
                        }


                    });
                }


            }
            UnwrappedP2PPacket::DataPacket(packet, transport_op) => {
                let session = tachyon_client.get_session(packet.session_id).unwrap();


            }
        }

    }

}


fn handle_slp_payload(
    slp_payload: &RawSlpPayload,
    sender: &EndpointId,
    receiver: &EndpointId
) -> Result<Option<RawSlpPayload>, PayloadError> {
    let error = String::from("error");
    let content_type = slp_payload.get_content_type().unwrap_or(&error);
    match content_type.as_str() {
        "application/x-msnmsgr-transreqbody" => {
            //  let slp_payload_response = SlpPayloadFactory::get_200_ok_direct_connect_bad_port(&slp_payload)?;
            //let mut slp_payload_response = SlpPayloadFactory::get_500_error_direct_connect(slp_payload, String::from("TCPv1"))?; //todo unwrap_or error slp message
            // if self.test > 0 {
            let slp_payload_response = SlpPayloadFactory::get_500_error_direct_connect(
                slp_payload,
                String::from("TCPv1"),
            )
                .unwrap(); //todo unwrap_or error slp message
            //  }

            // self.test += 1;

            // let mut p2p_payload_response = P2PPayloadFactory::get_sip_text_message();
            // p2p_payload_response.set_payload(slp_payload_response.to_string().as_bytes().to_owned());
            return Ok(Some(slp_payload_response));
            // return Err(Errors::PayloadNotComplete);
        }
        "application/x-msnmsgr-sessionreqbody" => {
            //if it's a file transfer request. TODO change this and put it inside slp_payload via an enum
            info!("GOT SESS REQ_BODY");
            if slp_payload.is_200_ok() {
                todo!()
            }
            todo!()
        }
        "application/x-msnmsgr-transrespbody" => {
            let bridge = slp_payload
                .get_body_property(&String::from("Bridge"))
                .unwrap();
            let slp_payload_response = SlpPayloadFactory::get_500_error_direct_connect(
                slp_payload,
                bridge.to_owned(),
            )?;
            return Ok(Some(slp_payload_response));
        }
        "application/x-msnmsgr-sessionclosebody" => {
            //TODO STOP sending when we receive this
            return Err(PayloadError::PayloadBytesMissing);
        }
        _ => {
            info!("not handled slp payload: {:?}", slp_payload);
            return Err(PayloadError::PayloadBytesMissing);
        }
    }
}