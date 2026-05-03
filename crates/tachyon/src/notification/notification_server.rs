use std::str::from_utf8_unchecked;

use anyhow::anyhow;
use log::{debug, error, info, warn};
use msnp::msnp::notification::command::command::NotificationClientCommand;
use msnp::msnp::{notification::command::command::NotificationServerCommand, raw_command_parser::RawCommandParser};
use msnp::shared::traits::{IntoBytes, TryFromRawCommand};
use tokio::{io::{AsyncReadExt, AsyncWriteExt, BufReader}, net::{tcp::OwnedWriteHalf, TcpListener, TcpStream}, sync::{broadcast::{self, Receiver}, mpsc::{self, Sender}}};
use msnp::msnp::raw_command_parser::RawCommand;
use crate::notification::handlers::command_handler::handle_command;
use crate::notification::models::local_client_data::LocalClientData;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::global::global_state::GlobalState;
use crate::tachyon::repository::RepositoryStr;

pub struct NotificationServer;


impl NotificationServer {

    pub async fn listen(ip_addr: &str, port: u32, global_shutdown_recv: Receiver<()>, global_state: GlobalState) -> Result<(), anyhow::Error>{
        info!("TCP Server started...");

        let listener = TcpListener::bind(format!("{}:{}", ip_addr, port))
            .await.map_err(|e| anyhow!(e))?;

            loop {
                let mut global_shutdown_recv_clone = global_shutdown_recv.resubscribe();
                let global_state_clone = global_state.clone();

                tokio::select! {
                    accepted = listener.accept() => {
                        let (socket, _addr)  = accepted.map_err(|e| anyhow!(e))?;
                        let _result = tokio::spawn(async move {
                            handle_client(socket, global_shutdown_recv_clone.resubscribe(), global_state_clone).await
                        }).await;
                    }
                    global_shutdown = global_shutdown_recv_clone.recv() => {
                        if let Err(err) = global_shutdown {
                            error!("Unable to listen for global kill: {}", err);
                        }
                        break;
                    }
                }               
            }

            info!("TCP Server gracefull shutdown...");
            Ok(())
    }

}



async fn handle_client(socket: TcpStream, mut global_shutdown_recv: broadcast::Receiver<()>, global_state: GlobalState) -> Result<(), anyhow::Error> {
    debug!("Client connected...");

    let (read, write) = socket.into_split();
    let (client_shutdown_snd, client_shutdown_recv) = broadcast::channel::<()>(1);
    let command_sender = start_write_task(write, client_shutdown_recv.resubscribe());

    let mut local_client_data = LocalClientData::new(client_shutdown_snd.clone(), client_shutdown_recv);

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
                            Err(e) => error!("MSNP|NOT: Unable to parse message into commands: {}", e),
                            Ok(commands) => {
                                handle_commands(commands, &command_sender, &global_state, &mut local_client_data).await;
                            }
                        }

                    }
                }
            },
            global_shutdown = global_shutdown_recv.recv() => {
                if let Err(err) = global_shutdown {
                    error!("Unable to listen for global kill: {}", err);
                }
                break;
            }
        }
    }

    info!("Client gracefully shutdown...");
    Ok(())

}

async fn handle_commands(commands: Vec<RawCommand>, command_sender: &Sender<NotificationServerCommand>, global_state: &GlobalState, local_client_data: &mut LocalClientData) {
    for command in commands {
        debug!("NS << | {}", command.get_command());

        let notification_command = NotificationClientCommand::try_from_raw(command);
        match notification_command {
            Err(e) => {
                error!("MSNP|NOT: Unable to parse command: {}", e);
                debug!("{:?}", e);
            },
            Ok(notification_command) => {
                let command_result = handle_command(notification_command, command_sender.clone(), &global_state, local_client_data, &global_state.get_config()).await;

                if let Err(error) = command_result {
                    error!("MSNP|NS: An error has occured handling a notification command: {}", &error);
                    debug!("MSNP|NS: {:?}", &error);
                    //TODO SEND ERROR BACK TO Client
                }
            }
        }
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

                        let bytes = command.into_bytes();

                        unsafe {
                            debug!("NS >> | {}", from_utf8_unchecked(&bytes));
                        }

                        let _result = write.write_all(&bytes).await;
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





