use std::future::Future;

use matrix_sdk::ruma::events::key::verification::start;
use msnp::{msnp::{notification::command::command::NotificationServerCommand, raw_command_parser::{RawCommand, RawCommandParser}}, shared::command::command::SerializeMsnp};
use tokio::{io::{AsyncReadExt, AsyncWriteExt, BufReader}, net::{tcp::{OwnedWriteHalf, WriteHalf}, TcpListener, TcpStream}, signal, sync::{broadcast::{self, Receiver}, mpsc::{self, Sender}}};
use anyhow::anyhow;
use std::mem;
use msnp::msnp::notification::command::command::NotificationClientCommand;
use msnp::msnp::notification::command::cvr::CvrServer;
use msnp::msnp::notification::command::usr::{AuthPolicy, OperationTypeClient, OperationTypeServer, SsoPhaseClient, SsoPhaseServer, TicketToken, UsrServer};
use msnp::msnp::notification::models::msnp_version::MsnpVersion::MSNP18;
use tokio::sync::oneshot;
use crate::notification::client_store::{ClientStore, ClientStoreEvent, ClientStoreGetterEvent, ClientStoreSetterEvent};

pub struct NotificationServer;


impl NotificationServer {

    pub fn new() -> Self {
        Self
    }

    pub async fn listen(&self, ip_addr: &str, port: u32, global_kill_recv: broadcast::Receiver<()>) -> Result<(), anyhow::Error>{
        println!("TCP Server started...");

        let listener = TcpListener::bind(format!("{}:{}", ip_addr, port))
            .await.map_err(|e| anyhow!(e))?;

            loop {
                let mut global_kill_recv = global_kill_recv.resubscribe();

                tokio::select! {
                    accepted = listener.accept() => {
                        let (socket, _addr)  = accepted.map_err(|e| anyhow!(e))?;
                        let _result = tokio::spawn(async move {handle_client(socket, global_kill_recv.resubscribe()).await}).await;
                    }
                    global_kill = global_kill_recv.recv() => {
                        if let Err(err) = global_kill {
                            println!("Unable to listen for global kill: {}", err);
                        }
                        break;
                    }
                }               
            }   

            println!("TCP Server gracefull shtudown...");
            Ok(())
    }

    
}

