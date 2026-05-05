
pub trait OutgoingMessageService: Send + Sync {
    fn send_message(&self);
}

pub struct OutgoingMessageServiceImpl {}

impl OutgoingMessageService for OutgoingMessageServiceImpl {
    fn send_message(&self) {
        todo!()
    }
}