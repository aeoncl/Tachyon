use std::path::Path;
use std::str::FromStr;
use anyhow::anyhow;
use lazy_static_include::syn::BinOp::Ne;
use log::{debug, warn};
use matrix_sdk::RoomMemberships;
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use tokio::sync::broadcast;
use tokio::sync::mpsc::Sender;
use msnp::msnp::notification::command::command::{NotificationClientCommand, NotificationServerCommand};
use msnp::msnp::notification::command::cvr::CvrServer;
use msnp::msnp::notification::command::iln::IlnServer;
use msnp::msnp::notification::command::msg::{MsgPayload, MsgServer};
use msnp::msnp::notification::command::nfy::{NfyOperation, NfyServer};
use msnp::msnp::notification::command::nln::NlnServer;
use msnp::msnp::notification::command::ubx::{ExtendedPresenceContent, UbxPayload, UbxServer};
use msnp::msnp::notification::command::usr::{AuthPolicy, OperationTypeClient, OperationTypeServer, SsoPhaseClient, SsoPhaseServer, UsrServer};
use msnp::msnp::notification::command::uum::UumPayload;
use msnp::msnp::notification::command::uux::UuxPayload;
use msnp::msnp::notification::models::endpoint_data::EndpointData;
use msnp::msnp::notification::models::endpoint_guid::EndpointGuid;
use msnp::msnp::notification::models::msnp_version::MsnpVersion::MSNP18;
use msnp::msnp::raw_command_parser::RawCommand;
use msnp::shared::models::capabilities::ClientCapabilities;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::endpoint_id::EndpointId;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::network_id::NetworkId;
use msnp::shared::models::network_id_email::NetworkIdEmail;
use msnp::shared::models::presence_status::PresenceStatus;
use msnp::shared::models::uuid::Uuid;
use msnp::shared::payload::msg::raw_msg_payload::factories::RawMsgPayloadFactory;
use msnp::shared::payload::msg::text_msg::FontStyle;
use msnp::shared::payload::nfy::nfy_put_payload::RawNfyPayload;
use crate::matrix;
use crate::notification::client_store::{ClientData, ClientStoreFacade};
use crate::notification::notification_server::{LocalStore, Phase};
use crate::shared::identifiers::{MatrixDeviceId, MatrixIdCompatible};
use crate::shared::msn_user_resolver;


pub(crate) async fn handle_negotiation(raw_command: NotificationClientCommand, notif_sender: Sender<NotificationServerCommand>, mut local_store: &mut LocalStore) -> Result<(), anyhow::Error> {
    match raw_command {
        NotificationClientCommand::VER(command) => {
            if command.first_candidate != MSNP18 && command.second_candidate != MSNP18 {
                //Unsupported protocol version
                //TODO add error code
                notif_sender.send(NotificationServerCommand::OUT).await?;
                return Ok(());
            }

            notif_sender.send(NotificationServerCommand::VER(command.get_response_for(MSNP18))).await;
            Ok(())
        },
        NotificationClientCommand::CVR(command) => {
            local_store.phase = crate::notification::notification_server::Phase::Authenticating;
            notif_sender.send(NotificationServerCommand::CVR(CvrServer::new(command.tr_id, "14.0.8117.0416".to_string(), "14.0.8117.0416".to_string(), "14.0.8117.0416".to_string(), "localhost".to_string(), "localhost".to_string() ))).await?;
            Ok(())
        },
        _ => {
            Err(anyhow!("WTF are you doing here"))
        }
    }
}


