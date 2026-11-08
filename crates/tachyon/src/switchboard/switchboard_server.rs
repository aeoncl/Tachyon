use crate::switchboard::handlers::handle_command;
use crate::switchboard::models::local_switchboard_data::LocalSwitchboardData;
use crate::switchboard::models::switchboard_handle::SwitchboardHandle;
use crate::tachyon::client::tachyon_client::TachyonClient;
use crate::tachyon::global_state::GlobalState;
use crate::COMMAND_CHANNEL_BOUND;
use anyhow::anyhow;
use log::{debug, error, info, logger, warn};
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::raw_command_parser::RawCommandParser;
use msnp::msnp::switchboard::command::command::{
    SwitchboardClientCommand, SwitchboardServerCommand,
};
use msnp::shared::traits::{IntoBytes, TryFromRawCommand};
use std::str::from_utf8;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::{broadcast, mpsc};

pub struct SwitchboardServer;

impl SwitchboardServer {
    pub async fn listen(
        ip_addr: &str,
        port: u32,
        global_kill_recv: Receiver<()>,
        tachyon_state: GlobalState,
    ) -> Result<(), anyhow::Error> {
        info!("Switchboard Server started...");

        let listener = TcpListener::bind(format!("{}:{}", ip_addr, port))
            .await
            .map_err(|e| anyhow!(e))?;

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

async fn handle_client(
    socket: TcpStream,
    mut global_kill_recv: broadcast::Receiver<()>,
    tachyon_state: GlobalState,
) -> Result<(), anyhow::Error> {
    debug!("Switchboard Client connected...");

    let (read, write) = socket.into_split();
    let (client_kill_snd, client_kill_recv) = broadcast::channel::<()>(1);
    let command_sender = start_write_task(write, client_kill_recv.resubscribe());

    let mut local_switchboard_data = LocalSwitchboardData::new(client_kill_recv);

    let mut parser = RawCommandParser::new();
    let mut reader = BufReader::new(read);
    let mut buffer = [0u8; 2048];

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
                                    // Checked UTF-8 with a <binary> fallback (H1): MSG
                                    // payloads legitimately carry binary P2P blobs.
                                    // Log-only; wire bytes untouched.
                                    match from_utf8(command.get_payload()) {
                                        Ok(payload) => debug!("SB << | {}{}", command.get_command(), payload),
                                        Err(_) => debug!("SB << | {}<binary {} bytes>", command.get_command(), command.get_payload().len()),
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

    //Cleanup — unconditional (H9, §1.4, D4): pre-D4 this block ran AFTER
    //`client_kill_snd.send(())?`, so any kill-send error skipped the
    //remove and leaked a stale SwitchboardHandle in the client's map.
    //Remove-before-early-return: the cleanup no longer depends on the
    //kill channel's receiver bookkeeping at all.
    teardown_switchboard(&local_switchboard_data);

    // Err = no live kill receiver (the write task ended and dropped its
    // resubscribed receiver): the kill is then a no-op — warn, never skip
    // the cleanup above (H9). Failure-path-only delta: pre-D4 the `?`
    // returned early from handle_client with the handle still registered.
    if let Err(error) = client_kill_snd.send(()) {
        warn!("MSNP|SB: kill signal undeliverable, write task already gone: {}", error);
    }

    info!("SB Client gracefully shutdown...");
    Ok(())
}

/// H9 (§1.4, D4): the switchboard-handle cleanup, unconditional and in
/// its own function so the connection tail cannot reorder it behind a
/// fallible step again — and so the invariant is testable without a
/// socket.
fn teardown_switchboard(local_switchboard_data: &LocalSwitchboardData) {
    //Cleanup
    if let Some(room_id) = &local_switchboard_data.room_id {
        if let Some(client) = &local_switchboard_data.tachyon_client {
            client.switchboards().remove(room_id);
        }
    }
}

pub enum SwitchboardSenderMsg {
    Single(SwitchboardServerCommand),
    Chunks(Vec<SwitchboardServerCommand>),
}

fn start_write_task(
    mut write: OwnedWriteHalf,
    mut kill_recv: Receiver<()>,
) -> Sender<SwitchboardSenderMsg> {
    println!("Socket write task started...");
    let (sender, mut receiver) = mpsc::channel::<SwitchboardSenderMsg>(COMMAND_CHANNEL_BOUND);

    let _result = tokio::spawn(async move {
        'write_loop: loop {
            tokio::select! {
                sender_msg = receiver.recv() => {
                    if let Some(sender_msg) = sender_msg {

                        match sender_msg {
                            SwitchboardSenderMsg::Single(command) => {


                                let bytes = command.into_bytes();

                                // make this optional when debug logs are enabled
                                match from_utf8(&bytes) {
                                    Ok(utf8_str) => {
                                          debug!("SB >> | {}", utf8_str);
                                    }
                                    Err(_) => {
                                          debug!("SB >> | MSG Containing binary blob");

                                    }
                                }

                                // Explicit policy (H2): a dead socket ends the write
                                // task with a warn, never a panic — teardown continues
                                // through the existing kill paths.
                                if let Err(error) = write.write_all(&bytes).await {
                                    warn!("MSNP|SB: socket write failed, closing write task: {}", error);
                                    break 'write_loop;
                                }
                            }
                            SwitchboardSenderMsg::Chunks(chunks) => {
                                for chunk in chunks {
                                    let bytes = chunk.into_bytes();
                                    // Same policy as Single (H2).
                                    if let Err(error) = write.write_all(&bytes).await {
                                        warn!("MSNP|SB: socket write failed, closing write task: {}", error);
                                        break 'write_loop;
                                    }
                                }

                            }
                        }
                    }
                },
                _kill_signal = kill_recv.recv() => {
                    break;
                }
            }
        }
        println!("Socket write task gracefully shutdown...");
    });
    sender
}

#[cfg(test)]
mod tests {
    use matrix_sdk::ruma::{room_id, OwnedRoomId};
    use msnp::msnp::models::contact_list::ContactList;
    use msnp::msnp::switchboard::models::session_id::SessionId;
    use msnp::shared::models::email_address::EmailAddress;
    use msnp::shared::models::endpoint_id::EndpointId;
    use msnp::shared::models::msn_user::MsnUser;
    use msnp::shared::models::ticket_token::TicketToken;
    use std::str::FromStr;

    use super::teardown_switchboard;
    use crate::switchboard::models::local_switchboard_data::LocalSwitchboardData;
    use crate::switchboard::models::switchboard_handle::SwitchboardHandle;
    use crate::tachyon::client::tachyon_client::TachyonClient;
    use crate::tachyon::config::tachyon_config::TachyonConfig;

    fn test_client() -> TachyonClient {
        let (snd, _rcv) = tokio::sync::mpsc::channel(10);
        let (kill_snd, kill_recv) = tokio::sync::broadcast::channel::<()>(1);
        let msn_user = MsnUser::new(EndpointId::new(EmailAddress::from_str("aeon@test.com").unwrap(), None));
        TachyonClient::new(TachyonConfig::default(), msn_user, TicketToken::default(), snd, kill_snd, kill_recv, ContactList::default())
    }

    fn lsd_with(client: TachyonClient, room_id: OwnedRoomId) -> LocalSwitchboardData {
        let (_, kill_recv) = tokio::sync::broadcast::channel::<()>(1);
        let mut lsd = LocalSwitchboardData::new(kill_recv);
        lsd.room_id = Some(room_id);
        lsd.tachyon_client = Some(client);
        lsd
    }

    /// H9 (§1.4, D4): the cleanup removes the room's registered handle
    /// from the client's switchboard map — the leak the pre-D4
    /// `send(())?` skip caused. Socket-free: it calls the extracted
    /// teardown directly.
    #[test]
    fn teardown_removes_the_rooms_switchboard_handle() {
        let client = test_client();
        let room_id = room_id!("!sbroom:localhost").to_owned();
        client.switchboards().insert(SwitchboardHandle::new(SessionId::new(7), room_id.clone())).unwrap();
        assert!(client.switchboards().contains(&room_id), "precondition: the handle is registered");

        let lsd = lsd_with(client.clone(), room_id.clone());
        teardown_switchboard(&lsd);

        assert!(!client.switchboards().contains(&room_id), "teardown removed the stale handle");
    }

    /// H9's "unconditional" half: a connection that never resolved a room
    /// (killed during bootstrap) leaves nothing to remove and must not
    /// panic — the pre-D4 `if let` guards are preserved, so the no-room
    /// and no-client paths are still safe.
    #[test]
    fn teardown_without_a_room_or_client_is_a_safe_noop() {
        let client = test_client();
        let room_id = room_id!("!sbroom:localhost").to_owned();
        client.switchboards().insert(SwitchboardHandle::new(SessionId::new(7), room_id.clone())).unwrap();

        // No room_id / no tachyon_client on this connection's data.
        let (_, kill_recv) = tokio::sync::broadcast::channel::<()>(1);
        let bare = LocalSwitchboardData::new(kill_recv);
        teardown_switchboard(&bare);

        assert!(client.switchboards().contains(&room_id), "an unrelated connection's handle is untouched");
    }
}
