use std::path::Path;
use std::str::FromStr;

use anyhow::anyhow;
use async_trait::async_trait;
use log::{info, warn};
use matrix_sdk::Client;
use substring::Substring;
use tokio::sync::broadcast::{self};
use tokio::sync::{mpsc, oneshot};
use tokio::sync::mpsc::{Sender, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;

use crate::{MATRIX_CLIENT_LOCATOR, MSN_CLIENT_LOCATOR, SETTINGS_LOCATOR};
use crate::generated::msnab_datatypes::types::RoleId;
use crate::generated::payloads::{PresenceStatus, PrivateEndpointData};
use crate::matrix::loop_bootstrap::listen;
use crate::matrix::matrix_client::login;
use crate::models::msg_payload::factories::MsgPayloadFactory;
use crate::models::msn_object::MSNObject;
use crate::models::msn_user::MSNUser;
use crate::models::notification::adl_payload::ADLPayload;
use crate::models::notification::error::{MSNPErrorCode, MSNPServerError};
use crate::models::notification::events::notification_event::NotificationEvent;
use crate::models::notification::msn_client::MSNClient;
use crate::models::notification::user_notification_type::UserNotificationType;
use crate::models::p2p::slp_payload::SlpPayload;
use crate::models::p2p::slp_payload_handler::SlpPayloadHandler;
use crate::models::tachyon_error::{MatrixError, TachyonError};
use crate::models::tachyon_error::PayloadError::StringPayloadParsingError;
use crate::models::uuid::UUID;
use crate::repositories::msn_user_repository::MSNUserRepository;
use crate::repositories::repository::Repository;
use crate::utils::identifiers::trim_endpoint_guid;
use crate::utils::string::decode_url;

use super::command_handler::CommandHandler;
use super::msnp_command::MSNPCommand;

pub struct NotificationCommandHandler {
    matrix_token: String,
    msn_addr: String,
    sender: UnboundedSender<String>,
    msnp_version: i16,
    msn_client: Option<MSNClient>,
    matrix_client: Option<Client>,
    matrix_loop_killer: Option<oneshot::Sender<()>>,
    needs_initial_presence: bool

}

impl NotificationCommandHandler {

    fn start_receiving(&mut self, mut notification_receiver: UnboundedReceiver<NotificationEvent>) {

        let sender = self.sender.clone();
        tokio::spawn(async move {

            let (ab_notify_task_stop_sender, ab_notify_task_stop_receiver) = broadcast::channel::<()>(1);
            let mut join_handle: Option<JoinHandle<()>> = None;
            loop {
                tokio::select! {
                    command_to_send = notification_receiver.recv() => {

                        //Todo handle Errors
                        if let Some(msg) = command_to_send {
                            match msg {
                                NotificationEvent::AddressBookUpdateEvent(content) => {

                                    info!("ABNotify: Sending AB Notify");

                                    let sender = sender.clone();
                                   // let mut stop_receiver = ab_notify_task_stop_sender.subscribe();
                                    let _result = sender.send(format!("NOT {payload_size}\r\n{payload}", payload_size = content.payload.len(), payload = content.payload));
                                    _result.expect("SENDING NOT, NOT TO BE AN ERROR");

                                    // if let Some(jh) = join_handle {
                                    //     if !jh.is_finished() {
                                    //         ab_notify_task_stop_sender.send(());
                                    //     }
                                    // }
                                    //
                                    // join_handle = Some(task::spawn(async move {
                                    //
                                    //     info!("ABNotify: Spawning ab notify task");
                                    //
                                    //     let mut interval = time::interval(Duration::from_secs(5));
                                    //     interval.tick().await;
                                    //
                                    //     tokio::select! {
                                    //         abort = stop_receiver.recv()  => {
                                    //             info!("ABNotify: Aborting task");
                                    //             return;
                                    //         },
                                    //         tick = interval.tick() => {
                                    //             let _result = sender.send(format!("NOT {payload_size}\r\n{payload}", payload_size = content.payload.len(), payload = content.payload));
                                    //             info!("ABNotify: Send command");
                                    //             return;
                                    //         }
                                    //     }
                                    // }));

                                },
                                NotificationEvent::HotmailNotificationEvent(content) => {
                                    let _result = sender.send(format!("NOT {payload_size}\r\n{payload}", payload_size = content.payload.len(), payload = content.payload));
                                },
                                NotificationEvent::DisconnectEvent(content) => {
                                    let user = &content.msn_user;
                                    let mut msn_obj = String::new();
                                    if let Some(display_pic) = user.get_display_picture().as_ref() {
                                        msn_obj = display_pic.to_string();
                                    }
                                    let _result = sender.send(format!("NLN HDN 1:{msn_addr} {nickname} {client_capabilities} {msn_obj}\r\n", client_capabilities= &user.get_capabilities() ,msn_addr= &user.get_msn_addr(), nickname= &user.get_display_name(), msn_obj = &msn_obj));                            
                                    let ubx_payload = format!("<PSM>{status_msg}</PSM><CurrentMedia></CurrentMedia><EndpointData id=\"{{{machine_guid}}}\"><Capabilities>{client_capabilities}</Capabilities></EndpointData>", status_msg = &user.get_psm(), client_capabilities= &user.get_capabilities(), machine_guid = &user.get_endpoint_guid());
                                    let _result = sender.send(format!("UBX 1:{msn_addr} {ubx_payload_size}\r\n{ubx_payload}", msn_addr = &user.get_msn_addr(), ubx_payload_size= ubx_payload.len(), ubx_payload=ubx_payload));
                                   
                                   // let _result = sender.send(format!("FLN 1:{msn_addr}\r\n", msn_addr = content.msn_addr));
                                },
                                NotificationEvent::PresenceEvent(content) => {
                                    let user = &content.user;
                                    NotificationCommandHandler::send_presence_for_user(user, &sender);
                                },
                                NotificationEvent::SwitchboardInitEvent(content) => {
                                    let _result = sender.send(format!("RNG {session_id} {sb_ip_addr}:{sb_port} CKI {ticket} {invite_passport} {invite_name} U messenger.msn.com 1\r\n",
                                    sb_ip_addr = "127.0.0.1",
                                    sb_port = 1864,
                                    invite_passport = &content.invite_passport,
                                    invite_name = &content.invite_name,
                                    session_id = &content.session_id,
                                    ticket = &content.ticket
                                ));
                                },
                                _ => {
                                    
                                }
                            }
                        } else {
                            info!("NotificationEvent pipe was closed -> breaking out the loop.");
                            break;
                        }
                      
                    }
                }
            }
        });
    }
    


    pub fn new(sender: UnboundedSender<String>) -> NotificationCommandHandler {
        return NotificationCommandHandler {
            sender,
            matrix_token: String::new(),
            msn_client: None,
            matrix_client: None,
            msnp_version: -1,
            msn_addr: String::new(),
            matrix_loop_killer: None,
            needs_initial_presence: true,
        };
    }
}




#[async_trait]
impl CommandHandler for NotificationCommandHandler {

    // FQY 138 53 | <ml><d n="shlasouf.local"><c n="aeontest4"/></d></ml> We received this command after deletion of contact, its not documented anywhere
    // SDC 17 aeontest4@shlasouf.local 0x0409 MSNMSGR msmsgs X X Aeonshl 4 | HEYY ======== Seems like the command to sednd the invite message by email
    async fn handle_command(&mut self, command: &MSNPCommand) -> Result<String, MSNPServerError> {
        let split = command.split();
        match command.operand.as_str() {
            "VER" => {
                //   0  1    2      3     4
                //=>VER 1 MSNP18 MSNP17 CVR0\r\n
                //<=VER 1 MSNP18\r\n

                let tr_id = split.get(1)
                    .ok_or(MSNPServerError::fatal_error_no_source(format!("VER trid is missing: {:?}", &command)))?;

                let ver: i16 = split[2]
                    .substring(4, split[2].chars().count())
                    .parse::<i16>()
                    .map_err(|e| MSNPServerError::new(true, Some(tr_id.to_string()), MSNPErrorCode::InvalidParameter, StringPayloadParsingError { payload: command.to_string(), sauce: anyhow!(e) }.into()))?;

                self.msnp_version = ver;
                return Ok(format!("VER {} MSNP{}\r\n", tr_id, ver).to_string());
            }
            "CVR" => {
                //    0  1    2     3     4    5      6          7          8          9
                //=> CVR 2 0x0409 winnt 6.0.0 i386 MSNMSGR 14.0.8117.0416 msmsgs login@email.com
                self.msn_addr = split[9].to_string();                
                let tr_id = split[1];
                let version = split[7];
                //<= CVR 2 14.0.8117.0416 14.0.8117.0416 14.0.8117.0416 localhost localhost
                return Ok(format!(
                    "CVR {tr_id} {version} {version} {version} {host} {host}\r\n",
                    tr_id = tr_id,
                    version = version,
                    host = "localhost"
                ));
            }
            "USR" => {
                /*
                I phase :
                        0   1  2  3      4
                    >>> USR 3 SSO I login@test.com
                    <<< USR 3 SSO S MBI_KEY_OLD LAhAAUzdC+JvuB33nooLSa6Oh0oDFCbKrN57EVTY0Dmca8Reb3C1S1czlP12N8VU
                S phase :
                        0   1  2  3     4      5                          6
                    >>> USR 4 SSO S t=ssotoken ???charabia {55192CF5-588E-4ABE-9CDF-395B616ED85B}
                    <<< USR 4 OK login@test.com 1 0
                */
                let tr_id = split[1];
                let auth_type = split[2];
                let phase = split[3];
                
                if auth_type == "SHA" {
                    return Ok(format!("USR {tr_id} OK {email} 1 0\r\n", tr_id=tr_id, email=self.msn_addr));
                } else if auth_type == "SSO" {
                    if phase == "I" {
                        let login = split[4];
                        let shields_payload = "<Policies><Policy type= \"SHIELDS\"><config><shield><cli maj= \"7\" min= \"0\" minbld= \"0\" maxbld= \"9999\" deny= \" \" /></shield><block></block></config></Policy><Policy type= \"ABCH\"><policy><set id= \"push\" service= \"ABCH\" priority= \"200\"><r id= \"pushstorage\" threshold= \"0\" /></set><set id= \"using_notifications\" service= \"ABCH\" priority= \"100\"><r id= \"pullab\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /><r id= \"pullmembership\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /></set><set id= \"delaysup\" service= \"ABCH\" priority= \"150\"><r id= \"whatsnew\" threshold= \"0\" /><r id= \"whatsnew_storage_ABCH_delay\" timer= \"1800000\" /><r id= \"whatsnewt_link\" threshold= \"0\" trigger= \"QueryActivities\" /></set><c id= \"PROFILE_Rampup\">100</c></policy></Policy><Policy type= \"ERRORRESPONSETABLE\"><Policy><Feature type= \"3\" name= \"P2P\"><Entry hr= \"0x81000398\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /></Feature><Feature type= \"4\"><Entry hr= \"0x81000440\" /></Feature><Feature type= \"6\" name= \"TURN\"><Entry hr= \"0x8007274C\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /><Entry hr= \"0x8007274A\" action= \"3\" /></Feature></Policy></Policy><Policy type= \"P2P\"><ObjStr SndDly= \"1\" /></Policy></Policies>";
                        return Ok(format!("USR {tr_id} SSO S MBI_KEY_OLD LAhAAUzdC+JvuB33nooLSa6Oh0oDFCbKrN57EVTY0Dmca8Reb3C1S1czlP12N8VU\r\nGCF 0 {shields_size}\r\n{shields_payload}", tr_id = tr_id, shields_payload = shields_payload, shields_size = shields_payload.len()));
                    } else if phase == "S" {
                        self.matrix_token = split[4][2..split[4].chars().count()].to_string();
                        
                        let endpoint_guid = split[6].to_string();
                        let trimmed_endpoint_guid = trim_endpoint_guid(split[6]).map_err(|e| MSNPServerError::from_source_with_trid(tr_id.to_string(), e.into()))?;



                        let mut msn_user = MSNUser::new(self.msn_addr.clone());
                        msn_user.set_endpoint_guid(trimmed_endpoint_guid.to_string());



                        let matrix_client = login(msn_user.get_matrix_id(), self.matrix_token.clone(), &Path::new(format!("C:\\temp\\{}", &self.msn_addr).as_str()), SETTINGS_LOCATOR.homeserver_url.clone(), true).await.map_err(|e| MSNPServerError::from_source_with_trid(tr_id.to_string(), e))?;

                        let room_id = OwnedRoomId::from_str("fdasfas");
                        let room = matrix_client.get_room(&room_id).unwrap();
                        room.timeline();
                        
                        let (notification_event_sender, mut notification_event_receiver) = mpsc::unbounded_channel::<NotificationEvent>();
                        let mut msn_client = MSNClient::new(matrix_client.clone(), msn_user.clone(), self.msnp_version,  notification_event_sender.clone());

                        //Token valid, client authenticated. Initializing shared data structures
                        self.msn_client = Some(msn_client.clone());
                        self.matrix_client = Some(matrix_client.clone());
                        MATRIX_CLIENT_LOCATOR.set(matrix_client.clone());
                        MSN_CLIENT_LOCATOR.set(msn_client.clone());

                        self.sender.send(format!("USR {tr_id} OK {email} 1 0\r\nSBS 0 null\r\n", tr_id = &tr_id, email=&self.msn_addr));

                        let msmsgs_profile_msg = MsgPayloadFactory::get_msmsgs_profile(&msn_client.get_user().get_puid(), self.msn_addr.clone(), self.matrix_token.clone()).serialize();
  
                        self.sender.send(format!("MSG Hotmail Hotmail {profile_payload_size}\r\n{profile_payload}", profile_payload_size=msmsgs_profile_msg.len(), profile_payload=&msmsgs_profile_msg));

                        let oim_payload = MsgPayloadFactory::get_initial_mail_data_notification().serialize();
                          
                        self.sender.send(format!("MSG Hotmail Hotmail {oim_payload_size}\r\n{oim_payload}", oim_payload_size=oim_payload.len(), oim_payload=&oim_payload));

                        let mut mpop_endpoints = msn_client.get_mpop_endpoints().await.map_err(|e| MSNPServerError::from_source(e.into()))?;


                        let endpoints = mpop_endpoints.drain(..).map(|e| e.to_string()).reduce(|cur: String, nxt: String| cur + &nxt).unwrap_or_default();

                        let endpoints_payload = format!("<Data>{endpoints}</Data>", endpoints = endpoints);
                            self.sender.send(format!("UBX 1:{email} {private_endpoint_payload_size}\r\n{private_endpoint_payload}", email=self.msn_addr, private_endpoint_payload_size= endpoints_payload.len(), private_endpoint_payload=&endpoints_payload));


                        let test_msg = MsgPayloadFactory::get_system_msg(String::from("1"), String::from("17"), String::from("2"));
                        let serialized = test_msg.serialize();
                        self.sender.send(format!("MSG Hotmail Hotmail {payload_size}\r\n{payload}", payload_size=serialized.len(), payload=&serialized));

                        self.start_receiving(notification_event_receiver);

                        let matrix_loop_killer = listen(matrix_client.clone(), msn_client);
                        self.matrix_loop_killer.insert(matrix_loop_killer);
                       // return Ok(format!("USR {tr_id} OK {email} 1 0\r\nSBS 0 null\r\nMSG Hotmail Hotmail {msmsgs_profile_payload_size}\r\n{payload}MSG Hotmail Hotmail {oim_payload_size}\r\n{oim_payload}UBX 1:{email} {private_endpoint_payload_size}\r\n{private_endpoint_payload}", tr_id = tr_id, email=&self.msn_addr, msmsgs_profile_payload_size= msmsgs_profile_msg.len(), payload=msmsgs_profile_msg, oim_payload = oim_payload, oim_payload_size = oim_payload.len(), private_endpoint_payload_size = endpoints_payload.len(), private_endpoint_payload = endpoints_payload));
                        return Ok(String::new());
                    }
                }

                return Ok(String::new());
            },
            "PNG" => {
                return Ok(String::from("QNG 60\r\n"));
            },
            "ADL" => {
                /*       0  1  2   payload
                    >>> ADL 6 68 <ml l="1"><d n="matrix.org"><c n="u.user" l="3" t="1"/></d></ml>
                    <<< ADL 6 OK
                */
                let tr_id = split[1];

                let ad_payload = ADLPayload::from_str(&command.payload).map_err(|e| MSNPServerError::fatal_error_with_trid(tr_id.to_string(), e.into()))?;
                info!("Add to Contact List received: {:?}", &ad_payload);
                self.msn_client.as_mut().expect("MSN Client to be in context").add_to_contact_list(&ad_payload);
                let matrix_client = self.matrix_client.as_ref().expect("Matrix client to be present at this point");
                if !ad_payload.is_initial() {

                     let user_repo = MSNUserRepository::new(self.matrix_client.as_ref().expect("matrix_client to be here").clone());
                     let new_contacts = ad_payload.get_contacts_for_role(RoleId::Forward);
                     'contacts: for partial_contact in new_contacts {
                    //     let user_id = partial_contact.get_matrix_id();
                    //
                    //     for invited_room in matrix_client.invited_rooms() {
                    //         let is_direct = invited_room.is_direct().await.unwrap_or(false);
                    //         let direct_targets = invited_room.direct_targets();
                    //         if is_direct && direct_targets.len() == 1usize && direct_targets.iter().any(|t| t == &user_id) {
                    //             invited_room.join().await;
                    //             break 'contacts;
                    //         }
                    //
                    //     }
                    //
                    //     if matrix_client.get_dm_room(&user_id).is_none() {
                    //         matrix_client.create_dm(&user_id).await;
                    //     }

                        if let Ok(user) = user_repo.get_msnuser_from_userid(&partial_contact.get_matrix_id(), true).await {
                            NotificationCommandHandler::send_presence_for_user(&user, &self.sender);
                        }
                    }
                }

                return Ok(format!("ADL {tr_id} OK\r\n", tr_id=tr_id));
            },
            "RML" => {
                 /*       0  1  2   payload
                    >>> RML 6 68 <ml l="1"><d n="matrix.org"><c n="u.user" l="3" t="1"/></d></ml>
                    <<< RML 6 OK
                */
                let tr_id = split[1];
                let ad_payload = ADLPayload::from_str(&command.payload).map_err(|e| MSNPServerError::fatal_error_with_trid(tr_id.to_string(), e.into()))?;
                self.msn_client.as_mut().expect("MSN Client to be in context").remove_from_contact_list(&ad_payload);
                let matrix_client = self.matrix_client.as_ref().expect("Matrix Client to be here when RML");

                for contact in ad_payload.get_contacts_for_role(RoleId::Forward){
                    if let Some(room) = matrix_client.get_dm_room(&contact.get_matrix_id()){
                        room.leave().await;
                    }
                }

                return Ok(format!("RML {tr_id} OK\r\n", tr_id=tr_id));
            },
            "UUX" => {
                /*       0  1  2
                    >>> UUX 8 130 payload
                    <<< UUX 8 0
                */
                let tr_id = split[1];
                let payload = &command.payload;
                if payload.starts_with("<PrivateEndpointData>") {
                    let matrix_client = self.matrix_client.as_ref().expect("Matrix Client to be in context");
                    let private_endpoint_data = PrivateEndpointData::from_str(payload).map_err(|e| MSNPServerError::fatal_error_with_trid(tr_id.to_string(), e.into()))?;
                    Self::handle_device_name_update(&private_endpoint_data, matrix_client).await.map_err(|e| MSNPServerError::from_source_with_trid(tr_id.to_string(), e.into()))?;
                } else {
                    warn!("Unknown UXX payload received: {:?}", &command)
                }
                return Ok(format!("UUX {tr_id} 0\r\n", tr_id=tr_id));
            },
            "BLP" => {
                /*  
                    >>> BLP 9 AL
                    <<< BLP 9 AL
                */
                return Ok(format!("{}\r\n", command.command));
            },
            "CHG" => {
                let tr_id = split[1];

                let status = PresenceStatus::from_str(split[2]).unwrap_or(PresenceStatus::NLN);
                let capabilities = split[3];
                let avatar = split[4];

                {
                    let mut me = self.msn_client.as_mut().expect("MSN Client to be in client context").get_user_mut();
                    me.set_status(status.clone());

                    if avatar != "0" {
                        let avatar_decoded = decode_url(avatar).map_err(|e| MSNPServerError::from_source_with_trid(tr_id.to_string(), e.into()))?;
                        let msn_obj = MSNObject::from_str(avatar_decoded.as_str()).map_err(|e| MSNPServerError::from_source_with_trid(tr_id.to_string(), e.into()))?;
                        me.set_display_picture(Some(msn_obj));
                    } else {
                        me.set_display_picture(None);
                    }
                }
                if let Some(matrix_client) = self.matrix_client.as_ref() {
                    //matrix_client.account().set_presence(status.clone().into(), None).await;
                }

                if self.needs_initial_presence {
                    self.needs_initial_presence = false;

                    let all_contacts = self.msn_client.as_ref().expect("MSN Client to be present").get_contacts(true).await;
                   let contacts_chunks: Vec<&[MSNUser]> = all_contacts.chunks(5).collect();
                    for contacts_chunk in contacts_chunks {

                        let mut iln = String::new();
                        let mut ubx = String::new();

                        for contact in contacts_chunk {
                            let mut status = contact.get_status();

                            if contact.get_status() == PresenceStatus::FLN {
                               status = PresenceStatus::HDN;
                            }
                            let mut msn_obj = String::new();


                            if let Some(display_pic) = contact.get_display_picture().as_ref() {
                                msn_obj = display_pic.to_string();
                            }

                            let current_iln = format!("ILN {tr_id} {status} 1:{msn_addr} {nickname} {client_capabilities} {msn_obj}\r\n", tr_id = tr_id, client_capabilities= &contact.get_capabilities() ,msn_addr= &contact.get_msn_addr(), status = status.to_string(), nickname= &contact.get_display_name(), msn_obj = &msn_obj);
                            iln.push_str(current_iln.as_str());

                            let current_ubx_payload = format!("<PSM>{status_msg}</PSM><CurrentMedia></CurrentMedia><EndpointData id=\"{{{machine_guid}}}\"><Capabilities>{client_capabilities}</Capabilities></EndpointData>", status_msg = &contact.get_psm(), client_capabilities= &contact.get_capabilities(), machine_guid = &contact.get_endpoint_guid());
                            let current_ubx = format!("UBX 1:{msn_addr} {ubx_payload_size}\r\n{ubx_payload}", msn_addr = &contact.get_msn_addr(), ubx_payload_size= current_ubx_payload.len(), ubx_payload=current_ubx_payload);
                            ubx.push_str(current_ubx.as_str());
                        }


                        let _result = self.sender.send(iln);
                        let _result = self.sender.send(ubx);
                    }

                }

                // >>> CHG 11 NLN 2789003324:48 0
                // >>> CHG 11 NLN 2789003324:48 %3Cmsnobj%20Creator%3D%22aeontest1%40shlasouf.local%22%20Type%3D%223%22%20SHA1D%3D%22Cqe%2FwD9gdClugwA%2BMGKwVgVD7BI%3D%22%20Size%3D%2227100%22%20Location%3D%220%22%20Friendly%3D%22QQBlAG8AbgBUAGUAcwB0ADEAAAA%3D%22%2F%3E
                // <<< CHG 11 NLN 2789003324:48 0
                return Ok(format!("{}\r\n", command.command));
            },
            "PRP" => {
                // >>> PRP 13 MFN display%20name
                // <<< PRP 13 MFN display%20name

                let res_type = split[2];

                if res_type == "MFN" {
                    let display_name = split[3];
                    {
                        let mut me = self.msn_client.as_mut().expect("MSN Client to be in client context").get_user_mut();
                        me.set_display_name(display_name.to_string());
                    }
                }
                return Ok(format!("{}\r\n", command.command));
            },
            "UUN" => {
                // >>> UUN 14 aeoncl@matrix.org;{0ab73364-6ccf-507b-bb66-a967fe281cd0} 4 14 | goawyplzthxbye
                // UUN 29 aeontest1@shlasouf.local 7 26 | aeontest2@shlasouf.local 1 | closure of conversation
                // UUN 15 aeontest1@shlasouf.local 6 46 | <State><Service type="ab" reason="1"/></State> ?? What is dis
                // UUN 24 aeontest1@shlasouf.local 6 85 | <State><Service type="ab" reason="1"/><Service type="membership" reason="1"/></State>
                // UUN 22 aeontest1@shlasouf.local 5 20 | 31758;127.0.0.1:1864

                // <<< UUN 14 OK
                let tr_id = split[1];
                let receiver = split[2].to_string();
                let receiver_split : Vec<&str> = receiver.split(';').collect();
                let receiver_msn_addr = receiver_split.get(0).unwrap_or(&receiver.as_str()).to_string();
                let endpoint_guid = self.parse_endpoint_guid(receiver_split.get(1));

                let notification_type = split[3];
                let notification_type_parsed: UserNotificationType = num::FromPrimitive::from_i32(i32::from_str(notification_type).unwrap()).unwrap();


                if receiver_msn_addr == self.msn_addr {
                    //this for me
                    if command.payload.as_str() == "goawyplzthxbye" {
                        self.handle_device_logout(endpoint_guid).await;
                    } else if command.payload.as_str() == "gtfo" {
                        //TODO
                    }

                } else {
                    // this not for me
                    if command.payload.contains("MSNSLP/1.0") {
                        //slp payload
                        let slp_request = SlpPayload::from_str(command.payload.as_str()).map_err(|e| MSNPServerError::from_source_with_trid(tr_id.to_string(), e.into()))?;
                        let slp_response = SlpPayloadHandler::handle(&slp_request).map_err(|e| MSNPServerError::from_source_with_trid(tr_id.to_string(), e.into()))?;

                        let payload = slp_response.to_string();
                        return Ok(format!("UUN {tr_id} OK\r\nUBN {msn_addr} {notification_type} {payload_size}\r\n{payload}", tr_id = tr_id, msn_addr= &receiver_msn_addr, notification_type=&notification_type, payload=&payload, payload_size = payload.len()));
                    } else {
                        warn!("Unhandled UUN command payload: {:?}", &command);
                    }

                    return Ok(format!("UUN {tr_id} OK\r\n", tr_id = tr_id));                
                }

                let payload = command.payload.as_str();
                //return format!("UUN {tr_id} OK\r\nUBN {msn_addr} 5 {payload_size}\r\n{payload}", tr_id = tr_id, msn_addr= &receiver_msn_addr, payload=&payload, payload_size = payload.len());
                return Ok(format!("UUN {tr_id} OK\r\n", tr_id = tr_id));
            },
            "XFR" => {
                // >>> XFR 17 SB
                // <<< XFR 17 SB 127.0.0.1:1864 CKI token
                let tr_id = split[1];
                let request_type = split[2];
                if request_type == "SB" {
                    return Ok(format!("XFR {tr_id} {req_type} 127.0.0.1:1864 CKI {token}\r\n", 
                        tr_id = tr_id,
                        req_type = request_type, 
                        token = &self.matrix_token));
                }
                return Ok(format!("{error_code} {tr_id}\r\n", error_code=MSNPErrorCode::InternalServerError as i32, tr_id=tr_id));
            },
            _ => {
                return Ok(String::new());
            }
        }
    }

    fn get_matrix_token(&self) -> String {
        return self.matrix_token.clone();
    }
}

