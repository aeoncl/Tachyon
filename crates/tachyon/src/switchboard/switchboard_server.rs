use tokio::sync::broadcast::Receiver;
use crate::notification::client_store::ClientStoreFacade;

pub struct SwitchboardServer;


impl SwitchboardServer {
    pub async fn listen(_ip_addr: &str, _port: u32, _global_kill_recv: Receiver<()>, _client_store_facade: ClientStoreFacade) -> Result<(), anyhow::Error> {
        Ok(())
    }
}