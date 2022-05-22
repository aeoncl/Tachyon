use std::path::PathBuf;
use std::str::{FromStr, from_utf8_unchecked};
use std::sync::Arc;
use matrix_sdk::ruma::{RoomId, OwnedRoomId};
use matrix_sdk::ruma::exports::ruma_macros::room_id;
use rand::Rng;
use tokio::task::JoinHandle;
use log::info;
use matrix_sdk::Client;
use matrix_sdk::room::Room;
use matrix_sdk::ruma::events::room::message::{SyncRoomMessageEvent, RoomMessageEventContent};
use substring::Substring;
use async_trait::async_trait;
use tokio::sync::broadcast::{Sender, Receiver, self};
use crate::generated::payloads::{PrivateEndpointData, PresenceStatus};
use crate::models::ab_data::AbData;
use crate::models::errors::{MsnpErrorCode};
use crate::models::msg_payload::MsgPayload;
use crate::models::msn_user::MSNUser;
use crate::models::p2p::pending_packet::PendingPacket;
use crate::models::p2p::proxy_client::ProxyClient;
use crate::models::slp_payload::{P2PTransportPacket, P2PPayload};
use crate::models::slp_payload::factories::{SlpPayloadFactory, P2PTransportPacketFactory};
use crate::models::switchboard_handle::SwitchboardHandle;
use crate::repositories::matrix_client_repository::MatrixClientRepository;
use crate::repositories::repository::Repository;
use crate::utils::identifiers::{msn_addr_to_matrix_id, matrix_id_to_msn_addr, msn_addr_to_matrix_user_id, matrix_room_id_to_annoying_matrix_room_id};
use crate::utils::matrix;
use crate::{CLIENT_DATA_REPO, MATRIX_CLIENT_REPO, AB_DATA_REPO};
use crate::models::client_data::ClientData;
use crate::models::uuid::UUID;
use crate::repositories::client_data_repository::{ClientDataRepository};
use crate::models::msg_payload::factories::{MsgPayloadFactory};
use super::msnp_command::MSNPCommand;
use crate::utils::matrix_sync_helpers::*;
use std::mem;
pub struct NotificationCommandHandler {
    protocol_version: i16,
    msn_addr: String,
    matrix_token: String,
    sender: Sender<String>,
    kill_sender: Option<Sender<String>>
}

#[async_trait]
pub trait CommandHandler : Send {
    async fn handle_command(&mut self, command: &MSNPCommand) -> String;

    fn get_matrix_token(&self) -> String;

    fn cleanup(&self);
}

impl NotificationCommandHandler {
    pub fn new(sender: Sender<String>) -> NotificationCommandHandler {
        return NotificationCommandHandler {
            sender: sender,
            protocol_version: -1,
            msn_addr: String::new(),
            matrix_token: String::new(),
            kill_sender: None
        };
    }
}

