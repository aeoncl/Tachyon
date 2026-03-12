use tokio::sync::mpsc;
use msnp::msnp::switchboard::command::command::SwitchboardServerCommand;

#[derive(Clone)]
pub struct SwitchboardHandle {
    room_id: String,
    msnp_sender: mpsc::Sender<SwitchboardServerCommand>,
    p2p_sender: mpsc::Sender<String>,
}