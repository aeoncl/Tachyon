use std::future::Future;
use std::path::Path;
use std::str::from_utf8_unchecked;

use anyhow::anyhow;
use log::{debug, error, info};
use matrix_sdk::Client;
use matrix_sdk::ruma::OwnedUserId;
use msnp::msnp::{notification::command::command::NotificationServerCommand, raw_command_parser::{RawCommand, RawCommandParser}};
use msnp::msnp::notification::command::command::NotificationClientCommand;
use msnp::msnp::notification::command::cvr::CvrServer;
use msnp::msnp::notification::command::msg::{MsgPayload, MsgServer};
use msnp::msnp::notification::command::usr::{AuthPolicy, OperationTypeClient, OperationTypeServer, SsoPhaseClient, SsoPhaseServer, UsrServer};
use msnp::msnp::notification::models::msnp_version::MsnpVersion::MSNP18;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::shared::models::uuid::Uuid;
use msnp::shared::payload::raw_msg_payload::factories::MsgPayloadFactory;
use msnp::shared::traits::MSNPCommand;
use tokio::{io::{AsyncReadExt, AsyncWriteExt, BufReader}, net::{tcp::OwnedWriteHalf, TcpListener, TcpStream}, sync::{broadcast::{self, Receiver}, mpsc::{self, Sender}}};
use tokio::sync::oneshot;
use crate::matrix;

use crate::notification::client_store::{ClientData, ClientDataGetter, ClientDataOperation, ClientDataSetter, ClientStoreFacade};
use crate::shared::identifiers::MatrixDeviceId;
use crate::shared::traits::TryFromMsnAddr;

pub struct NotificationServer;


impl NotificationServer {

    pub async fn listen(ip_addr: &str, port: u32, global_kill_recv: Receiver<()>, client_store_facade: ClientStoreFacade) -> Result<(), anyhow::Error>{
        info!("TCP Server started...");

        let listener = TcpListener::bind(format!("{}:{}", ip_addr, port))
            .await.map_err(|e| anyhow!(e))?;

            loop {
                let mut global_kill_recv = global_kill_recv.resubscribe();
                let client_store_facade = client_store_facade.clone();

                tokio::select! {
                    accepted = listener.accept() => {
                        let (socket, _addr)  = accepted.map_err(|e| anyhow!(e))?;
                        let _result = tokio::spawn(async move {handle_client(socket, global_kill_recv.resubscribe(), client_store_facade).await}).await;
                    }
                    global_kill = global_kill_recv.recv() => {
                        if let Err(err) = global_kill {
                            error!("Unable to listen for global kill: {}", err);
                        }
                        break;
                    }
                }               
            }

            info!("TCP Server gracefull shtudown...");
            Ok(())
    }

    
}

struct LocalStore {
    email_addr: String,
    token: TicketToken
}

impl Default for LocalStore {
    fn default() -> Self {
        Self {
            email_addr: String::new(),
            token: TicketToken(String::new()),
        }
    }
}

