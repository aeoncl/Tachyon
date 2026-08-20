use crate::matrix::services::sync::{MatrixSyncService, MatrixSyncServiceImpl};
use crate::tachyon::client::tachyon_client::TachyonClient;

impl TachyonClient {

    pub fn sync_service(&self) -> Box<dyn MatrixSyncService> {
        Box::new(MatrixSyncServiceImpl::new(self.matrix_client(), self.clone()))
    }

}