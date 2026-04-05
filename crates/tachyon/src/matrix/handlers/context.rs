use crate::tachyon::client::tachyon_client::TachyonClient;

#[derive(Clone)]
pub struct TachyonContext {
    pub tachyon_client: TachyonClient
}
