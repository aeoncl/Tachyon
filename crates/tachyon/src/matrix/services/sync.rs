use matrix_sdk::{async_trait, Client};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::task::JoinHandle;
use crate::tachyon::client::tachyon_client::TachyonClient;

#[async_trait]
pub trait MatrixSyncService: Send + Sync  {
    async fn start_sync(&self, kill_signal_snd: Sender<()>, kill_signal_rcv: Receiver<()>) -> JoinHandle<()>;

}

pub struct MatrixSyncServiceImpl {
    matrix_client: Client,
    tachyon_client: TachyonClient
}

impl MatrixSyncServiceImpl {

    pub fn new(matrix_client: Client, tachyon_client: TachyonClient) -> Self {
        Self {
            matrix_client,
            tachyon_client
        }
    }

}

#[async_trait]
impl MatrixSyncService for MatrixSyncServiceImpl {
    async fn start_sync(&self, kill_signal_snd: Sender<()>, kill_signal_rcv: Receiver<()>) -> JoinHandle<()> {
        crate::matrix::sync::sync(self.tachyon_client.clone(), self.matrix_client.clone(), kill_signal_snd, kill_signal_rcv).await
    }
}