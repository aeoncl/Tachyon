use std::str::from_utf8_unchecked;

use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose;
use log::error;
use log::info;
use matrix_sdk::media::MediaFormat;
use matrix_sdk::media::MediaRequest;
use matrix_sdk::ruma::events::room::MediaSource;
use matrix_sdk::ruma::MxcUri;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::BufReader;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use crate::models::conversion::audio_conversion::convert_incoming_audio_message;
use crate::{MATRIX_CLIENT_LOCATOR, P2P_REPO};
use crate::models::msn_object::MSNObjectType;
use crate::models::msn_user::MSNUser;
use crate::models::p2p::events::p2p_event::P2PEvent;
use crate::models::p2p::p2p_client::P2PClient;
use crate::models::p2p::pending_packet::PendingPacket;
use crate::repositories::msn_user_repository::MSNUserRepository;
use crate::sockets::msnp2p_command::P2PCommand;
use crate::sockets::msnp2p_command::P2PCommandParser;

use super::tcpserver::TCPServer;

pub struct P2PServer {
    url: String,
    port: u32
}


#[async_trait]
impl TCPServer for P2PServer {

//TODO GET RID OF P2PREPO

    async fn listen(&self) {
        let listener = TcpListener::bind(format!("{}:{}", self.url, self.port))
            .await
            .unwrap();
        
        loop {
            let (mut socket, _addr) = listener.accept().await.unwrap();
            let (tx, mut rx) = mpsc::unbounded_channel::<P2PEvent>();
            let mut p2p_session = P2PClient::new(tx.clone());

            let seq_number=  P2P_REPO.get_seq_number();
            info!("fetched seq_num: {:x}", seq_number);
            p2p_session.set_seq_number(seq_number);
            p2p_session.set_initialized(true);
            // p2p_session.listen_for_raks();

            let matrix_client = MATRIX_CLIENT_LOCATOR.get().unwrap();

            let mut real_user: Option<MSNUser> = None;
            let mut proxy_user: Option<MSNUser> = None;
            let mut do_nonce: bool = true;
            let mut parser = P2PCommandParser::new();

            let _result = tokio::spawn(async move {
                let (read, mut write) = socket.split();
                let mut reader = BufReader::new(read);
                let mut buffer = [0u8; 4096];
                loop {
                    tokio::select! {
                        bytes_read = reader.read(&mut buffer) => {
                           // info!("P2P << BUFFER: {:?}", &buffer);

                            let line = unsafe {from_utf8_unchecked(&buffer)};

                        //    info!("P2P << STRING: {}", &line);
                            let bytes_read = bytes_read.unwrap_or(0);
                            if bytes_read == 0 {
                                break;
                            }

                            let commands: Vec<P2PCommand> = parser.parse_message(&buffer[0..bytes_read], do_nonce);
                            
                            for command in commands {
                            //info!("P2P << msg");

                            info!("P2P << {:?}", &command);

                                    if command.is_nonce() && do_nonce {
                                        //respond with our nonce
                                        let mut buff = [16,0,0,0,0x37,0x29,0x2d,0x12,0x86,0x5c,0x7b,0x4c,0x81,0xf5,0xe,0x5,0x1,0x78,0x80,0xc2];
                                        //LittleEndian::write_u32(&mut buff, 16);
    
                                        info!("P2P >> Nonce");
                                        write.write(&buff).await;
                                        do_nonce = false;
    
                                    } else if command.is_data() {
                                        let p2p_data = command.data.unwrap();
    
                                        if real_user.is_none() {
                                            if let Some(payload) = p2p_data.get_payload() {
                                                if let Ok(slp) = payload.get_payload_as_slp() {
                                                    real_user = slp.get_sender();
                                                    proxy_user = slp.get_receiver();
                                                }
                                            }
                                        }
                                        p2p_session.on_message_received(PendingPacket::new(p2p_data, real_user.as_ref().unwrap_or(&MSNUser::default()).clone(), proxy_user.as_ref().unwrap_or(&MSNUser::default()).clone()));
                                        
                                    } else {
                                        info!("unkown command {:?}", &command);
                                    }
                            }
                            buffer = [0u8; 4096];
                        },
                        command_to_send = rx.recv() => {
                            if let Some(msg) = command_to_send.as_ref() {

                                match msg {
                                    P2PEvent::MSNObjectReceived(_osef_staline) => {

                                    },
                                    P2PEvent::FileReceived(content) => {}
                                    P2PEvent::Message(content) => {

                                        for bytes_to_send in content.as_directs_p2p() {
                                            //let line = unsafe {from_utf8_unchecked(&bytes_to_send.as_slice())};
                                            info!("P2P >> {:?}", &bytes_to_send);
                                            write.write_all(bytes_to_send.as_slice()).await;
                                        }

                                    },
                                    P2PEvent::FileTransferAccepted(content) => {
                                    },
                                   P2PEvent::MSNObjectRequested(content) => {
                                info!("RECEIVED MSNObjectRequestedEvent: {:?}", &content);
                                let msn_obj = &content.msn_object;
                                //TODO ERROR HANDLING

                                let base64decoded_id  = if msn_obj.location == "0" && msn_obj.friendly.is_some(){
                                    general_purpose::STANDARD.decode(msn_obj.friendly.as_ref().unwrap()).expect("MSNObj friendly to be base64encoded")
                                } else {
                                    let base64encoded_id =  msn_obj.location.trim_end_matches(".tmp");
                                    general_purpose::STANDARD.decode(base64encoded_id).expect("MSNObj location to be base64encoded")
                                };

                                match msn_obj.obj_type {
                                    MSNObjectType::Avatar | MSNObjectType::DisplayPicture => {
                                        let avatar_url = String::from_utf8(base64decoded_id).expect("MSNObj location to be UTF-8");
                                        let msn_user_repo = MSNUserRepository::new(matrix_client.clone());
                                        if let Ok(avatar_bytes) = msn_user_repo.get_avatar_from_string(avatar_url.clone()).await {
                                            p2p_session.send_msn_object(content.session_id, content.call_id.clone(), avatar_bytes, content.invitee.clone(), content.inviter.clone());
                                        } else {
                                            error!("Could not download avatar for: {}", &avatar_url);
                                            //TODO sent SLP error
                                        }
                                    },
                                    MSNObjectType::SharedFile => todo!(),
                                    MSNObjectType::Background => todo!(),
                                    MSNObjectType::History => todo!(),
                                    MSNObjectType::DynamicDisplayPicture => todo!(),
                                    MSNObjectType::Wink => todo!(),
                                    MSNObjectType::MapFile => todo!(),
                                    MSNObjectType::DynamicBackground => todo!(),
                                    MSNObjectType::VoiceClip => {
                                        let uri = String::from_utf8(base64decoded_id).expect("MSNObj location to be UTF-8");
                                        let owned_mxc_uri = <&MxcUri>::try_from(uri.as_str()).unwrap().to_owned();



                                        let media_request = MediaRequest{ source: MediaSource::Plain(uri.into()), format: MediaFormat::File };
                                        let media_client = &matrix_client.media();
                                        let media = media_client.get_media_content(&media_request, true).await.unwrap(); //TODO exception handling

                                        let converted_media = convert_incoming_audio_message(media).await.unwrap(); //TODO change this double conversion shit


                                        p2p_session.send_msn_object(content.session_id, content.call_id.clone(), converted_media, content.invitee.clone(), content.inviter.clone());
                                        //TODO sent SLP error
                                    },
                                    MSNObjectType::PluginState => todo!(),
                                    MSNObjectType::RoamingObject => todo!(),
                                    MSNObjectType::SignatureSound => todo!(),
                                    MSNObjectType::UnknownYet => todo!(),
                                    MSNObjectType::Scene => todo!(),
                                    MSNObjectType::WebcamDynamicDisplayPicture => todo!(),
                                    MSNObjectType::CustomEmoticon => todo!()
                                }
                            }
                                }
                            } else {
                                info!("P2P >> BAD COMMAND {:?}", &command_to_send);

                            }
                            
                           
                        }
                    }
                }
                //cleanup
            });
        }
    }



}

impl P2PServer {
    pub fn new(
        url: String,
        port: u32,
    ) -> P2PServer {
        return P2PServer {
            url,
            port,
        };
    }
}