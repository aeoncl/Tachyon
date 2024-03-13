
pub trait ProtocolVersion {

    fn is_payload_command(&self, operand: &str) -> bool;

}