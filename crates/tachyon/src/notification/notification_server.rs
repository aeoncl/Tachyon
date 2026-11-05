use std::str::from_utf8;

use anyhow::anyhow;
use log::{debug, error, info, warn};
use msnp::msnp::notification::command::command::NotificationClientCommand;
use msnp::msnp::{notification::command::command::NotificationServerCommand, raw_command_parser::RawCommandParser};
use msnp::shared::traits::{IntoBytes, TryFromRawCommand};
use tokio::{io::{AsyncReadExt, AsyncWriteExt, BufReader}, net::{tcp::OwnedWriteHalf, TcpListener, TcpStream}, sync::{broadcast::{self, Receiver}, mpsc::{self, Sender}}};
use msnp::msnp::raw_command_parser::RawCommand;
use crate::notification::handlers::command_handler::handle_command;
use crate::COMMAND_CHANNEL_BOUND;
use crate::notification::models::session_state::SessionState;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::global_state::{ClientDropGuard, GlobalState};

pub struct NotificationServer;

/// Shell-owned state of one notification connection: the shutdown channel that
/// kills the connection's write task and the drop guard that unregisters the
/// client from the global state. Decision inputs live in `SessionState`.
pub(crate) struct ConnectionShell {
    pub(crate) client_shutdown_snd: broadcast::Sender<()>,
    pub(crate) client_shutdown_recv: Receiver<()>,
    pub(crate) client_drop_guard: Option<ClientDropGuard>,
}

impl ConnectionShell {
    pub(crate) fn new(client_shutdown_snd: broadcast::Sender<()>, client_shutdown_recv: Receiver<()>) -> Self {
        Self {
            client_shutdown_snd,
            client_shutdown_recv,
            client_drop_guard: None,
        }
    }
}


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
                        });
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

    let mut shell = ConnectionShell::new(client_shutdown_snd, client_shutdown_recv);
    let mut session_state = SessionState::new();

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
                                handle_commands(commands, &command_sender, &global_state, &mut session_state, &mut shell).await;
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

async fn handle_commands(commands: Vec<RawCommand>, command_sender: &Sender<NotificationServerCommand>, global_state: &GlobalState, session_state: &mut SessionState, shell: &mut ConnectionShell) {
    for command in commands {
        debug!("NS << | {}", command.get_command());

        let notification_command = NotificationClientCommand::try_from_raw(command);
        match notification_command {
            Err(e) => {
                error!("MSNP|NOT: Unable to parse command: {}", e);
                debug!("{:?}", e);
            },
            Ok(notification_command) => {
                let command_result = handle_command(notification_command, command_sender.clone(), &global_state, session_state, shell, &global_state.get_config()).await;

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
    info!("Socket write task started...");
    let (sender, mut receiver) = mpsc::channel::<NotificationServerCommand>(COMMAND_CHANNEL_BOUND);

    let _result = tokio::spawn(async move {
        loop {
            tokio::select! {
                command = receiver.recv() => {
                    if let Some(command) = command {

                        let bytes = command.into_bytes();

                        // Checked UTF-8 with a <binary> fallback (H1): rendered
                        // commands can legitimately carry binary payloads (UBX,
                        // RAW, P2P MSG blobs). Log-only; wire bytes untouched.
                        match from_utf8(&bytes) {
                            Ok(text) => debug!("NS >> | {}", text),
                            Err(_) => debug!("NS >> | <binary {} bytes>", bytes.len()),
                        }

                        // Explicit policy (H3), identical to the SB write task: a
                        // dead socket ends the write task with a warn — never
                        // silently ignore, a silently-dead write task hangs every
                        // sender once channels are bounded (§8 D2, §10).
                        if let Err(error) = write.write_all(&bytes).await {
                            warn!("MSNP|NOT: socket write failed, closing write task: {}", error);
                            break;
                        }
                    }
                },
                _kill_signal = kill_recv.recv() => {
                    break;
                }
            }
        }
        info!("Socket write task gracefully shutdown...");
    } );
    sender
}