impl Drop for NotificationCommandHandler {
    fn drop(&mut self) {
        //Clean shared data structures
        let token = &self.get_matrix_token();
        if !token.is_empty()  {
            MATRIX_CLIENT_LOCATOR.remove();
            MSN_CLIENT_LOCATOR.remove();
        }

        if let Some(killer) = self.matrix_loop_killer.take() {
            killer.send(());
        }
    }
}

impl NotificationCommandHandler {

    async fn handle_device_name_update(private_endpoint_data: &PrivateEndpointData, matrix_client: &Client) -> Result<(), MatrixError> {
        let device_id = matrix_client.device_id().expect("Matrix Client to have a device id");
        matrix_client.rename_device(device_id, &private_endpoint_data.ep_name).await?;
        Ok(())
    }

    async fn handle_device_logout(&self, endpoint_guid : String) {

        let matrix_client =  MATRIX_CLIENT_LOCATOR.get().unwrap();

        let devices = matrix_client.devices().await.unwrap().devices;
        for device in devices {
            let current_endpoint_guid = UUID::from_string(&device.device_id.to_string()).to_string();
            if current_endpoint_guid == endpoint_guid {
                let result = matrix_client.delete_devices(&[device.device_id], None).await;
                //TODO handle user credential input. (Maybe via opening a web page in browser or in msn using COM object call)
            }

        }
    }

