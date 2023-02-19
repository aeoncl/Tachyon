use std::path::Path;
use std::str::{FromStr};

use log::info;
use matrix_sdk::Client;
use substring::Substring;
use async_trait::async_trait;
use tokio::sync::broadcast::{Sender, self, Receiver};
use crate::generated::payloads::{PrivateEndpointData, PresenceStatus};

use crate::models::ab_data::AbData;
use crate::models::msn_user::MSNUser;
use crate::models::notification::error::{MsnpError, MsnpErrorCode};
use crate::models::notification::events::notification_event::NotificationEvent;
use crate::models::notification::msn_client::{MSNClient, self};
use crate::models::p2p::slp_payload::SlpPayload;
use crate::models::p2p::slp_payload_handler::SlpPayloadHandler;
use crate::models::wlmatrix_client::WLMatrixClient;
use crate::repositories::repository::Repository;
use crate::{AB_DATA_REPO, MSN_CLIENT_LOCATOR, MATRIX_CLIENT_LOCATOR};
use crate::models::uuid::UUID;
use crate::models::msg_payload::factories::{MsgPayloadFactory};
use super::command_handler::CommandHandler;
use super::msnp_command::MSNPCommand;

pub struct NotificationCommandHandler {
    matrix_token: String,
    msn_addr: String,
    sender: Sender<String>,
    msnp_version: i16,
    msn_client: Option<MSNClient>,
    matrix_client: Option<Client>,
    wlmatrix_client: Option<WLMatrixClient>,

}

impl NotificationCommandHandler {

    fn start_receiving(&mut self, mut notification_receiver: Receiver<NotificationEvent>) {

        let sender = self.sender.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    command_to_send = notification_receiver.recv() => {
                        if let Ok(msg) = command_to_send {
                            match msg {
                                NotificationEvent::HotmailNotificationEvent(content) => {
                                    let _result = sender.send(format!("NOT {payload_size}\r\n{payload}", payload_size = content.payload.len(), payload = content.payload));
                                },
                                NotificationEvent::DisconnectEvent(content) => {
                                    let _result = sender.send(format!("FLN 1:{msn_addr}\r\n", msn_addr = content.msn_addr));
                                },
                                NotificationEvent::PresenceEvent(content) => {
                                    let user = &content.user;
                                    let _result = sender.send(format!("NLN {status} 1:{msn_addr} {nickname} {client_capabilities} {msn_obj}\r\n", client_capabilities= &user.get_capabilities() ,msn_addr= &user.get_msn_addr(), status = &user.get_status().to_string(), nickname= &user.get_display_name(), msn_obj = ""));
                                    //msn_ns_sender.send(format!("NLN {status} 1:{msn_addr} {nickname} 2788999228:48 {msn_obj}\r\n", msn_addr= &sender_msn_addr, status = presence_status.to_string(), nickname= test3, msn_obj = msn_obj));
                            
                                    let ubx_payload = format!("<PSM>{status_msg}</PSM><CurrentMedia></CurrentMedia><EndpointData id=\"{{{machine_guid}}}\"><Capabilities>{client_capabilities}</Capabilities></EndpointData>", status_msg = &user.get_psm(), client_capabilities= &user.get_capabilities(), machine_guid = &user.get_endpoint_guid());
                                    //let ubx_payload = format!("<PSM>{status_msg}</PSM><CurrentMedia></CurrentMedia>", status_msg = ev.content.status_msg.unwrap_or(String::new()));
                                    let _result = sender.send(format!("UBX 1:{msn_addr} {ubx_payload_size}\r\n{ubx_payload}", msn_addr = &user.get_msn_addr(), ubx_payload_size= ubx_payload.len(), ubx_payload=ubx_payload));
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
                            info!("bad message received in notif command handler -> exitting.");
                            break;
                        }
                      
                    }
                }
            }
        });
    }
    


    pub fn new(sender: Sender<String>) -> NotificationCommandHandler {
        return NotificationCommandHandler {
            sender: sender,
            matrix_token: String::new(),
            msn_client: None,
            matrix_client: None,
            msnp_version: -1,
            msn_addr: String::new(),
            wlmatrix_client: None,
        };
    }
}




#[async_trait]
impl CommandHandler for NotificationCommandHandler {

