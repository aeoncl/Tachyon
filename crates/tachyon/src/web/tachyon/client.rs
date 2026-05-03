use matrix_sdk::Client;
use crate::tachyon::client::tachyon_client::TachyonClient;

trait AdminWebClient {

    fn matrix_client(&self) -> matrix_sdk::Client;

}

impl AdminWebClient for TachyonClient {
    fn matrix_client(&self) -> Client {
        self.session_data.matrix_client.clone()
    }
}