    // async fn handle_all_devices_logout(&self) {
    //     let matrix_client = MATRIX_CLIENT_LOCATOR.get().unwrap();

    //     let devices = matrix_client.devices().await.unwrap().devices;

    //     let mut devices_ids : [OwnedDeviceId];
    //     for device in devices {
    //         let current_endpoint_guid = UUID::from_string(&device.device_id.to_string()).to_string();
    //             let result = matrix_client.delete_devices(&[device.device_id], None).await;
    //             //TODO handle user credential input. (Maybe via opening a web page in browser or in msn using COM object call)
    //     }
    // }


    pub fn send_presence_for_user(user: &MSNUser, sender: &UnboundedSender<String>) {
        let mut msn_obj = String::new();
        if let Some(display_pic) = user.get_display_picture().as_ref() {
            msn_obj = display_pic.to_string();
        }

        let _result = sender.send(format!("NLN {status} 1:{msn_addr} {nickname} {client_capabilities} {msn_obj}\r\n", client_capabilities= &user.get_capabilities() ,msn_addr= &user.get_msn_addr(), status = &user.get_status().to_string(), nickname= &user.get_display_name(), msn_obj = &msn_obj));
        //msn_ns_sender.send(format!("NLN {status} 1:{msn_addr} {nickname} 2788999228:48 {msn_obj}\r\n", msn_addr= &sender_msn_addr, status = presence_status.to_string(), nickname= test3, msn_obj = msn_obj));

        let ubx_payload = format!("<PSM>{status_msg}</PSM><CurrentMedia></CurrentMedia><EndpointData id=\"{{{machine_guid}}}\"><Capabilities>{client_capabilities}</Capabilities></EndpointData>", status_msg = &user.get_psm(), client_capabilities= &user.get_capabilities(), machine_guid = &user.get_endpoint_guid());
        //let ubx_payload = format!("<PSM>{status_msg}</PSM><CurrentMedia></CurrentMedia>", status_msg = ev.content.status_msg.unwrap_or(String::new()));
        let _result = sender.send(format!("UBX 1:{msn_addr} {ubx_payload_size}\r\n{ubx_payload}", msn_addr = &user.get_msn_addr(), ubx_payload_size= ubx_payload.len(), ubx_payload=ubx_payload));
    }

    fn parse_endpoint_guid(&self, maybe_endpoint_guid: Option<&&str>) -> String {

        if let Some(mut endpoind_guid) = maybe_endpoint_guid {
            return endpoind_guid.to_string().substring(1, endpoind_guid.len()-1).to_string()
        }
        return String::new();
    }

}