async fn handle_client(socket: TcpStream, mut global_kill_recv : broadcast::Receiver<()>, client_store_facade: ClientStoreFacade) -> Result<(), anyhow::Error> {

    debug!("Client connected...");

    let (read, write) = socket.into_split();
    
    let (client_kill_snd, client_kill_recv) = broadcast::channel::<()>(1);

    let command_sender = start_write_task(write, client_kill_recv.resubscribe());

    let mut local_store = LocalStore::default();



    let mut parser = RawCommandParser::new();
    let mut reader = BufReader::new(read);
    let mut buffer= [0u8; 2048];

    loop {

        tokio::select! {
            bytes_read = reader.read(&mut buffer) => {
                match bytes_read {
                    Err(e) => {
                        error!("MSNP|NOT: Socket Read Error: {}", e);
                        break;
                    },
                    Ok(bytes_read) => {

                        if bytes_read == 0 {
                            break;
                        }

                        let data = &buffer[..bytes_read];

                        let commands = parser.parse_message(data);

                        match commands {
                            Err(e) => error!("MSNP|NOT: Unable to parse commands: {}", e),
                            Ok(commands) => {

                                for command in commands {
                                    debug!("NS << | {}", command.get_command());


                                    let notification_command = NotificationClientCommand::try_from_raw(command);
                                    match notification_command {
                                        Err(e) => {
                                            error!("MSNP|NOT: Unable to parse command: {}", e);
                                            debug!("{:?}", e);
                                        },
                                        Ok(notification_command) => {
                                            let command_result = handle_command(notification_command, command_sender.clone(), &client_store_facade, &mut local_store, &client_kill_recv).await;
                                            if let Err(error) = command_result {
                                                error!("MSNP|NS: An error has occured handling a notification command: {}", &error);
                                                debug!("MSNP|NS: {:?}", &error);
                                                //TODO SEND ERROR BACK TO Client
                                            }


                                        }
                                    }
                                }
                            }

                        }

                    }
                }
            },
            global_kill = global_kill_recv.recv() => {
                if let Err(err) = global_kill {
                    error!("Unable to listen for global kill: {}", err);
                }
                break;
            }
        }
    }

    client_kill_snd.send(())?;

    info!("Client gracefully shutdown...");
    Ok(())

}

fn start_write_task(mut write: OwnedWriteHalf, mut kill_recv: Receiver<()>) -> Sender<NotificationServerCommand> {
    println!("Socket write task started...");
    let (sender, mut receiver) = mpsc::channel::<NotificationServerCommand>(300);

    let _result = tokio::spawn(async move {
        loop {
            tokio::select! {
                command = receiver.recv() => {
                    if let Some(command) = command {

                        let bytes = command.to_bytes();

                        unsafe {
                            debug!("NS >> | {}", from_utf8_unchecked(&bytes));
                        }

                        let result = write.write_all(&bytes).await;
                    }
                },
                _kill_signal = kill_recv.recv() => {
                    break;
                }
            }
        }
        println!("Socket write task gracefully shutdown...");
    } );
    sender
}

const SHIELDS_PAYLOAD: &str = "<Policies><Policy type= \"SHIELDS\"><config><shield><cli maj= \"7\" min= \"0\" minbld= \"0\" maxbld= \"9999\" deny= \" \" /></shield><block></block></config></Policy><Policy type= \"ABCH\"><policy><set id= \"push\" service= \"ABCH\" priority= \"200\"><r id= \"pushstorage\" threshold= \"0\" /></set><set id= \"using_notifications\" service= \"ABCH\" priority= \"100\"><r id= \"pullab\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /><r id= \"pullmembership\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /></set><set id= \"delaysup\" service= \"ABCH\" priority= \"150\"><r id= \"whatsnew\" threshold= \"0\" /><r id= \"whatsnew_storage_ABCH_delay\" timer= \"1800000\" /><r id= \"whatsnewt_link\" threshold= \"0\" trigger= \"QueryActivities\" /></set><c id= \"PROFILE_Rampup\">100</c></policy></Policy><Policy type= \"ERRORRESPONSETABLE\"><Policy><Feature type= \"3\" name= \"P2P\"><Entry hr= \"0x81000398\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /></Feature><Feature type= \"4\"><Entry hr= \"0x81000440\" /></Feature><Feature type= \"6\" name= \"TURN\"><Entry hr= \"0x8007274C\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /><Entry hr= \"0x8007274A\" action= \"3\" /></Feature></Policy></Policy><Policy type= \"P2P\"><ObjStr SndDly= \"1\" /></Policy></Policies>";