const SHIELDS_PAYLOAD: &str = "<Policies><Policy type= \"SHIELDS\"><config><shield><cli maj= \"7\" min= \"0\" minbld= \"0\" maxbld= \"9999\" deny= \" \" /></shield><block></block></config></Policy><Policy type= \"ABCH\"><policy><set id= \"push\" service= \"ABCH\" priority= \"200\"><r id= \"pushstorage\" threshold= \"0\" /></set><set id= \"using_notifications\" service= \"ABCH\" priority= \"100\"><r id= \"pullab\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /><r id= \"pullmembership\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /></set><set id= \"delaysup\" service= \"ABCH\" priority= \"150\"><r id= \"whatsnew\" threshold= \"0\" /><r id= \"whatsnew_storage_ABCH_delay\" timer= \"1800000\" /><r id= \"whatsnewt_link\" threshold= \"0\" trigger= \"QueryActivities\" /></set><c id= \"PROFILE_Rampup\">100</c></policy></Policy><Policy type= \"ERRORRESPONSETABLE\"><Policy><Feature type= \"3\" name= \"P2P\"><Entry hr= \"0x81000398\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /></Feature><Feature type= \"4\"><Entry hr= \"0x81000440\" /></Feature><Feature type= \"6\" name= \"TURN\"><Entry hr= \"0x8007274C\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /><Entry hr= \"0x8007274A\" action= \"3\" /></Feature></Policy></Policy><Policy type= \"P2P\"><ObjStr SndDly= \"1\" /></Policy></Policies>";

pub(crate) async fn handle_auth(raw_command: NotificationClientCommand, notif_sender: Sender<NotificationServerCommand>, client_store: &ClientStoreFacade, mut local_store: &mut LocalStore, kill_signal: &broadcast::Receiver<()>) -> Result<(), anyhow::Error> {
    match raw_command {
        NotificationClientCommand::USR(command) => {
            match command.auth_type {
                OperationTypeClient::Sso(content) => {
                    match content {
                        SsoPhaseClient::I { email_addr } => {
                            local_store.email_addr = email_addr;
                            let usr_response = UsrServer::new(command.tr_id, OperationTypeServer::Sso(SsoPhaseServer::S { policy: AuthPolicy::MbiKeyOld, nonce: "LAhAAUzdC+JvuB33nooLSa6Oh0oDFCbKrN57EVTY0Dmca8Reb3C1S1czlP12N8VU".to_string() }));
                            let gcf_response = RawCommand::with_payload("GCF 0", SHIELDS_PAYLOAD.as_bytes().to_vec());

                            notif_sender.send(NotificationServerCommand::USR(usr_response)).await?;
                            notif_sender.send(NotificationServerCommand::RAW(gcf_response)).await?;
                        },

                        SsoPhaseClient::S { ticket_token, challenge, endpoint_guid } => {

                            let user_id = local_store.email_addr.to_owned_user_id();

                            let device_id = format!("tachyon3-{}", MatrixDeviceId::from_hostname()?.to_string());


                            let matrix_client = matrix::login::login(user_id, device_id, ticket_token.clone(), &Path::new(format!("C:\\temp\\{}", &local_store.email_addr).as_str()), None, true).await?;

                            let endpoint_id = EndpointId::new(local_store.email_addr.clone(), Some(endpoint_guid));
                            let msn_user = MsnUser::new(endpoint_id);

                            let client_data = ClientData::new(msn_user.clone(), ticket_token.clone(), matrix_client.clone());
                            client_store.insert_client_data(ticket_token.as_str().to_owned(), client_data.clone());

                            local_store.token = ticket_token.clone();
                            local_store.client_data = Some(client_data.clone());
                            local_store.phase = Phase::Ready;

                            let usr_response = UsrServer::new(command.tr_id, OperationTypeServer::Ok {
                                email_addr: local_store.email_addr.clone(),
                                verified: true,
                                unknown_arg: false,
                            });

                            notif_sender.send(NotificationServerCommand::USR(usr_response)).await?;
                            notif_sender.clone().send(NotificationServerCommand::RAW(RawCommand::without_payload("SBS 0 null"))).await?;

                            let initial_profile_msg = NotificationServerCommand::MSG(MsgServer {
                                sender: "Hotmail".to_string(),
                                display_name: "Hotmail".to_string(),
                                payload: MsgPayload::Raw(RawMsgPayloadFactory::get_msmsgs_profile(&msn_user.uuid.get_puid(), &local_store.email_addr, &ticket_token))
                            });

                            notif_sender.send(initial_profile_msg).await?;

                            let notif_sender_ed = notif_sender.clone();
                            let email_addr = msn_user.get_email_address().clone();
                            tokio::spawn(async move {
                                //Todo fetch endpoint data
                                let endpoint_data = b"<Data></Data>";
                                notif_sender_ed.send(NotificationServerCommand::RAW(RawCommand::with_payload(&format!("UBX 1:{}", &email_addr.as_str()), endpoint_data.to_vec()))).await;
                            });

                            let kill_signal_clone = kill_signal.resubscribe();
                            tokio::spawn(async move {
                                matrix::sync::start_sync_task(matrix_client, notif_sender, client_data, kill_signal_clone).await;
                            });


                        }

                    }
                },
                OperationTypeClient::Sha(_) => {
                    //return unauth error
                    todo!()
                }

            }
            Ok(())
        },

        _ => {todo!()}
    }

}


