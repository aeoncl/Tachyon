use crate::switchboard::handlers::handle_command;
use crate::switchboard::models::local_switchboard_data::LocalSwitchboardData;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::global_state::GlobalState;
use anyhow::anyhow;
use log::{debug, error, info};
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::raw_command_parser::RawCommandParser;
use msnp::msnp::switchboard::command::command::{SwitchboardClientCommand, SwitchboardServerCommand};
use msnp::shared::traits::{IntoBytes, TryFromRawCommand};
use std::str::from_utf8_unchecked;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::{broadcast, mpsc};

pub struct SwitchboardServer;


impl SwitchboardServer {
    pub async fn listen(ip_addr: &str, port: u32, global_kill_recv: Receiver<()>, tachyon_state: GlobalState) -> Result<(), anyhow::Error> {
        info!("Switchboard Server started...");

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
                        });
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
    debug!("Switchboard Client connected...");

    let (read, write) = socket.into_split();
    let (client_kill_snd, client_kill_recv) = broadcast::channel::<()>(1);
    let command_sender = start_write_task(write, client_kill_recv.resubscribe());

    let mut local_switchboard_data = LocalSwitchboardData::new(client_kill_recv);

    let mut parser = RawCommandParser::new();
    let mut reader = BufReader::new(read);
    let mut buffer= [0u8; 2048];

    loop {

        tokio::select! {
            bytes_read = reader.read(&mut buffer) => {
                match bytes_read {
                    Err(e) => {
                        error!("MSNP|SB: Socket Read Error: {}", e);
                        break;
                    },
                    Ok(bytes_read) => {

                        if bytes_read == 0 {
                            break;
                        }

                        let data = &buffer[..bytes_read];

                        let commands = parser.parse_message(data);

                        match commands {
                            Err(e) => error!("MSNP|SB: Unable to parse message into commands: {}", e),
                            Ok(commands) => {

                                for command in commands {
                                    unsafe {
                                        debug!("SB << | {}{}", command.get_command(), from_utf8_unchecked(command.get_payload()));
                                    }

                                    let notification_command = SwitchboardClientCommand::try_from_raw(command);
                                    match notification_command {
                                        Err(e) => {
                                            error!("MSNP|SB: Unable to parse command: {}", e);
                                            debug!("{:?}", e);
                                        },
                                        Ok(notification_command) => {
                                            let command_result = handle_command(notification_command, command_sender.clone(), &tachyon_state, &mut local_switchboard_data).await;

                                            if let Err(error) = command_result {
                                                error!("MSNP|SB: An error has occured handling a notification command: {}", &error);
                                                debug!("MSNP|SB: {:?}", &error);
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

    //Cleanup
    if let Some(room_id) = local_switchboard_data.room_id {
        if let Some(client) = local_switchboard_data.tachyon_client {
            client.switchboards().remove(&room_id);
        }
    }
    
    info!("SB Client gracefully shutdown...");
    Ok(())
}


fn start_write_task(mut write: OwnedWriteHalf, mut kill_recv: Receiver<()>) -> Sender<SwitchboardServerCommand> {
    println!("Socket write task started...");
    let (sender, mut receiver) = mpsc::channel::<SwitchboardServerCommand>(300);

    let _result = tokio::spawn(async move {
        loop {
            tokio::select! {
                command = receiver.recv() => {
                    if let Some(command) = command {

                        let bytes = command.into_bytes();

                        unsafe {
                            debug!("SB >> | {}", from_utf8_unchecked(&bytes));
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