use std::sync::Arc;

use substring::Substring;
use async_trait::async_trait;
use crate::repositories::matrix_client_repository::MatrixClientRepository;
use crate::repositories::repository::Repository;
use crate::utils::identifiers::msn_addr_to_matrix_id;
use crate::utils::matrix;
use crate::{CLIENT_DATA_REPO, MATRIX_CLIENT_REPO};
use crate::models::client_data::ClientData;
use crate::models::uuid::UUID;
use crate::repositories::client_data_repository::{ClientDataRepository};
use crate::models::msg_payload::factories::{MsgPayloadFactory};
use super::msnp_command::MSNPCommand;

pub struct NotificationCommandHandler {
    protocol_version: i16,
    msn_addr: String,
    matrix_token: String
}

pub struct SwitchboardCommandHandler {
    protocol_version: i16,
}

#[async_trait]
pub trait CommandHandler : Send {
    async fn handle_command(&mut self, command: &MSNPCommand) -> String;
}

impl NotificationCommandHandler {
    pub fn new() -> NotificationCommandHandler {
        return NotificationCommandHandler {
            protocol_version: -1,
            msn_addr: String::new(),
            matrix_token: String::new(),
        };
    }
}

impl SwitchboardCommandHandler {
    pub fn new() -> SwitchboardCommandHandler {
        return SwitchboardCommandHandler {
            protocol_version: -1,
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
                        return format!("USR {tr_id} SSO S MBI_KEY_OLD LAhAAUzdC+JvuB33nooLSa6Oh0oDFCbKrN57EVTY0Dmca8Reb3C1S1czlP12N8VU\r\nGCF 0 1233\r\n<Policies><Policy type= \"SHIELDS\"><config><shield><cli maj= \"7\" min= \"0\" minbld= \"0\" maxbld= \"9999\" deny= \" \" /></shield><block></block></config></Policy><Policy type= \"ABCH\"><policy><set id= \"push\" service= \"ABCH\" priority= \"200\"><r id= \"pushstorage\" threshold= \"180000\" /></set><set id= \"using_notifications\" service= \"ABCH\" priority= \"100\"><r id= \"pullab\" threshold= \"86400000\" timer= \"1800000\" trigger= \"Timer\" /><r id= \"pullmembership\" threshold= \"86400000\" timer= \"1800000\" trigger= \"Timer\" /></set><set id= \"delaysup\" service= \"ABCH\" priority= \"150\"><r id= \"whatsnew\" threshold= \"1800000\" /><r id= \"whatsnew_storage_ABCH_delay\" timer= \"1800000\" /><r id= \"whatsnewt_link\" threshold= \"900000\" trigger= \"QueryActivities\" /></set><c id= \"PROFILE_Rampup\">100</c></policy></Policy><Policy type= \"ERRORRESPONSETABLE\"><Policy><Feature type= \"3\" name= \"P2P\"><Entry hr= \"0x81000398\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /></Feature><Feature type= \"4\"><Entry hr= \"0x81000440\" /></Feature><Feature type= \"6\" name= \"TURN\"><Entry hr= \"0x8007274C\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /><Entry hr= \"0x8007274A\" action= \"3\" /></Feature></Policy></Policy><Policy type= \"P2P\"><ObjStr SndDly= \"1\" /></Policy></Policies>", tr_id = tr_id);
                    } else if phase == "S" {
                        self.matrix_token = split[4][2..split[4].chars().count()].to_string();
                        let matrix_id = msn_addr_to_matrix_id(&self.msn_addr);

                        if let Ok(client) = matrix::login(matrix_id, self.matrix_token.clone()).await {
                            //Token valid, client authenticated.
                            let client_data_repo : Arc<ClientDataRepository> = CLIENT_DATA_REPO.clone();
                            client_data_repo.add(self.matrix_token.clone(), ClientData::new(self.msn_addr.clone(), self.protocol_version.clone(), split[5].to_string()));

                            let matrix_client_repo : Arc<MatrixClientRepository> = MATRIX_CLIENT_REPO.clone();
                            matrix_client_repo.add(self.matrix_token.clone(), client);
    
                            let msmsgs_profile_msg = MsgPayloadFactory::get_msmsgs_profile(UUID::from_string(&self.msn_addr).get_puid(), self.msn_addr.clone(), self.matrix_token.clone()).serialize();
                            return format!("USR {tr_id} OK {email} 1 0\r\nSBS 0 null\r\nMSG HOTMAIL HOTMAIL {msmsgs_profile_payload_size}\r\n{payload}UBX 1:{email} 0\r\n", tr_id = tr_id, email=&self.msn_addr, msmsgs_profile_payload_size= msmsgs_profile_msg.len(), payload=msmsgs_profile_msg);
                        } else {
                            //Invalid token. Auth failure.
                            return format!("911 {tr_id}\r\n", tr_id= tr_id);
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
            "UUX" => {
                /*       0  1  2
                    >>> UUX 8 130 payload
                    <<< UUX 8 0
                */
                let tr_id = split[1];
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
                // >>> CHG 11 NLN 2789003324:48 0
                // <<< CHG 11 NLN 2789003324:48 0
                return format!("{}\r\n", command.command);
            },
            "PRP" => {
                // >>> PRP 13 MFN display%20name
                // <<< PRP 13 MFN display%20name
                return format!("{}\r\n", command.command);
            }
            _ => {
                return String::new();
            }
        }
    }
}

#[async_trait]
impl CommandHandler for SwitchboardCommandHandler {

    async fn handle_command(&mut self, command: &MSNPCommand) -> String {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::sockets::msnp_command::MSNPCommandParser;
    use crate::sockets::msnp_command_handlers::{CommandHandler, NotificationCommandHandler};

    #[actix_rt::test]
    async fn test_ver_command() {
        //Arrange
        let command = String::from("VER 1 MSNP18 MSNP17 CVR0\r\n");
        let parsed = MSNPCommandParser::parse_message(&command);
        let mut handler = NotificationCommandHandler::new();

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
        let mut handler = NotificationCommandHandler::new();

        //Act
        let result = handler.handle_command(&parsed[0]).await;

        //Assert
        assert_eq!(
            result,
            "CVR 2 14.0.8117.0416 14.0.8117.0416 14.0.8117.0416 localhost localhost\r\n"
        );
    }
}