#[async_trait]
impl CommandHandler for NotificationCommandHandler {
    async fn handle_command(&mut self, command: &MSNPCommand) -> String {
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
                self.protocol_version = ver;
                //<=VER 1 MSNP18\r\n
                return format!("VER {} MSNP{}\r\n", split[1], ver).to_string();
            }
            "CVR" => {
                //    0  1    2     3     4    5      6          7          8          9
                //=> CVR 2 0x0409 winnt 6.0.0 i386 MSNMSGR 14.0.8117.0416 msmsgs login@email.com
                let _msn_login = split[9];
                let tr_id = split[1];
                let version = split[7];
                //<= CVR 2 14.0.8117.0416 14.0.8117.0416 14.0.8117.0416 localhost localhost
                return format!(
                    "CVR {tr_id} {version} {version} {version} {host} {host}\r\n",
                    tr_id = tr_id,
                    version = version,
                    host = "localhost"
                );
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
                    return format!("USR {tr_id} OK {email} 1 0\r\n", tr_id=tr_id, email=self.msn_addr);
                } else if auth_type == "SSO" {
                    if phase == "I" {
                        let login = split[4];
                        self.msn_addr = login.to_string();
                        let shields_payload = "<Policies><Policy type= \"SHIELDS\"><config><shield><cli maj= \"7\" min= \"0\" minbld= \"0\" maxbld= \"9999\" deny= \" \" /></shield><block></block></config></Policy><Policy type= \"ABCH\"><policy><set id= \"push\" service= \"ABCH\" priority= \"200\"><r id= \"pushstorage\" threshold= \"0\" /></set><set id= \"using_notifications\" service= \"ABCH\" priority= \"100\"><r id= \"pullab\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /><r id= \"pullmembership\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /></set><set id= \"delaysup\" service= \"ABCH\" priority= \"150\"><r id= \"whatsnew\" threshold= \"0\" /><r id= \"whatsnew_storage_ABCH_delay\" timer= \"1800000\" /><r id= \"whatsnewt_link\" threshold= \"0\" trigger= \"QueryActivities\" /></set><c id= \"PROFILE_Rampup\">100</c></policy></Policy><Policy type= \"ERRORRESPONSETABLE\"><Policy><Feature type= \"3\" name= \"P2P\"><Entry hr= \"0x81000398\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /></Feature><Feature type= \"4\"><Entry hr= \"0x81000440\" /></Feature><Feature type= \"6\" name= \"TURN\"><Entry hr= \"0x8007274C\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /><Entry hr= \"0x8007274A\" action= \"3\" /></Feature></Policy></Policy><Policy type= \"P2P\"><ObjStr SndDly= \"1\" /></Policy></Policies>";
                        return format!("USR {tr_id} SSO S MBI_KEY_OLD LAhAAUzdC+JvuB33nooLSa6Oh0oDFCbKrN57EVTY0Dmca8Reb3C1S1czlP12N8VU\r\nGCF 0 {shields_size}\r\n{shields_payload}", tr_id = tr_id, shields_payload = shields_payload, shields_size = shields_payload.len());
                    } else if phase == "S" {
                        self.matrix_token = split[4][2..split[4].chars().count()].to_string();
                        let matrix_id = msn_addr_to_matrix_id(&self.msn_addr);

                        if let Ok(client) = matrix::login(matrix_id.clone(), self.matrix_token.clone()).await {
    
                            //Token valid, client authenticated.
                            let client_data_repo : Arc<ClientDataRepository> = CLIENT_DATA_REPO.clone();
                            client_data_repo.add(self.matrix_token.clone(), ClientData::new(self.msn_addr.clone(), self.protocol_version.clone(), split[5].to_string(), PresenceStatus::HDN));

    
                            let msmsgs_profile_msg = MsgPayloadFactory::get_msmsgs_profile(UUID::from_string(&matrix_id).get_puid(), self.msn_addr.clone(), self.matrix_token.clone()).serialize();
    
                            let oim_payload = MsgPayloadFactory::get_initial_mail_data_notification().serialize();
                            let oim_payload_size = oim_payload.len();
                            
                            let devices = client.devices().await.unwrap().devices;
                            

                            let mut endpoints = String::new();
                            let this_device_id = client.device_id().await.unwrap();

                            for device in devices {
                                if device.device_id != this_device_id {
                                    let machine_guid = UUID::from_string(&device.device_id.to_string());
                                    let endpoint_name = device.display_name.unwrap_or(device.device_id.to_string());
                                    let private_endpoint = format!("<EndpointData id=\"{{{machine_guid}}}\"><Capabilities>2789003324:48</Capabilities></EndpointData><PrivateEndpointData id=\"{{{machine_guid}}}\"><EpName>{endpoint_name}</EpName><Idle>false</Idle><ClientType>1</ClientType><State>NLN</State></PrivateEndpointData>", machine_guid = machine_guid.to_string(), endpoint_name = endpoint_name);
                                    endpoints.push_str(private_endpoint.as_str());
                                }
                            }

                            let private_endpoint_test = format!("<Data>{endpoints}</Data>", endpoints = endpoints);


                            let test_msg = MsgPayloadFactory::get_system_msg(String::from("1"), String::from("17"));
                            let serialized = test_msg.serialize();
                            self.sender.send(format!("MSG Hotmail Hotmail {payload_size}\r\n{payload}", payload_size=serialized.len(), payload=&serialized));

                            let matrix_client_repo : Arc<MatrixClientRepository> = MATRIX_CLIENT_REPO.clone();
                            matrix_client_repo.add(self.matrix_token.clone(), client);

                            let ab_data_repo  = AB_DATA_REPO.clone();
                            ab_data_repo.add(self.matrix_token.clone(), AbData::new());

                            self.kill_sender = Some(start_matrix_loop(self.matrix_token.clone(), self.msn_addr.clone(), self.sender.clone()).await);

                            return format!("USR {tr_id} OK {email} 1 0\r\nSBS 0 null\r\nMSG Hotmail Hotmail {msmsgs_profile_payload_size}\r\n{payload}MSG Hotmail Hotmail {oim_payload_size}\r\n{oim_payload}UBX 1:{email} {private_endpoint_payload_size}\r\n{private_endpoint_payload}", tr_id = tr_id, email=&self.msn_addr, msmsgs_profile_payload_size= msmsgs_profile_msg.len(), payload=msmsgs_profile_msg, oim_payload = oim_payload, oim_payload_size = oim_payload_size, private_endpoint_payload_size = private_endpoint_test.len(), private_endpoint_payload = private_endpoint_test);
                        } else {
                            //Invalid token. Auth failure.
                            return format!("{error_code} {tr_id}\r\n", error_code = MsnpErrorCode::AuthFail as i32, tr_id= tr_id);
                        }

                    
                    }
                }

