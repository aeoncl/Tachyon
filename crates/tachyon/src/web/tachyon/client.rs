use crate::tachyon::state::session::tachyon_client_repository::TachyonSessionData;
use matrix_sdk::Client;

trait AdminWebClient {
    fn matrix_client(&self) -> matrix_sdk::Client;
}

impl AdminWebClient for TachyonSessionData {
    fn matrix_client(&self) -> Client {
        self.session_data.matrix_client.clone()
    }
}
