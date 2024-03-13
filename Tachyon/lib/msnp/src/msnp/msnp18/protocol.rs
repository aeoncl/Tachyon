use crate::msnp::protocol::ProtocolVersion;

pub struct MSNP18;

impl MSNP18 {

    pub fn new() -> Self {
        Self
    }

}

impl ProtocolVersion for MSNP18 {
    fn is_payload_command(&self, operand: &str) -> bool {
        matches!(operand, "ADL" | "RML" | "UUX" | "UUN" | "MSG")
    }
}


