use crate::tachyon::tachyon_client::TachyonClient;

#[derive(Clone)]
pub struct TachyonContext {
    pub client_data: TachyonClient
}
