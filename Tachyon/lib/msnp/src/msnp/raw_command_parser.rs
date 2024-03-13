use std::{any, fmt::{self, Debug}};

use anyhow::anyhow;
use lazy_static::lazy_static;
use log::info;
use regex::Regex;

use super::{error::CommandError, protocol::ProtocolVersion};

lazy_static! {
    static ref COMMAND_REGEX: Regex = Regex::new(r"([A-Z]{3}).*[\r\n]").unwrap();
}

pub struct RawCommandParser<T> {
    protocol_version: T,
    incomplete_command: Vec<RawCommand>
}

impl<T: ProtocolVersion> RawCommandParser<T> {

    pub fn new(protocol_version: T) -> Self {
        RawCommandParser { incomplete_command: Vec::new(), protocol_version }
    }

    pub fn parse_message(&mut self, message: &str) -> Result<Vec<RawCommand>, CommandError> {

        let mut bytes_to_handle = message;
        let mut out: Vec<RawCommand> = Vec::new();

        //Handle previous chunks
        if let Some(mut incomplete) = self.incomplete_command.pop() {
            let remaining_bytes = incomplete.get_missing_bytes_count();

            info!("previous message was chunked!");

            if message.len() >= remaining_bytes {
                //incomplete will be complete
                info!("no longer chunked!");

                incomplete.payload.push_str(&message[..remaining_bytes]);
                out.push(incomplete);
                bytes_to_handle = &bytes_to_handle[remaining_bytes..message.len()];
            } else {
                //still not complete
                info!("still chunked!");

                incomplete.payload.push_str(&message[..message.len()]);
                self.incomplete_command.push(incomplete);
                return Ok(out);
            }
        }

        //handle message content

        let mut maybe_cap = COMMAND_REGEX.captures(bytes_to_handle);

        while let Some(ref mut cap) = maybe_cap {
            let mut offset: usize = 0;
            let mut command = RawCommand::new(
                cap[0][0..cap[0].len() - 2].to_string(),
                cap[1].to_string()
            );

            offset += cap[0].len();

            

            let mut payload_size: usize = self.extract_expected_payload_size(&mut command)?;
            if offset + payload_size > bytes_to_handle.len() {
                //If the payload size is bigger than what we have, don't go past our buffer. Payload is chunked.
                payload_size = bytes_to_handle.len() - offset;
            }

            if payload_size > 0 {
                let payload = bytes_to_handle[offset..offset + payload_size].to_string();
                offset += payload.len();
                command.payload = payload;
            }

            if command.is_complete() {
                out.push(command);
            } else {
                info!("message was chunked: {}", &message);
                self.incomplete_command.push(command);
                break;
            }

            bytes_to_handle = &bytes_to_handle[offset..bytes_to_handle.len()];
            maybe_cap = COMMAND_REGEX.captures(bytes_to_handle);

        }

        Ok(out)
    }

    fn extract_expected_payload_size(&self, command: &mut RawCommand) -> Result<usize, CommandError> {
        if !self.protocol_version.is_payload_command(&command.operand) {
            return Ok(0);
        }

        let split = command.split();

        let expected_payload_size = match split.last() {
            Some(last) => {
                last.parse::<usize>().map_err(|e| CommandError::MalformedPayloadCommand { source: e.into() })?
            },
            _ => {
                return Err(CommandError::MalformedPayloadCommand { source: anyhow!("Payload command did not contain any arguments") });
            }
        };

        command.expected_payload_size = expected_payload_size;

        Ok(expected_payload_size)

    }

}

pub struct RawCommand {
    pub command: String,
    pub operand: String,
    pub payload: String,
    pub expected_payload_size: usize
}

impl RawCommand {
    pub fn new(command: String, operand: String) -> RawCommand {
        RawCommand {
            command,
            operand,
            payload: String::new(),
            expected_payload_size: 0
        }
    }

    pub fn split(&self) -> Vec<&str> {
        return self.command.split_whitespace().collect::<Vec<&str>>();
    }

    pub fn is_complete(&self) -> bool {
        self.expected_payload_size == self.payload.len()
    }

    pub fn get_missing_bytes_count(&self) -> usize {
        self.expected_payload_size - self.payload.len()
    }
}


