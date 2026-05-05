
pub trait OutgoingMessageService {

    fn send_message(&self);

}

struct OutgoingMessageServiceImpl {

}

impl OutgoingMessageService for OutgoingMessageServiceImpl {
    fn send_message(&self) {
        todo!()
    }
}

