use std::fmt::Display;

pub struct OkCommand {
    pub operand: String,
    pub tr_id: u128
}

impl Display for OkCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{operand} {tr_id} OK\r\n", operand = self.operand, tr_id = self.tr_id)
    }
}