async fn handle_command(raw_command: NotificationClientCommand, notif_sender: Sender<NotificationServerCommand>, client_store: &ClientStoreFacade, mut local_store: &mut LocalStore, kill_signal: &broadcast::Receiver<()>) -> Result<(), anyhow::Error> {
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
            notif_sender.send(NotificationServerCommand::CVR(CvrServer::new(command.tr_id, "14.0.8117.0416".to_string(), "14.0.8117.0416".to_string(), "14.0.8117.0416".to_string(), "localhost".to_string(), "localhost".to_string() ))).await?;
            Ok(())
        },
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

                            let matrix_client = matrix::login::login(OwnedUserId::try_from_msn_addr(&local_store.email_addr)?, MatrixDeviceId::from_hostname()?, ticket_token.clone(), &Path::new(format!("C:\\temp\\{}", &local_store.email_addr).as_str()), None, false).await?;

                            local_store.token = ticket_token.clone();
                            client_store.set_client_email(&ticket_token.0, local_store.email_addr.clone()).await?;
                            client_store.set_ticket_token_and_endpoint_guid(&ticket_token.0, ticket_token.clone(), endpoint_guid).await?;
                            client_store.set_matrix_client(&ticket_token.0, matrix_client.clone()).await?;

                            let usr_response = UsrServer::new(command.tr_id, OperationTypeServer::Ok {
                                email_addr: local_store.email_addr.clone(),
                                verified: true,
                                unknown_arg: false,
                            });

                            notif_sender.send(NotificationServerCommand::USR(usr_response)).await?;
                            notif_sender.send(NotificationServerCommand::RAW(RawCommand::without_payload("SBS 0 null"))).await?;

                            matrix::sync::start_sync_task(matrix_client, notif_sender.clone(), client_store.clone(), kill_signal.resubscribe()).await;


                            let uuid = Uuid::from_seed(&local_store.email_addr);

                            let initial_profile_msg = NotificationServerCommand::MSG(MsgServer {
                                sender: "Hotmail".to_string(),
                                display_name: "Hotmail".to_string(),
                                payload: MsgPayload::Raw(MsgPayloadFactory::get_msmsgs_profile(&uuid.get_puid(), &local_store.email_addr, &ticket_token))
                            });

                            notif_sender.send(initial_profile_msg).await?;

                            notif_sender.send(NotificationServerCommand::MSG(MsgServer {
                                sender: "Hotmail".to_string(),
                                display_name: "Hotmail".to_string(),
                                payload: MsgPayload::Raw(MsgPayloadFactory::get_initial_mail_data_notification())
                            })).await?;

                            //Todo fetch endpoint data
                            let endpoint_data = b"<Data></Data>";

                            notif_sender.send(NotificationServerCommand::RAW(RawCommand::with_payload(&format!("UBX 1:{}", &local_store.email_addr), endpoint_data.to_vec()))).await?;


                        }

                    }
                },
                OperationTypeClient::Sha() => {

                }

            }
            Ok(())
        },
        NotificationClientCommand::PNG => {
            notif_sender.send(NotificationServerCommand::QNG(60)).await?;
            Ok(())
        }
        //Seems to wait indefinitely for ADL Response
        NotificationClientCommand::ADL(command) => {

            notif_sender.send(NotificationServerCommand::Ok(command.get_ok_response("ADL"))).await?;

            Ok(())
        }
        NotificationClientCommand::RML(command) => {Ok(())}
        NotificationClientCommand::UUX(command) => {
            notif_sender.send(NotificationServerCommand::Uux(command.get_ok_response())).await?;
            Ok(())
        }
        NotificationClientCommand::BLP(command) => {
            notif_sender.send(NotificationServerCommand::BLP(command)).await?;
            Ok(())
        }
        NotificationClientCommand::CHG(command) => {
            notif_sender.send(NotificationServerCommand::CHG(command)).await?;
            Ok(())
        }
        NotificationClientCommand::PRP(command) => {Ok(())}
        NotificationClientCommand::UUN(command) => {Ok(())}
        NotificationClientCommand::XFR() => {Ok(())}
        NotificationClientCommand::RAW(command) => {Ok(())}
        NotificationClientCommand::OUT => {Ok(())}
    }
}