async fn handle_client(socket: TcpStream, mut global_kill_recv : broadcast::Receiver<()>) -> Result<(), anyhow::Error> {    

    println!("Client connected...");

    let (read, write) = socket.into_split();
    
    let (client_kill_snd, client_kill_recv) = broadcast::channel::<()>(1);

    let command_sender = start_write_task(write, client_kill_recv.resubscribe());

    let client_store_sender = start_client_store_task(client_kill_recv.resubscribe());

    let mut email_addr = "test".to_string();

    let mut parser = RawCommandParser::new();
    let mut reader = BufReader::new(read);
    let mut buffer= [0u8; 2048];

    loop {

        tokio::select! {
            bytes_read = reader.read(&mut buffer) => {
                match bytes_read {
                    Err(e) => {
                        //TODO add log
                        eprintln!("SOCKET ERROR");
                        break;
                    },
                    Ok(bytes_read) => {

                        if bytes_read == 0 {
                            break;
                        }

                        let data = &buffer[..bytes_read];

                        let commands = parser.parse_message(data);

                        match commands {
                            Err(e) => println!("Unable to parse commands: {}", e),
                            Ok(commands) => {
                                for command in commands {
                                    let notification_command = NotificationClientCommand::try_from(command);
                                    match notification_command {
                                        Err(e) => println!("Unable to parse command: {}", e),
                                        Ok(notification_command) => {
                                            handle_command(notification_command, command_sender.clone(), client_store_sender.clone(), &mut email_addr).await;
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
                    println!("Unable to listen for global kill: {}", err);
                }
                break;
            }
        }
    }

    client_kill_snd.send(())?;

    println!("Client gracefully shutdown...");
    Ok(())

}


const SHIELDS_PAYLOAD: &str = "<Policies><Policy type= \"SHIELDS\"><config><shield><cli maj= \"7\" min= \"0\" minbld= \"0\" maxbld= \"9999\" deny= \" \" /></shield><block></block></config></Policy><Policy type= \"ABCH\"><policy><set id= \"push\" service= \"ABCH\" priority= \"200\"><r id= \"pushstorage\" threshold= \"0\" /></set><set id= \"using_notifications\" service= \"ABCH\" priority= \"100\"><r id= \"pullab\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /><r id= \"pullmembership\" threshold= \"0\" timer= \"1800000\" trigger= \"Timer\" /></set><set id= \"delaysup\" service= \"ABCH\" priority= \"150\"><r id= \"whatsnew\" threshold= \"0\" /><r id= \"whatsnew_storage_ABCH_delay\" timer= \"1800000\" /><r id= \"whatsnewt_link\" threshold= \"0\" trigger= \"QueryActivities\" /></set><c id= \"PROFILE_Rampup\">100</c></policy></Policy><Policy type= \"ERRORRESPONSETABLE\"><Policy><Feature type= \"3\" name= \"P2P\"><Entry hr= \"0x81000398\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /></Feature><Feature type= \"4\"><Entry hr= \"0x81000440\" /></Feature><Feature type= \"6\" name= \"TURN\"><Entry hr= \"0x8007274C\" action= \"3\" /><Entry hr= \"0x82000020\" action= \"3\" /><Entry hr= \"0x8007274A\" action= \"3\" /></Feature></Policy></Policy><Policy type= \"P2P\"><ObjStr SndDly= \"1\" /></Policy></Policies>";

async fn handle_command(raw_command: NotificationClientCommand, notif_sender: Sender<NotificationServerCommand>, client_store_sender: Sender<ClientStoreEvent>, mut email_addr:  &String) {
    email_addr = &"".to_string();
    match raw_command {
        NotificationClientCommand::VER(command) => {
            if command.first_candidate != MSNP18 && command.second_candidate != MSNP18 {
                //Unsupported protocol version
                //TODO add error code
                notif_sender.send(NotificationServerCommand::OUT).await?;
                return;
            }

            notif_sender.send(NotificationServerCommand::VER(command.get_response_for(MSNP18))).await?;
        },
        NotificationClientCommand::CVR(command) => {
            notif_sender.send(NotificationServerCommand::CVR(CvrServer::new(command.tr_id, "14.0.8117.0416".to_string(), "14.0.8117.0416".to_string(), "14.0.8117.0416".to_string(), "localhost".to_string(), "localhost".to_string() ))).await?;
        },
        NotificationClientCommand::USR(command) => {
            match command.auth_type {

                OperationTypeClient::Sso(content) => {

                    match content {

                        SsoPhaseClient::I { email_addr } => {
                            client_store_sender.send(ClientStoreEvent::Setter(ClientStoreSetterEvent::SetClientEmail(email_addr))).await?;

                            let usr_response = UsrServer::new(command.tr_id, OperationTypeServer::Sso(SsoPhaseServer::S { policy: AuthPolicy::MbiKeyOld, nonce: "LAhAAUzdC+JvuB33nooLSa6Oh0oDFCbKrN57EVTY0Dmca8Reb3C1S1czlP12N8VU".to_string() }));
                            let gcf_response = RawCommand::with_payload("GCF 0", SHIELDS_PAYLOAD.as_bytes().to_vec())?;

                            notif_sender.send(NotificationServerCommand::USR(usr_response)).await?;
                            notif_sender.send(NotificationServerCommand::RAW(gcf_response)).await?;
                        },

                        SsoPhaseClient::S { ticket_token, challenge, endpoint_guid } => {
                            client_store_sender.send(ClientStoreEvent::Setter(ClientStoreSetterEvent::SetTicketTokenAndEndpoint(ticket_token, endpoint_guid))).await?;

                            let (snd, rcv) = oneshot::channel::<Option<String>>();
                            client_store_sender.send(ClientStoreEvent::Getter(ClientStoreGetterEvent::GetClientEmail(snd))).await?;
                            let email_addr = rcv.await?;

                            let usr_response = UsrServer::new(command.tr_id, OperationTypeServer::Ok {
                                email_addr: "".to_string(),
                                verified: true,
                                unknown_arg: false,
                            });

                            notif_sender.send(NotificationServerCommand::USR(usr_response)).await?;
                        }

                    }
                },
                OperationTypeClient::Sha() => {

                }
            }
        },
        NotificationClientCommand::PNG => {
            notif_sender.send(NotificationServerCommand::QNG(60)).await?;
        }
        NotificationClientCommand::ADL(command) => {}
        NotificationClientCommand::RML(command) => {}
        NotificationClientCommand::UUX(command) => {}
        NotificationClientCommand::BLP(command) => {}
        NotificationClientCommand::CHG(command) => {}
        NotificationClientCommand::PRP(command) => {}
        NotificationClientCommand::UUN(command) => {}
        NotificationClientCommand::XFR() => {}
        NotificationClientCommand::RAW(command) => {}
    }
}

fn start_write_task(mut write: OwnedWriteHalf, mut kill_recv: Receiver<()>) -> Sender<NotificationServerCommand> {
    println!("Socket write task started...");
    let (sender, mut receiver) = mpsc::channel::<NotificationServerCommand>(300);

    let _result = tokio::spawn(async move {
        loop {
            tokio::select! {
                command = receiver.recv() => {
                    if let Some(command) = command {
                        let result = write.write_all(&command.serialize_msnp()).await;
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

fn start_client_store_task(mut kill_recv: Receiver<()>) -> Sender<ClientStoreEvent> {
    let (sender, mut receiver) = mpsc::channel::<ClientStoreEvent>(300);

    let _result = tokio::spawn(async move {
        let mut client_store = ClientStore::default();

        loop {
            tokio::select! {
                event = receiver.recv() => {
                    if let Some(event) = event {
                        match event {
                            ClientStoreEvent::Setter(setter) => {
                                match setter {
                                    ClientStoreSetterEvent::SetClientEmail(email_addr) => {
                                        client_store.email_addr = Some(email_addr);
                                    },
                                    ClientStoreSetterEvent::SetTicketTokenAndEndpoint(ticket_token, endpoint_guid) => {
                                        client_store.ticket_token = Some(ticket_token);
                                        client_store.endpoint_guid = Some(endpoint_guid);
                                    }
                                    ClientStoreSetterEvent::SetMatrixClient(client) => {
                                        client_store.matrix_client = Some(client);
                                    }
                                }
                            },
                            ClientStoreEvent::Getter(getter) => {
                                match getter {
                                    ClientStoreGetterEvent::GetClientEmail(channel) => {
                                        let _result = channel.send(client_store.email_addr.clone());
                                    },
                                    ClientStoreGetterEvent::GetClientTicketToken(channel) => {
                                        let _result = channel.send(client_store.ticket_token.clone());

                                    },
                                    ClientStoreGetterEvent::GetClientEndpointGuid(channel) => {
                                       let _result = channel.send(client_store.endpoint_guid.clone());
                                    }
                                    ClientStoreGetterEvent::GetMatrixClient(channel) => {
                                       let _result = channel.send(client_store.matrix_client.clone());
                                    }}

                            }
                        }
                    }
                },
                _kill_signal = kill_recv.recv() => {
                    break;
                }
            }
        }
        println!("ClientStore task gracefully shutdown...");
    } );
    sender

}
