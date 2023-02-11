use std::str::{FromStr};
use std::sync::Arc;

use substring::Substring;
use async_trait::async_trait;
use tokio::sync::broadcast::{Sender};
use crate::generated::payloads::{PrivateEndpointData, PresenceStatus};
use crate::models::ab_data::AbData;
use crate::models::errors::{MsnpErrorCode};

use crate::models::p2p::slp_payload::SlpPayload;
use crate::models::p2p::slp_payload_handler::SlpPayloadHandler;
use crate::repositories::matrix_client_repository::MatrixClientRepository;
use crate::repositories::repository::Repository;
use crate::utils::identifiers::{msn_addr_to_matrix_id};
use crate::utils::matrix;
use crate::{CLIENT_DATA_REPO, MATRIX_CLIENT_REPO, AB_DATA_REPO};
use crate::models::client_data::ClientData;
use crate::models::uuid::UUID;
use crate::repositories::client_data_repository::{ClientDataRepository};
use crate::models::msg_payload::factories::{MsgPayloadFactory};
use super::command_handler::CommandHandler;
use super::msnp_command::MSNPCommand;
use crate::utils::matrix_sync_helpers::*;

pub struct NotificationCommandHandler {
    protocol_version: i16,
    msn_addr: String,
    matrix_token: String,
    sender: Sender<String>,
    kill_sender: Option<Sender<String>>
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
                            let this_device_id = client.device_id().unwrap();

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
                    }

                } else {
                    // this not for me
                    if command.payload.contains("MSNSLP/1.0") {
                        //slp payload
                        let slp_request = SlpPayload::from_str(command.payload.as_str()).unwrap();
                        let slp_response = SlpPayloadHandler::handle(&slp_request).unwrap();

                        let payload = slp_response.to_string();
                        return format!("UUN {tr_id} OK\r\nUBN {msn_addr} 5 {payload_size}\r\n{payload}", tr_id = tr_id, msn_addr= &receiver_msn_addr, payload=&payload, payload_size = payload.len());
                    }


                    return format!("UUN {tr_id} OK\r\n", tr_id = tr_id);                
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

                 let device_id = matrix_client.device_id().unwrap();
                 matrix_client.update_device(device_id.to_owned(), private_endpoint_data.ep_name).await.unwrap_or_default();

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

