use crate::tachyon::client::tachyon_client::{TachyonClient, TachyonSessionData};

pub trait OutgoingMessagingPortal {

    fn send_message(&self);

}

impl OutgoingMessagingPortal for TachyonClient {
    fn send_message(&self) {
        todo!()
    }
}