                return String::new();
            },
            "PNG" => {
                return String::from("QNG 60\r\n");
            },
            "ADL" => {
                /*       0  1  2   payload
                    >>> ADL 6 68 <ml l="1"><d n="matrix.org"><c n="u.user" l="3" t="1"/></d></ml>
                    <<< ADL 6 OK
                */
                let tr_id = split[1];
                return format!("ADL {tr_id} OK\r\n", tr_id=tr_id);
            },
            "RML" => {
                 /*       0  1  2   payload
                    >>> RML 6 68 <ml l="1"><d n="matrix.org"><c n="u.user" l="3" t="1"/></d></ml>
                    <<< RML 6 OK
                */
                let tr_id = split[1];
                return format!("RML {tr_id} OK\r\n", tr_id=tr_id);
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
                return format!("UUX {tr_id} 0\r\n", tr_id=tr_id);
            },
            "BLP" => {
                /*  
                    >>> BLP 9 AL
                    <<< BLP 9 AL
                */
                return format!("{}\r\n", command.command);
            },
            "CHG" => {
                let status = PresenceStatus::from_str(split[2]).unwrap_or(PresenceStatus::NLN);

                if let Some(matrix_client) = MATRIX_CLIENT_REPO.clone().find(&self.matrix_token){
                    matrix_client.account().set_presence(status.clone().into(), None).await;
                }

                let client_data_repo : Arc<ClientDataRepository> = CLIENT_DATA_REPO.clone();

                if let Some(mut client_data) = client_data_repo.find_mut(&self.matrix_token) {
                    client_data.presence_status = status;
                }


                // >>> CHG 11 NLN 2789003324:48 0
                // <<< CHG 11 NLN 2789003324:48 0
                return format!("{}\r\n", command.command);
            },
            "PRP" => {
                // >>> PRP 13 MFN display%20name
                // <<< PRP 13 MFN display%20name

                return format!("{}\r\n", command.command);
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
                    } else {
                        return format!("{error_code} {tr_id}\r\n", error_code = MsnpErrorCode::PrincipalNotOnline as i32, tr_id= tr_id);
                    }
                } else {
                    // this not for me
                    return format!("{error_code} {tr_id}\r\n", error_code = MsnpErrorCode::PrincipalNotOnline as i32, tr_id= tr_id);                
                }


                let payload = command.payload.as_str();
                //return format!("UUN {tr_id} OK\r\nUBN {msn_addr} 5 {payload_size}\r\n{payload}", tr_id = tr_id, msn_addr= &receiver_msn_addr, payload=&payload, payload_size = payload.len());
                return format!("UUN {tr_id} OK\r\n", tr_id = tr_id);
            },
            "XFR" => {
                // >>> XFR 17 SB
                // <<< XFR 17 SB 127.0.0.1:1864 CKI token
                let tr_id = split[1];
                let request_type = split[2];
                if request_type == "SB" {
                    return format!("XFR {tr_id} {req_type} 127.0.0.1:1864 CKI {token}\r\n", 
                        tr_id = tr_id,
                        req_type = request_type, 
                        token = &self.matrix_token);
                }
                return format!("{error_code} {tr_id}\r\n", error_code=MsnpErrorCode::InternalServerError as i32, tr_id=tr_id);
            },
            _ => {
                return String::new();
            }
        }
    }

    fn get_matrix_token(&self) -> String {
        return self.matrix_token.clone();
    }

    fn cleanup(&self) {
        if let Some(kill_sender) = &self.kill_sender {
            kill_sender.send(String::from("STOP"));
        }

        let token = &self.get_matrix_token();
        if(!token.is_empty()) {
            MATRIX_CLIENT_REPO.remove(token);
            CLIENT_DATA_REPO.remove(token);
            AB_DATA_REPO.remove(token);
        }
    }
}

