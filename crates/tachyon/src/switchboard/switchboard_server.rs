use tokio::sync::broadcast::Receiver;
use crate::notification::client_store::ClientStoreFacade;

pub struct SwitchboardServer;


impl SwitchboardServer {
    pub async fn listen(ip_addr: &str, port: u32, global_kill_recv: Receiver<()>, client_store_facade: ClientStoreFacade) -> Result<(), anyhow::Error> {
        Ok(())
    }
}