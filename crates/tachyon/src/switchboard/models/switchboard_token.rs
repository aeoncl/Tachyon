use anyhow::anyhow;
use base64::engine::general_purpose;
use base64::Engine;
use msnp::msnp::error::CommandError;
use msnp::shared::models::ticket_token::TicketToken;
use std::str::FromStr;
use matrix_sdk::ruma::OwnedRoomId;

#[derive(Clone)]
pub struct SwitchboardToken {
    pub room_id: OwnedRoomId,
    pub matrix_token: String,
}

impl SwitchboardToken {
    pub fn new(room_id: OwnedRoomId, matrix_token: String) -> Self {
        Self {
            room_id,
            matrix_token,
        }
    }
}

impl Into<TicketToken> for SwitchboardToken {
    fn into(self) -> TicketToken {
        let csv = format!("{};{}", self.room_id, self.matrix_token);
        TicketToken(general_purpose::STANDARD.encode(csv))
    }
}

impl TryFrom<TicketToken> for SwitchboardToken {
    type Error = CommandError;

    fn try_from(value: TicketToken) -> Result<Self, Self::Error> {
        let decoded_bytes = general_purpose::STANDARD.decode(value.0)
            .map_err(|e| CommandError::ArgumentParseError {
            argument: "ticket_token".to_string(),
            command: "".to_string(),
            source: anyhow!(e),
        })?;

        let csv = String::from_utf8(decoded_bytes).map_err(|e| CommandError::ArgumentParseError {
            argument: "ticket_token".to_string(),
            command: "".to_string(),
            source: anyhow!(e),
        })?;
        
        let split: Vec<&str> = csv.split(";").collect();
        
        if split.len() != 2 {
            return Err(CommandError::ArgumentParseError {
                argument: "ticket_token".to_string(),
                command: "".to_string(),
                source: anyhow!("Decoded ticket token csv contained unexpected count of parts: {}", &csv),
            });
        }

        let raw_room_id = split.get(0).expect("to be here");
        let room_id = OwnedRoomId::from_str(raw_room_id).map_err(|e| CommandError::ArgumentParseError {
            argument: "ticket_token".to_string(),
            command: "".to_string(),
            source: anyhow!("Failed to parse room id from ticket token: {}", raw_room_id),
        })?;

        let matrix_token =  split.get(1).expect("to be here").to_string();

        Ok(Self{ room_id, matrix_token })
    }
}