    async fn handle_command(&mut self, command: &MSNPCommand) -> Result<String, MsnpError> {
        let split = command.split();
        match command.operand.as_str() {
            "VER" => {
                // 0  1    2      3     4
                //=>VER 1 MSNP18 MSNP17 CVR0\r\n
                //<=VER 1 MSNP18
                let ver: i16 = split[2]
                    .substring(4, split[2].chars().count())
                    .parse::<i16>()
                    .unwrap();

                self.msnp_version = ver;
                //<=VER 1 MSNP18\r\n
                return Ok(format!("VER {} MSNP{}\r\n", split[1], ver).to_string());
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
                        0   1  2  3     4                    5
                    >>> USR 4 SSO S t=ssotoken {55192CF5-588E-4ABE-9CDF-395B616ED85B}
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

                        let msn_user = MSNUser::new(self.msn_addr.clone());

                        let matrix_client = WLMatrixClient::login(msn_user.get_matrix_id(), self.matrix_token.clone(), &Path::new("C:\\temp")).await.or(Err(MsnpError::auth_fail(&tr_id)))?;

                        let mut msn_client = MSNClient::new(matrix_client.clone(), msn_user.clone(), self.msnp_version);

                        //Token valid, client authenticated. Initializing shared data structures
                        self.msn_client = Some(msn_client.clone());
                        MATRIX_CLIENT_LOCATOR.set(matrix_client.clone());
                        MSN_CLIENT_LOCATOR.set(msn_client.clone());
                        AB_DATA_REPO.add(self.matrix_token.clone(), AbData::new());

                        self.sender.send(format!("USR {tr_id} OK {email} 1 0\r\nSBS 0 null\r\n", tr_id = &tr_id, email=&self.msn_addr));

                        let msmsgs_profile_msg = MsgPayloadFactory::get_msmsgs_profile(&msn_client.get_user().get_puid(), self.msn_addr.clone(), self.matrix_token.clone()).serialize();
  
                        self.sender.send(format!("MSG Hotmail Hotmail {profile_payload_size}\r\n{profile_payload}", profile_payload_size=msmsgs_profile_msg.len(), profile_payload=&msmsgs_profile_msg));

                        let oim_payload = MsgPayloadFactory::get_initial_mail_data_notification().serialize();
                          
                        self.sender.send(format!("MSG Hotmail Hotmail {oim_payload_size}\r\n{oim_payload}", oim_payload_size=oim_payload.len(), oim_payload=&oim_payload));

                        match msn_client.get_mpop_endpoints().await {
                            Ok(mpop_endpoints) => {
                                let mut endpoints = String::new();
                                for endpoint in mpop_endpoints {
                                    endpoints.push_str(endpoint.to_string().as_str());
                                }
                               let endpoints_payload = format!("<Data>{endpoints}</Data>", endpoints = endpoints);
                               self.sender.send(format!("UBX 1:{email} {private_endpoint_payload_size}\r\n{private_endpoint_payload}", email=self.msn_addr, private_endpoint_payload_size= endpoints_payload.len(), private_endpoint_payload=&endpoints_payload));
                            }, 
                            Err(err_code) => {
                                return Err(MsnpError::new(err_code, tr_id.to_string()));
                            }
                        }

                        //   let test_msg = MsgPayloadFactory::get_system_msg(String::from("1"), String::from("17"));
                        //   let serialized = test_msg.serialize();
                        //   self.sender.send(format!("MSG Hotmail Hotmail {payload_size}\r\n{payload}", payload_size=serialized.len(), payload=&serialized));

                        let (notification_sender, notification_receiver) = broadcast::channel::<NotificationEvent>(30);
                        self.start_receiving(notification_receiver);

                        let wlmatrix_client = WLMatrixClient::listen(matrix_client.clone(), msn_user.clone(), notification_sender).await.unwrap();
                        self.wlmatrix_client.insert(wlmatrix_client);
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
                return Ok(format!("ADL {tr_id} OK\r\n", tr_id=tr_id));
            },
            "RML" => {
                 /*       0  1  2   payload
                    >>> RML 6 68 <ml l="1"><d n="matrix.org"><c n="u.user" l="3" t="1"/></d></ml>
                    <<< RML 6 OK
                */
                let tr_id = split[1];
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
                    self.handle_device_name_update(payload.as_str()).await;
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

                self.msn_client.as_mut().ok_or(MsnpError::internal_server_error(tr_id))?.get_user_mut().set_status(status.clone());

                if let Some(matrix_client) = self.matrix_client.as_ref() {
                    matrix_client.account().set_presence(status.clone().into(), None).await;
                }

                // >>> CHG 11 NLN 2789003324:48 0
                // <<< CHG 11 NLN 2789003324:48 0
                return Ok(format!("{}\r\n", command.command));
            },
            "PRP" => {
                // >>> PRP 13 MFN display%20name
                // <<< PRP 13 MFN display%20name

                return Ok(format!("{}\r\n", command.command));
            },
            "UUN" => {
                // >>> UUN 14 aeoncl@matrix.org;{0ab73364-6ccf-507b-bb66-a967fe281cd0} 4 14 | goawyplzthxbye
                // <<< UUN 14 OK
                let tr_id = split[1];
                let receiver = split[2].to_string();
                let receiver_split : Vec<&str> = receiver.split(';').collect();
                let receiver_msn_addr = receiver_split.get(0).unwrap_or(&receiver.as_str()).to_string();
                let endpoint_guid = self.parse_endpoint_guid(receiver_split.get(1));

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
                        let slp_request = SlpPayload::from_str(command.payload.as_str()).unwrap();
                        let slp_response = SlpPayloadHandler::handle(&slp_request).unwrap();

                        let payload = slp_response.to_string();
                        return Ok(format!("UUN {tr_id} OK\r\nUBN {msn_addr} 5 {payload_size}\r\n{payload}", tr_id = tr_id, msn_addr= &receiver_msn_addr, payload=&payload, payload_size = payload.len()));
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
                return Ok(format!("{error_code} {tr_id}\r\n", error_code=MsnpErrorCode::InternalServerError as i32, tr_id=tr_id));
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
            AB_DATA_REPO.remove(token);
        }
    }
}

impl NotificationCommandHandler {

    async fn handle_device_name_update(&self, payload: &str) {

        if let Ok(private_endpoint_data) = PrivateEndpointData::from_str(payload) {
            if let Some(matrix_client) = MATRIX_CLIENT_LOCATOR.get() {

                 let device_id = matrix_client.device_id().unwrap();
                 matrix_client.update_device(device_id.to_owned(), private_endpoint_data.ep_name).await.unwrap_or_default();

            }
        }

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

    fn parse_endpoint_guid(&self, maybe_endpoint_guid: Option<&&str>) -> String{

        if let Some(mut endpoind_guid) = maybe_endpoint_guid {
            return endpoind_guid.to_string().substring(1, endpoind_guid.len()-1).to_string()
        }
        return String::new();
    }

}