pub(crate) async fn handle_command(raw_command: NotificationClientCommand, notif_sender: Sender<NotificationServerCommand>, client_data: ClientData, mut local_store: &mut LocalStore, kill_signal: &broadcast::Receiver<()>) -> Result<(), anyhow::Error> {
    match raw_command {
        NotificationClientCommand::USR(command) => {
            match command.auth_type {

                OperationTypeClient::Sso(_) => {
                    todo!()
                    //return error;
                },
                OperationTypeClient::Sha(phase) => {

                    let usr_response = UsrServer::new(command.tr_id, OperationTypeServer::Ok {
                        email_addr: local_store.email_addr.clone(),
                        verified: true,
                        unknown_arg: false,
                    });
                    notif_sender.send(NotificationServerCommand::USR(usr_response)).await?;
                }

            }
            Ok(())
        },
        NotificationClientCommand::PNG => {
            notif_sender.send(NotificationServerCommand::QNG(60)).await?;
            Ok(())
        }
        //The client waits indefinitely for initial ADL Response, useful if we need time to sync contacts without hitting the timeout :D
        NotificationClientCommand::ADL(command) => {

            debug!("ADL: {:?}", &command);

            let contacts = command.payload.get_contacts()?;

            client_data.inner.contact_list.lock().unwrap().add_contacts(contacts, command.payload.is_initial());

            notif_sender.send(NotificationServerCommand::Ok(command.get_ok_response("ADL"))).await?;

            Ok(())
        }
        NotificationClientCommand::RML(command) => {
            debug!("RML: {:?}", &command);

            client_data.inner.contact_list.lock().unwrap().remove_contacts(command.payload.get_contacts()?);
            notif_sender.send(NotificationServerCommand::Ok(command.get_ok_response("RML"))).await?;

            Ok(())
        }
        NotificationClientCommand::UUX(command) => {
            let ok_resp = command.get_ok_response();

            match command.payload {
                None => {}
                Some(payload) => {
                    match payload {
                        UuxPayload::PrivateEndpointData(private_endpoint_data) => {
                            local_store.private_endpoint_data = private_endpoint_data;
                            //TODO
                        }
                        UuxPayload::Unknown(_) => {}
                    }
                }
            }

            notif_sender.send(NotificationServerCommand::Uux(ok_resp)).await?;
            Ok(())
        },
        NotificationClientCommand::UUM(command) => {

            let ok_response = command.get_ok_response();

            match command.payload {
                UumPayload::TextMessage(content) => {
                    let matrix_client = client_data.get_matrix_client();
                    let room = matrix_client.get_dm_room(&command.destination.email_addr.to_owned_user_id());
                    match room {
                        None => {
                            //NO DM ROOM FOUND
                        }
                        Some(room) => {
                            //TODO SMILEY TO EMOJI
                            //TODO Store event id for dedup

                            let content = if content.is_styling_default() {
                                RoomMessageEventContent::text_plain(content.body)
                            } else {
                                let mut message = content.body.clone();

                                if !content.is_default_font_styles() {
                                    if content.font_styles.matches(FontStyle::Bold) {
                                        message = format!("<b>{}</b>", message)
                                    }

                                    if content.font_styles.matches(FontStyle::Italic) {
                                        message = format!("<i>{}</i>", message)
                                    }

                                    if content.font_styles.matches(FontStyle::Underline) {
                                        message = format!("<u>{}</u>", message)
                                    }

                                    if content.font_styles.matches(FontStyle::StrikeThrough) {
                                        message = format!("<strike>{}</strike>", message)
                                    }
                                }

                                let color_attr = if content.is_default_font_color() { String::new() } else { format!(" color=\"{}\"", content.font_color.serialize_rgb())};
                                let face_attr = if content.is_default_font() { String::new() } else { format!(" face=\"{}\"", content.font_family) };
                                message = format!("<font{}{}>{}</font>",  color_attr, face_attr, message);

                                RoomMessageEventContent::text_html(content.body, message)
                            };

                            let response = room.send(content).await?;
                            //self.add_to_events_sent(response.event_id.to_string());
                            notif_sender.send(NotificationServerCommand::Ok(ok_response)).await?;
                        }
                    }

                    Ok(())
                },
                UumPayload::TypingUser(_) => {
                    todo!()
                }
                UumPayload::Nudge(_) => {
                    todo!()

                }
                UumPayload::Raw(_) => {
                    todo!()
                }
            }
        }
        NotificationClientCommand::BLP(command) => {
            notif_sender.send(NotificationServerCommand::BLP(command)).await?;
            Ok(())
        }
        NotificationClientCommand::CHG(command) => {
            notif_sender.send(NotificationServerCommand::CHG(command.clone())).await?;


            let contacts = {
                let lock = client_data.inner.contact_list.lock().unwrap();
                lock.get_forward_list()
            };


            for (contact) in contacts {

                if contact.is_from_network(NetworkId::WindowsLive) && local_store.needs_initial_presence {
                    let msn_user = msn_user_resolver::resolve_msn_user(&contact.email_address.to_owned_user_id(), None, &client_data, true, true).await?;
                    notif_sender.send(NotificationServerCommand::ILN(IlnServer {
                        tr_id: command.tr_id,
                        presence_status: PresenceStatus::NLN,
                        target_user: contact.get_network_id_email(),
                        via: None,
                        display_name: msn_user.compute_display_name().to_string(),
                        client_capabilities: msn_user.capabilities.clone(),
                        avatar: None,
                        badge_url: None,
                    })).await?;

                    notif_sender.send(NotificationServerCommand::UBX(UbxServer{
                        target_user: contact.get_network_id_email(),
                        via: None,
                        payload: UbxPayload::ExtendedPresence(ExtendedPresenceContent {
                            psm: msn_user.psm.clone(),
                            current_media: "".to_string(),
                            endpoint_data: EndpointData { machine_guid: msn_user.endpoint_id.endpoint_guid.clone(), capabilities: Default::default() },
                            private_endpoint_data: None,
                        })
                    })).await?;

                } else if contact.is_from_network(NetworkId::Circle) {

                        let circle_target_user = contact.get_network_id_email();
                        let (local_part, _domain) = circle_target_user.email.crack();
                        //TODO have a store of UUID room mappings
                        let circle_uuid = Uuid::from_str(local_part).unwrap();
                        let rooms = client_data.get_matrix_client().rooms();
                        let found = rooms.iter().find(|r| {
                            let room_uuid = Uuid::from_seed(r.room_id().as_str());
                            room_uuid == circle_uuid
                        }).unwrap();


                    let presence_body = format!("<circle><props><presence dtype=\"xml\"><Data><UTL></UTL><MFN>{}</MFN><PSM></PSM><CurrentMedia></CurrentMedia></Data></presence></props></circle>", found.name().unwrap());

                    let mut presence_payload = RawNfyPayload::new_circle(contact.get_network_id_email(), NetworkIdEmail::new(NetworkId::WindowsLive, local_store.email_addr.clone()));
                    presence_payload.set_body_string(presence_body);

                    notif_sender.send(NotificationServerCommand::NFY(NfyServer {
                        operation: NfyOperation::Put,
                        payload: presence_payload
                    })).await?;


                        let members = found.members_no_sync(RoomMemberships::JOIN).await.unwrap();

                        for member in members {

                            let msn_user = msn_user_resolver::resolve_msn_user_from_rm(&member, &client_data, true, true).await?;

                            //TODO avatar

                            notif_sender.send(NotificationServerCommand::NLN(NlnServer {
                                presence_status: PresenceStatus::NLN,
                                target_user: msn_user.get_network_id_email(),
                                via: Some(circle_target_user.clone()),
                                display_name: msn_user.compute_display_name().to_string(),
                                client_capabilities: msn_user.capabilities.clone(),
                                avatar: None,
                                badge_url: None
                            })).await?;

                            notif_sender.send(NotificationServerCommand::UBX(UbxServer{
                                target_user: msn_user.get_network_id_email(),
                                via: Some(circle_target_user.clone()),
                                payload: UbxPayload::ExtendedPresence(ExtendedPresenceContent {
                                    psm: msn_user.psm.clone(),
                                    current_media: "".to_string(),
                                    endpoint_data: EndpointData { machine_guid: msn_user.endpoint_id.endpoint_guid.clone(), capabilities: Default::default() },
                                    private_endpoint_data: None,
                                })
                            })).await?;




                            let roster_body = format!("<circle><roster><id>IM</id><user><id>{network_id_email}</id></user></roster></circle>", network_id_email = msn_user.get_network_id_email());

                            let mut roster_payload = RawNfyPayload::new_circle_partial(contact.get_network_id_email(), NetworkIdEmail::new(NetworkId::WindowsLive, local_store.email_addr.clone()));
                            roster_payload.set_body_string(roster_body);

                            notif_sender.send(NotificationServerCommand::NFY(NfyServer {
                                operation: NfyOperation::Put,
                                payload: roster_payload
                            })).await?;

                        }
                }






            }
            local_store.needs_initial_presence = false;

            let me = client_data.get_user_clone().unwrap();

            notif_sender.send(NotificationServerCommand::NLN(NlnServer {
                presence_status: PresenceStatus::NLN,
                target_user: NetworkIdEmail::new(NetworkId::WindowsLive, local_store.email_addr.clone()),
                via: None,
                display_name: local_store.email_addr.to_string(),
                client_capabilities: Default::default(),
                avatar: None,
                badge_url: None,
            })).await?;

            notif_sender.send(NotificationServerCommand::UBX(UbxServer{
                target_user: NetworkIdEmail::new(NetworkId::WindowsLive, local_store.email_addr.clone()),
                via: None,
                payload: UbxPayload::ExtendedPresence(ExtendedPresenceContent {
                    psm: "".to_string(),
                    current_media: "".to_string(),
                    endpoint_data: EndpointData { machine_guid: me.endpoint_id.endpoint_guid, capabilities: Default::default() },
                    private_endpoint_data: Some(local_store.private_endpoint_data.clone()),
                })
            })).await?;

            Ok(())
        }
        NotificationClientCommand::PRP(command) => {Ok(())}
        NotificationClientCommand::UUN(command) => {Ok(())}
        NotificationClientCommand::XFR() => {Ok(())}
        NotificationClientCommand::RAW(command) => {
            warn!("Received RAW command: {:?}", command);
            Ok(())
        },
        NotificationClientCommand::PUT(command) => {



            let ok = command.get_ok_command();
            notif_sender.send(NotificationServerCommand::PUT(ok)).await?;

            let mut payload = command.payload;
            payload.envelope.swap_sides();
            payload.envelope.flags = None;

            notif_sender.send(NotificationServerCommand::NFY(NfyServer {
                operation: NfyOperation::Put,
                payload
            })).await?;

            Ok(())
        },
        NotificationClientCommand::OUT => {Ok(())}
        _ => {
            warn!("Received unknown command");
            Ok(())
        }
    }
}

