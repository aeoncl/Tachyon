use crate::matrix::extensions::msn_user_resolver::FindRoomFromEmail;
use crate::p2p::client::session::{ReceiveMsnObject, SendFileContent, SessionType};
use crate::p2p::client::transport::{Transport, UnwrappedP2PPacket};
use crate::tachyon::client::tachyon_client::TachyonClient;
use log::{debug, info};
use matrix_sdk::media::{MediaFormat, MediaRequestParameters};
use msnp::msnp::error::PayloadError;
use msnp::p2p::v2::factories::{P2PPayloadFactory, P2PTransportPacketFactory};
use msnp::p2p::v2::p2p_transport_packet::P2PTransportPacket;
use msnp::p2p::v2::raw_p2p_payload::RawP2PPayload;
use msnp::p2p::v2::slp::raw_slp_payload::{RawSlpPayload, SlpPayloadFactory, TryFromRawSlpPayload};
use msnp::p2p::v2::slp::session_slp_payload::{SessionInviteRequestPayload, SessionReqInviteContext};
use msnp::p2p::v2::slp::{SlpHeaders, SlpPayload};
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::models::msn_object::MsnObjectType;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::traits::IntoBytes;
use ruma::RoomId;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;

pub async fn handle_p2p_packet(room_id: &RoomId, transport: Transport, p2p_packet: P2PTransportPacket, tachyon_client: TachyonClient) {

    let sorted_packet = match transport.unwrap_packet(p2p_packet).await {
        Ok(packet) => packet,
        Err(e) => {
            log::error!("Could not unwrap P2P packet: {:?}", e);
            return;
        }
    };

    if let Some(packet) = sorted_packet {

        match packet {
            UnwrappedP2PPacket::Slp(slp_payload, transport_op) => {

                //Handle SLP
                let Some(content_type) = slp_payload.get_content_type() else {
                    log::warn!("SLP payload without Content-Type, ignoring: {}", slp_payload.to_string());
                    return;
                };
                let content_type = content_type.trim();

                if content_type == "application/x-msnmsgr-sessionclosebody" {
                    let session_id = slp_payload
                        .get_body_property(&String::from("SessionID"))
                        .ok_or(PayloadError::MandatoryPartNotFound { name: "SessionID".to_string(), payload: slp_payload.to_string() }).unwrap()
                        .parse::<u32>().unwrap();

                    tachyon_client.clear_session(session_id);
                }

                if content_type == "application/x-msnmsgr-sessionreqbody" && slp_payload.is_200_ok() {

                    let session_id = slp_payload
                        .get_body_property(&String::from("SessionID"))
                        .ok_or(PayloadError::MandatoryPartNotFound { name: "SessionID".to_string(), payload: slp_payload.to_string() }).unwrap()
                        .parse::<u32>().unwrap();

                    let session = tachyon_client.get_session(session_id).unwrap();
                    session.accept().unwrap();

                    let matrix_client = tachyon_client.matrix_client();
                    tokio::spawn(async move {

                        let session_type = session.session_type();
                        match session_type {
                            SessionType::ReceiveFile(content) => {
                                let file = matrix_client.media().get_media_content(
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
                            _ => {}
                        }


                    });
                }

                if content_type == "application/x-msnmsgr-sessionreqbody" && slp_payload.is_invite() {

                    let invite = SessionInviteRequestPayload::try_from_raw_slp_payload(slp_payload.clone()).unwrap();

                    match invite.context() {
                        SessionReqInviteContext::MsnObject(obj) => {

                            let (session_id, session) = tachyon_client.create_session(transport.clone(), SessionType::ReceiveMsnObject(ReceiveMsnObject {
                               msn_object: obj.clone()
                            }), invite.session_id());

                            match obj.obj_type {
                                MsnObjectType::Avatar => {}
                                MsnObjectType::CustomEmoticon => {}
                                MsnObjectType::DisplayPicture => {

                                    let sender =  invite.headers().sender().clone();
                                    let receiver = invite.headers().receiver().clone();

                                    let response = SlpPayloadFactory::get_200_ok_session(&slp_payload).unwrap();

                                    let mut packet = P2PPayloadFactory::get_sip_text_message();
                                    packet.set_payload(response.into_bytes());

                                    session.receive_packet(&receiver, "", &sender, packet).await;
                                    session.accept();

                                    let client = tachyon_client.clone();
                                    let proxy_room_email =  EmailAddress::from_str(&obj.creator).unwrap();
                                    let room = client.matrix_client().find_room_from_email(&proxy_room_email).unwrap().unwrap();
                                    tokio::spawn(async move {
                                        let (_, bytes) = client.get_avatar_thumbnail(&room).await.unwrap().unwrap();

                                        //The client expects a data preparation packet before the first data packet of the session.
                                        let data_preparation = P2PPayloadFactory::get_data_preparation_message(session_id);
                                        session.receive_packet(&receiver, "blablabla", &sender, data_preparation).await;

                                        let mut p2p_payload = P2PPayloadFactory::get_msn_obj(session_id);
                                        p2p_payload.payload = bytes;
                                        session.receive_packet(&receiver, "blablabla", &sender, p2p_payload).await;
                                    });


                                }
                                MsnObjectType::SharedFile => {}
                                MsnObjectType::Background => {}
                                MsnObjectType::History => {}
                                MsnObjectType::DynamicDisplayPicture => {}
                                MsnObjectType::Wink => {}
                                MsnObjectType::MapFile => {}
                                MsnObjectType::DynamicBackground => {}
                                MsnObjectType::VoiceClip => {}
                                MsnObjectType::PluginState => {}
                                MsnObjectType::RoamingObject => {}
                                MsnObjectType::SignatureSound => {}
                                MsnObjectType::UnknownYet => {}
                                MsnObjectType::Scene => {}
                                MsnObjectType::WebcamDynamicDisplayPicture => {}
                            }

                        }
                        SessionReqInviteContext::FileTransfer(transfer) => {

                            let (_, session) = tachyon_client.create_session(transport.clone(), SessionType::SendFile(SendFileContent {
                                room_id: room_id.to_owned(),
                                file_size: transfer.get_size(),
                                filename: transfer.get_filename(),
                            }), invite.session_id());

                            let sender =  invite.headers().sender();
                            let receiver = invite.headers().receiver();

                            let response = SlpPayloadFactory::get_200_ok_session(&slp_payload).unwrap();

                            let mut packet = P2PPayloadFactory::get_sip_text_message();
                            packet.set_payload(response.into_bytes());

                            session.receive_packet(receiver, "", sender, packet).await;
                            session.accept();
                        }
                        SessionReqInviteContext::MediaReceiveOnly => {}
                        SessionReqInviteContext::MediaSession => {}
                        SessionReqInviteContext::SharePhoto => {}
                        SessionReqInviteContext::Activity => {}
                    }



                }

                if content_type == "application/x-msnmsgr-transreqbody" {
                    transport.handle_transport_request(slp_payload).await;
                }
            }
            UnwrappedP2PPacket::DataPacket(packet, transport_op) => {
                let session = tachyon_client.get_session(packet.session_id).unwrap();

                match session.session_type() {
                    SessionType::ReceiveFile(_) => {}
                    SessionType::SendFile(content) => {
                        tachyon_client.send_file_buffered(packet.session_id, packet, room_id, &content.filename, content.file_size).await.unwrap();
                    }
                    SessionType::ReceiveMsnObject(_) => {}
                }
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