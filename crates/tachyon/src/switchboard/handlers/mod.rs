use crate::switchboard::handlers::bootstrap_handlers::{handle_auth, handle_init};
use crate::switchboard::handlers::ready::handle_ready;
use crate::switchboard::models::connection_phase::ConnectionPhase;
use crate::switchboard::models::local_switchboard_data::LocalSwitchboardData;
use crate::switchboard::models::switchboard_handle::{SwitchboardHandle, SwitchboardState};
use crate::tachyon::global_state::GlobalState;
use anyhow::anyhow;
use matrix_sdk::ruma::OwnedRoomId;
use msnp::msnp::switchboard::command::command::{SwitchboardClientCommand, SwitchboardServerCommand};
use msnp::msnp::switchboard::models::session_id::SessionId;
use tokio::sync::mpsc::Sender;
use crate::switchboard::switchboard_server::SwitchboardSenderMsg;

mod bootstrap_handlers;
mod ready;

/// Pattern-A output (§3.1): the bootstrap handlers return plain wire data —
/// the commands the dispatcher sends, in order, each wrapped in the Single
/// envelope (the envelope wrapping stays shell-side) — plus at most one
/// ready-wiring the dispatcher executes AFTER the sends. The Sender left
/// the handler signatures with D4.
pub(crate) struct BootstrapOutcome {
    pub(crate) commands: Vec<SwitchboardServerCommand>,
    pub(crate) wiring: Option<ReadyWiring>,
}

impl BootstrapOutcome {
    pub(crate) fn none() -> Self {
        Self { commands: vec![], wiring: None }
    }
}

/// The Sender hand-over (§3.1): a `SwitchboardHandle` stores the shell's
/// Sender as its Ready state, so the wiring is shell work — the handlers
/// decide WHICH room/session, the dispatcher owns the channel. Both
/// variants are real pre-D4 flows (ANS flips the existing handle, CAL
/// inserts a fresh one); neither is speculative.
pub(crate) enum ReadyWiring {
    /// ANS flow: flip the room's existing handle to Ready. Must run after
    /// the flow's commands are on the wire — set_state(Ready) flushes the
    /// pending events, and that flush landed after the roster/OK/JOI sends
    /// pre-D4.
    FlipHandleReady { room_id: OwnedRoomId },
    /// CAL flow: insert a fresh ready handle for the callee's room, also
    /// after the sends (the target's JOI preceded the handle's existence
    /// pre-D4).
    InsertReadyHandle { session_id: SessionId, room_id: OwnedRoomId },
}

pub(crate) async fn handle_command(command: SwitchboardClientCommand, command_sender: Sender<SwitchboardSenderMsg>, tachyon_state: &GlobalState, local_switchboard_data: &mut LocalSwitchboardData) -> Result<(), anyhow::Error> {

    match local_switchboard_data.phase {
        ConnectionPhase::Authenticating => {
            let outcome = handle_auth(command, tachyon_state, local_switchboard_data).await?;
            send_bootstrap_outcome(outcome, &command_sender, local_switchboard_data).await?;
        }
        ConnectionPhase::Initializing => {
            let tachyon_client = local_switchboard_data.tachyon_client.as_ref().ok_or(anyhow!("Client Data should be here by now"))?.clone();
            let matrix_client = local_switchboard_data.matrix_client.as_ref().ok_or(anyhow!("Matrix Client Data should be here by now"))?.clone();
            let outcome = handle_init(command, tachyon_client, matrix_client, local_switchboard_data).await?;
            send_bootstrap_outcome(outcome, &command_sender, local_switchboard_data).await?;
        }
        ConnectionPhase::Ready => {
            // The ready-phase gather is room + client only: no Ready flow
            // reads lsd.matrix_client (the pre-D4 pull fed an unused
            // handle_msg parameter).
            let room = local_switchboard_data.room.as_ref().ok_or(anyhow!("Room should be here by now"))?.clone();
            let tachyon_client = local_switchboard_data.tachyon_client.as_ref().ok_or(anyhow!("Tachyon Client should be here by now"))?.clone();
            handle_ready(command, command_sender, tachyon_client, room, local_switchboard_data).await?
        }
    }
    Ok(())
}

/// The dispatcher side of pattern A (§3.1): send the decided commands in
/// order — each as its own Single, the pre-D4 envelope — then execute the
/// flow's ready-wiring with the shell-owned Sender. A send error (client
/// gone) propagates before the remaining commands and the wiring run,
/// exactly as the pre-D4 `?` chain aborted mid-flow.
async fn send_bootstrap_outcome(outcome: BootstrapOutcome, command_sender: &Sender<SwitchboardSenderMsg>, local_switchboard_data: &LocalSwitchboardData) -> Result<(), anyhow::Error> {
    for command in outcome.commands {
        command_sender.send(SwitchboardSenderMsg::Single(command)).await?;
    }

    if let Some(wiring) = outcome.wiring {
        let tachyon_client = local_switchboard_data.tachyon_client.as_ref().ok_or(anyhow!("Client Data should be here by now"))?;
        match wiring {
            ReadyWiring::FlipHandleReady { room_id } => {
                //TODO: send an error if this expect is not here.
                let mut switchboard_handle = tachyon_client.switchboards().get(room_id.as_ref()).expect("To be here");
                //set handle as ready — the pending-events flush (Chunks
                //envelope) stays shell-side: it needs the shell's Sender.
                switchboard_handle.set_state(SwitchboardState::Ready {
                    msnp_sender: command_sender.clone()
                }).await?;
            }
            ReadyWiring::InsertReadyHandle { session_id, room_id } => {
                let switchboard_handle = SwitchboardHandle::new_ready(session_id, room_id, command_sender.clone(), tachyon_client.dialect().profile().max_msg_body_size);
                tachyon_client.switchboards().insert(switchboard_handle)?;
            }
        }
    }

    Ok(())
}