impl NotificationCommandHandler {

    async fn handle_device_name_update(&self, payload: &str) {
        let matrix_client_repo : Arc<MatrixClientRepository> = MATRIX_CLIENT_REPO.clone();

        if let Ok(private_endpoint_data) = PrivateEndpointData::from_str(payload) {
            if let Some(matrix_client) = matrix_client_repo.find(&self.matrix_token) {

                 let device_id = matrix_client.device_id().await.unwrap();
                 matrix_client.update_device(&device_id, private_endpoint_data.ep_name).await.unwrap_or_default();

            }
        }

    }

    async fn handle_device_logout(&self, endpoint_guid : String) {

        let client_repo : Arc<MatrixClientRepository> = MATRIX_CLIENT_REPO.clone();
        let matrix_client = client_repo.find(&self.matrix_token).unwrap();

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
    //     let client_repo : Arc<MatrixClientRepository> = MATRIX_CLIENT_REPO.clone();
    //     let matrix_client = client_repo.find(&self.matrix_token).unwrap();

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


pub struct SwitchboardCommandHandler {
    protocol_version: i16,
    msn_addr: String,
    endpoint_guid: String,
    matrix_token: String,
    target_room_id: String,
    target_matrix_id: String,
    target_msn_addr: String,
    matrix_client: Option<Client>,
    sender: Sender<String>,
    sb_handle: Option<SwitchboardHandle>,
    proxy_client: Option<ProxyClient>
}

impl SwitchboardCommandHandler {
    pub fn new(sender: Sender<String>) -> SwitchboardCommandHandler {
        return SwitchboardCommandHandler {
            protocol_version: -1,
            msn_addr: String::new(),
            endpoint_guid: String::new(),
            matrix_token: String::new(),
            target_room_id: String::new(),
            matrix_client: None,
            target_matrix_id: String::new(),
            target_msn_addr: String::new(),
            sender: sender,
            proxy_client: None,
            sb_handle: None
        };
    }


    fn start_receiving(&mut self, mut sb_handle_receiver: Receiver<String>) {

        let (p2p_sender, mut p2p_receiver) = broadcast::channel::<PendingPacket>(10);
        self.proxy_client = Some(ProxyClient::new(p2p_sender));

        let sender = self.sender.clone();
        tokio::spawn(async move {
                let sender = sender;
                loop {
                    tokio::select! {
                        command_to_send = sb_handle_receiver.recv() => {
                            let msg = command_to_send.unwrap();
                            if msg.starts_with("STOP") {
                                break;
                            } else {
                                let _result = sender.send(msg);
                            }
                        },
                        p2p_packet_to_send_maybe = p2p_receiver.recv() => {
                            if let Ok(p2p_packet_to_send) = p2p_packet_to_send_maybe {
                                let msn_sender = &p2p_packet_to_send.sender;
                                let msn_receiver = &p2p_packet_to_send.receiver;

                                let msg_to_send = MsgPayloadFactory::get_p2p(msn_sender, msn_receiver,  &p2p_packet_to_send.packet);
                                let serialized_response = msg_to_send.serialize();
                                let _result = sender.send(format!("MSG {msn_addr} {msn_addr} {payload_size}\r\n{payload}", msn_addr = &msn_sender.msn_addr, payload_size = serialized_response.len(), payload = &serialized_response));
                            } 
                        }
                    }
                }
            });
    }

    pub async fn send_initial_roster(&mut self, tr_id: &str) {

        let room_id = matrix_room_id_to_annoying_matrix_room_id(&self.target_room_id);
        if let Some(room) = &self.matrix_client.as_ref().unwrap().get_joined_room(&room_id) {
            let members = room.joined_members().await.unwrap();
            let mut index = 1;
            //let count = (members.len() - 1)*2;
            let count = members.len() - 1;

            for member in members {
                let msn_user = MSNUser::from_matrix_id(member.user_id().to_string());
                if msn_user.msn_addr != self.msn_addr {
                    self.send_initial_roster_member(tr_id, index, count as i32, &msn_user);
                    index += 2;
                }
            }
        }
    }

    fn send_initial_roster_member(&self, tr_id: &str, index: i32, count: i32, msn_user: &MSNUser) {
            self.sender.send(format!("IRO {tr_id} {index} {roster_count} {passport} {friendly_name} {capabilities}\r\n",
            tr_id = &tr_id,
            index = &index,
            roster_count = &count,
            passport = &msn_user.msn_addr,
            friendly_name = &msn_user.msn_addr,
            capabilities = &msn_user.capabilities));


            let endpoint_guid = UUID::from_string(&msn_user.msn_addr).to_string().to_uppercase();

            self.sender.send(format!("IRO {tr_id} {index} {roster_count} {passport};{{{endpoint_guid}}} {friendly_name} {capabilities}\r\n",
            tr_id = &tr_id,
            index = &index+1,
            roster_count = &count,
            passport = &msn_user.msn_addr,
            friendly_name = &msn_user.msn_addr,
            endpoint_guid = &endpoint_guid,
            capabilities = &msn_user.capabilities));

    }

    pub fn send_me_joined(&self) {
        let mut me = MSNUser::new(self.msn_addr.clone());
        me.endpoint_guid = self.endpoint_guid.clone();
        self.send_contact_joined(&me);
    }

    pub fn send_contact_joined(&self, user: &MSNUser) {
        self.sender.send(format!("JOI {passport} {friendly_name} {capabilities}\r\n", passport=&user.msn_addr, friendly_name = &user.msn_addr, capabilities = &user.capabilities));
    }



}

#[async_trait]
impl CommandHandler for SwitchboardCommandHandler {

    async fn handle_command(&mut self, command: &MSNPCommand) -> String {
        let split = command.split();
        match command.operand.as_str() {
            "ANS" => {
                // >>> ANS 3 aeontest@shl.local;{F52973B6-C926-4BAD-9BA8-7C1E840E4AB0} base64token 4060759068338340280
                // <<< 
                let token = String::from_utf8(base64::decode(split[3]).unwrap()).unwrap();
                let split_token : Vec<&str> = token.split(";").collect();
                let tr_id = split[1];
                let endpoint = split[2];
                let endpoint_parts : Vec<&str> = endpoint.split(";").collect();

                self.msn_addr = endpoint_parts.get(0).unwrap().to_string();

                let endpoint_guid = endpoint_parts.get(1).unwrap().to_string();
                self.endpoint_guid = endpoint_guid.substring(1, endpoint_guid.len()-1).to_string();
                self.target_room_id = split_token.get(0).unwrap().to_string();
                self.matrix_token = split_token.get(1).unwrap().to_string();
                self.target_matrix_id = split_token.get(2).unwrap().to_string();
                self.target_msn_addr = matrix_id_to_msn_addr(&self.target_matrix_id);
                self.matrix_client = Some(MATRIX_CLIENT_REPO.find(&self.matrix_token).unwrap().clone());


                let client_data = CLIENT_DATA_REPO.find_mut(&self.matrix_token).unwrap();

                if let Some(mut sb_handle) = client_data.switchboards.find(&self.target_room_id){

                    
                        let _result = self.send_initial_roster(&tr_id).await;

                        self.sender.send(format!("ANS {tr_id} OK\r\n", tr_id = &tr_id));
                        self.send_me_joined();

                        let mut receiver = sb_handle.take_receiver().unwrap();
                        self.start_receiving(receiver);

                } 
                return String::new();
            },
            "USR" => {
                // >>> USR 55 aeontest@shl.local;{F52973B6-C926-4BAD-9BA8-7C1E840E4AB0} matrix_token
                // <<< USR 55 aeontest@shl.local aeontest@shl.local OK

                let tr_id = split[1];
                let endpoint_str = split[2].to_string();
                self.matrix_token = split[3].to_string();
                let endpoint_str_split : Vec<&str> = endpoint_str.split(";").collect(); 
                if let Some(msn_addr) = endpoint_str_split.get(0){
                    self.msn_addr = msn_addr.to_string();
                    let endpoint_guid = endpoint_str_split.get(1).unwrap().to_string();
                    self.endpoint_guid = endpoint_guid.substring(1, endpoint_guid.len()-1).to_string();

                    if let Some(client_data) = CLIENT_DATA_REPO.find(&self.matrix_token){
                        if let Some(client) = MATRIX_CLIENT_REPO.find(&self.matrix_token) {
                            self.matrix_client = Some(client.clone());
                            self.protocol_version = client_data.msnp_version;
                            return format!("USR {tr_id} {msn_addr} {msn_addr} OK\r\n", tr_id = tr_id, msn_addr = msn_addr);
                        }
                    }
                }
                return format!("{error_code} {tr_id}\r\n", error_code = MsnpErrorCode::AuthFail as i32, tr_id = tr_id)
            },
            "CAL" => {
                //Calls all the members to join the SB
                // >>> CAL 58 aeontest@shl.local
                // <<< CAL 58 RINGING 4324234

                let tr_id = split[1];
                let msn_addr_to_add = split[2].to_string();
                let session_id = UUID::new().to_decimal_cid_string();

                self.sender.send(format!("CAL {tr_id} RINGING {session_id}\r\n", tr_id = tr_id, session_id = session_id));

                if msn_addr_to_add == self.msn_addr {
                    self.send_me_joined();
                    //self.sender.send(format!("JOI {msn_addr};{{{endpoint_guid}}} {msn_addr} 2788999228:48\r\n", msn_addr= &msn_addr_to_add, endpoint_guid = &self.endpoint_guid));

                    //that's me !
                } else {
                    let user_to_add = msn_addr_to_matrix_user_id(&msn_addr_to_add);


                    let mut client = self.matrix_client.as_ref().unwrap().clone();

                    let target_room = client.find_or_create_dm_room(&user_to_add).await.unwrap().unwrap(); //TODO handle this
                    self.target_room_id = target_room.room_id().to_string();
                    let client_data = CLIENT_DATA_REPO.find_mut(&self.matrix_token).unwrap();

                    let mut sb_data = SwitchboardHandle::new(client.clone(), target_room.room_id().to_owned(), self.msn_addr.clone());
                                        
                    let user_to_add = MSNUser::new(msn_addr_to_add.clone());
                    self.send_contact_joined(&user_to_add);

                    self.start_receiving(sb_data.take_receiver().unwrap());

                    client_data.switchboards.add(target_room.room_id().to_string(), sb_data);
                }

                return String::new();
            },
            "MSG" => {
                //      0   1  2 3     
                // >>> MSG 231 U 91 
                // <<< ACK 231           on success
                // <<< NAK 231          on failure
                // The 2nd parameter is the type of ack the clients wants.
                // N: ack only when the message was not received
                // A: always send an ack
                // U: never ack
                
                if let Ok(payload) = MsgPayload::from_str(command.payload.as_str()){

                    if payload.content_type == "application/x-msnmsgrp2p" {
                        //P2P packets
                       if let Ok(mut p2p_packet) = P2PTransportPacket::from_str(&payload.body){
                            if let Some(proxy_client) = &mut self.proxy_client {

                                let source = MSNUser::from_mpop_addr_string(payload.get_header(&String::from("P2P-Src")).unwrap().to_owned()).unwrap();
                                let dest = MSNUser::from_mpop_addr_string(payload.get_header(&String::from("P2P-Dest")).unwrap().to_owned()).unwrap();
                                proxy_client.on_message_received(PendingPacket::new(p2p_packet, source, dest));

                            } else {
                                info!("P2P: Message received while proxy_client wasn't initialized: {}", &payload.body);

                            }

                           
                        } else {
                            info!("P2P: Transport packet deserialization failed: {}", &payload.body);
    
                        }
                    } else {

                        let client_data = CLIENT_DATA_REPO.find(&self.matrix_token).unwrap();
                        if let Some(mut sb_handle) = client_data.switchboards.find(&self.target_room_id){
                            sb_handle.send_message_to_server(payload).await;
                        }
                    }
                  
                }

                let tr_id = split[1];
                let type_of_ack = split[2];


                if type_of_ack == "A" {
                    return format!("ACK {tr_id}\r\n", tr_id= &tr_id);
                }
                    
                return String::new();

            },
            _=> {
                return String::new();
            }
        }
    }

    fn get_matrix_token(&self) -> String {
        return String::new();
    }

    fn cleanup(&self) {
        if let Some(client_data) = CLIENT_DATA_REPO.find_mut(&self.matrix_token) {
            if let Some(found) = client_data.switchboards.find(&self.target_room_id){
                found.stop();
            }
            client_data.switchboards.remove(&self.target_room_id);
        }
    }


    

}

#[cfg(test)]
mod tests {
    use tokio::sync::broadcast::{Sender, self};

    use crate::sockets::msnp_command::MSNPCommandParser;
    use crate::sockets::msnp_command_handlers::{CommandHandler, NotificationCommandHandler};

    #[actix_rt::test]
    async fn test_ver_command() {
        //Arrange
        let command = String::from("VER 1 MSNP18 MSNP17 CVR0\r\n");
        let parsed = MSNPCommandParser::parse_message(&command);
        let (tx, mut rx1) = broadcast::channel(16);
        let mut rx2 = tx.subscribe();
        let mut handler = NotificationCommandHandler::new(tx);

        //Act
        let result = handler.handle_command(&parsed[0]).await;
        

        //Assert
        assert_eq!(result, "VER 1 MSNP18\r\n");
    }

    
    #[actix_rt::test]
    async fn test_cvr_command() {
        //Arrange
        let command = String::from(
            "CVR 2 0x0409 winnt 6.0.0 i386 MSNMSGR 14.0.8117.0416 msmsgs login@email.com\r\n",
        );
        let parsed = MSNPCommandParser::parse_message(&command);
        let (tx, mut rx1) = broadcast::channel(16);
        let mut rx2 = tx.subscribe();
        let mut handler = NotificationCommandHandler::new(tx);

        //Act
        let result = handler.handle_command(&parsed[0]).await;

        //Assert
        assert_eq!(
            result,
            "CVR 2 14.0.8117.0416 14.0.8117.0416 14.0.8117.0416 localhost localhost\r\n"
        );
    }
}
