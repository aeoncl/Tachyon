use std::str::from_utf8_unchecked;

use anyhow::anyhow;
use log::{debug, error, info, warn};
use msnp::msnp::notification::command::command::NotificationClientCommand;
use msnp::msnp::{notification::command::command::NotificationServerCommand, raw_command_parser::RawCommandParser};
use msnp::shared::traits::{IntoBytes, TryFromRawCommand};
use tokio::{io::{AsyncReadExt, AsyncWriteExt, BufReader}, net::{tcp::OwnedWriteHalf, TcpListener, TcpStream}, sync::{broadcast::{self, Receiver}, mpsc::{self, Sender}}};

use crate::notification::handlers::command_handler::handle_command;
use crate::notification::models::local_client_data::LocalClientData;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::global_state::GlobalState;
use crate::tachyon::repository::RepositoryStr;

pub struct NotificationServer;


impl NotificationServer {

    pub async fn listen(ip_addr: &str, port: u32, global_kill_recv: Receiver<()>, tachyon_state: GlobalState) -> Result<(), anyhow::Error>{
        info!("TCP Server started...");

        let listener = TcpListener::bind(format!("{}:{}", ip_addr, port))
            .await.map_err(|e| anyhow!(e))?;

            loop {
                let mut global_kill_recv = global_kill_recv.resubscribe();
                let tachyon_state_clone = tachyon_state.clone();

                tokio::select! {
                    accepted = listener.accept() => {
                        let (socket, _addr)  = accepted.map_err(|e| anyhow!(e))?;
                        let _result = tokio::spawn(async move {
                            handle_client(socket, global_kill_recv.resubscribe(), tachyon_state_clone).await
                        }).await;
                    }
                    global_kill = global_kill_recv.recv() => {
                        if let Err(err) = global_kill {
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



async fn handle_client(socket: TcpStream, mut global_kill_recv : broadcast::Receiver<()>, tachyon_state: GlobalState) -> Result<(), anyhow::Error> {
    debug!("Client connected...");

    let (read, write) = socket.into_split();
    let (client_kill_snd, client_kill_recv) = broadcast::channel::<()>(1);
    let command_sender = start_write_task(write, client_kill_recv.resubscribe());

    let mut local_client_data = LocalClientData::new(client_kill_snd.clone(), client_kill_recv);

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

                                for command in commands {
                                    debug!("NS << | {}", command.get_command());

                                    let notification_command = NotificationClientCommand::try_from_raw(command);
                                    match notification_command {
                                        Err(e) => {
                                            error!("MSNP|NOT: Unable to parse command: {}", e);
                                            debug!("{:?}", e);
                                        },
                                        Ok(notification_command) => {
                                            let command_result = handle_command(notification_command, command_sender.clone(), &tachyon_state, &mut local_client_data).await;

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

    if let Err(e) = client_kill_snd.send(()) {
        error!("NS: Unable to send kill signal to client: {}", e);
    }

    let removed = tachyon_state.tachyon_clients().remove(local_client_data.token.0.as_str());
    if let Some(client_data) = removed {
        info!("Tachyon Client: {} removed successfully", client_data.own_user().get_email_address());
    } else {
        warn!("Failed to remove client data");
    }

    let removed = tachyon_state.matrix_clients().remove(local_client_data.token.0.as_str());
    if let Some(client_data) = removed {
        info!("Matrix Client: {} removed successfully", local_client_data.email_addr.as_str());
    } else {
        warn!("Failed to remove client data");
    }
    
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