impl Debug for RawCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.payload.is_empty() {
            write!(f, "{}", self.command)
        } else {
            write!(f, "{} | {}", self.command, self.payload)
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::msnp::{raw_command_parser::RawCommandParser, msnp18::protocol::MSNP18};


    #[test]
    fn test_one_simple_command_old() {
        //Arrange
        let mut parser = RawCommandParser::new(MSNP18::new());
        let command = String::from("TST 1 TST\r\n");

        //Act
        let parsed = parser.parse_message(command.as_str()).unwrap();

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].command, "TST 1 TST");
        assert_eq!(parsed[0].operand, "TST");
    }

    #[test]
    fn test_two_simple_command_old() {
        //Arrange
        let mut parser = RawCommandParser::new(MSNP18::new());
        let command = String::from("TST 1 TST\r\nMOV 4 WOOWOO\r\n");

        //Act
        let parsed = parser.parse_message(command.as_str()).unwrap();

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].command, "TST 1 TST");
        assert_eq!(parsed[0].operand, "TST");

        assert_eq!(parsed[1].command, "MOV 4 WOOWOO");
        assert_eq!(parsed[1].operand, "MOV");
    }

    #[test]
    fn test_payload_command_old() {
        //Arrange
        let mut parser = RawCommandParser::new(MSNP18::new());
        let command = String::from("ADL 6 15\r\n<ml l=\"1\"></ml>");

        //Act
        let parsed = parser.parse_message(command.as_str()).unwrap();

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].command, "ADL 6 15");
        assert_eq!(parsed[0].operand, "ADL");
        assert_eq!(parsed[0].payload, "<ml l=\"1\"></ml>");
    }

    #[test]
    fn test_malformed_payload_command() {
        //Arrange
        let mut parser = RawCommandParser::new(MSNP18::new());
        let command = String::from("ADL 6 sdfdasdf\r\n<ml l=\"1\"></ml>");

        //Act
        let parsed = parser.parse_message(command.as_str());
        
        assert!(parsed.is_err());
    }

    #[test]
    fn test_payload_command2_old() {
        //Arrange
        let mut parser = RawCommandParser::new(MSNP18::new());
        let command = String::from("MOV 4 WOOWOO\r\nADL 6 15\r\n<ml l=\"1\"></ml>TST 1 TST\r\n");

        //Act
        let parsed = parser.parse_message(command.as_str()).unwrap();

        assert_eq!(parsed.len(), 3);

        assert_eq!(parsed[0].command, "MOV 4 WOOWOO");
        assert_eq!(parsed[0].operand, "MOV");

        assert_eq!(parsed[1].command, "ADL 6 15");
        assert_eq!(parsed[1].operand, "ADL");
        assert_eq!(parsed[1].payload, "<ml l=\"1\"></ml>");

        assert_eq!(parsed[2].command, "TST 1 TST");
        assert_eq!(parsed[2].operand, "TST");
    }

    #[test]
    fn test_payload_contains_psm() {
        let mut parser = RawCommandParser::new(MSNP18::new());
        let commands = String::from("BLP 9 AL\r\nUUX 10 224\r\n<Data><PSM>Hi my dude</PSM><CurrentMedia></CurrentMedia><MachineGuid>&#x7B;F52973B6-C926-4BAD-9BA8-7C1E840E4AB0&#x7D;</MachineGuid><DDP></DDP><SignatureSound></SignatureSound><Scene></Scene><ColorScheme></ColorScheme></Data>CHG 11 NLN 2789003324:48 0\r\n");

        let parsed = parser.parse_message(commands.as_str()).unwrap();

        assert_eq!(parsed.len(), 3);

        assert_eq!(parsed[0].command, "BLP 9 AL");
        assert_eq!(parsed[0].operand, "BLP");

        assert_eq!(parsed[1].command, "UUX 10 224");
        assert_eq!(parsed[1].operand, "UUX");
        assert_eq!(parsed[1].payload, "<Data><PSM>Hi my dude</PSM><CurrentMedia></CurrentMedia><MachineGuid>&#x7B;F52973B6-C926-4BAD-9BA8-7C1E840E4AB0&#x7D;</MachineGuid><DDP></DDP><SignatureSound></SignatureSound><Scene></Scene><ColorScheme></ColorScheme></Data>");

        assert_eq!(parsed[2].command, "CHG 11 NLN 2789003324:48 0");
        assert_eq!(parsed[2].operand, "CHG");
    }

    #[test]
    fn test_chunked() {
      //  let command = String::from("MOV 4 WOOWOO\r\nADL 6 15\r\n");
      let mut parser = RawCommandParser::new(MSNP18::new());
      let command = String::from("MOV 4 WOOWOO\r\nADL 6 15\r\n");
        let chunked_payload = String::from("<ml l=\"1\"></ml>MOV 5 WEEWOO\r\n");

        let parsed = parser.parse_message(command.as_str()).unwrap();

        assert!(parsed.len() == 1);


        let mut parsed = parser.parse_message(chunked_payload.as_str()).unwrap();

        let payload_command = parsed.pop().unwrap();
        assert!(payload_command.is_complete() == true);
    }

    #[test]
    fn test_weird_chunk_bug() {
        env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

        let mut parser = RawCommandParser::new(MSNP18::new());

        let command = String::from("ANS 9 aeontest4@shlasouf.local;{F52973B6-C926-4BAD-9BA8-7C1E840E4AB0} IWlIc1N6VHNzZXh6ZWVWV1pjVDpzaGxhc291Zi5sb2NhbDtzeXRfWVdWdmJuUmxjM1EwX09xUklRQktRd0ZFRU1aSE5KY2JiXzBCQjJzcjtAYWVvbnRlc3QzOnNobGFzb3VmLmxvY2Fs 15800445832891040610");
   
        let parsed = parser.parse_message(command.as_str()).unwrap();

    }

    #[test]
    fn test_utf8() {
        let mut parser = RawCommandParser::new(MSNP18::new());

        let payload = "MIME-Version: 1.0\r\nContent-Type: text/plain; charset=UTF-8\r\nX-MMS-IM-Format: FN=Segoe%20UI; EF=; CO=0; CS=1; PF=0\r\n\r\nö";
        let first_message = format!("MSG 1 U {payload_size}\r\n{payload}", payload_size = payload.len(), payload = payload);


        let mut parsed = parser.parse_message(first_message.as_str()).unwrap();
          let payload_command = parsed.pop().unwrap();

          assert_eq!(payload_command.payload,payload);

          println!("size in message: {}, size with len(): {}", payload_command.expected_payload_size, payload_command.payload.len());
          assert!(payload_command.is_complete() == true);
  
    }
}
