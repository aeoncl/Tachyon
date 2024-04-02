use std::{fmt::{self, Debug}, str::{from_utf8, FromStr}};

use anyhow::anyhow;
use log::{debug, info};

use crate::shared::command::command::split_raw_command_no_arg;
use crate::shared::traits::SerializeMsnp;

use super::error::{CommandError, PayloadError};

pub struct RawCommandParser {
    incomplete_command: Option<RawCommand>
} 

impl RawCommandParser {

    pub fn new() -> Self {
        RawCommandParser { incomplete_command: None }
    }

    pub fn parse_message(&mut self, message: &[u8]) -> Result<Vec<RawCommand>, CommandError> {
        let mut out: Vec<RawCommand> = Vec::new();

        let mut bytes_to_handle = message;


        //Handle previous chunks
        if let Some(mut incomplete) = self.incomplete_command.take() {
            let remaining_bytes = incomplete.get_missing_bytes_count();

            info!("previous message was chunked!");

            if message.len() >= remaining_bytes {
                //incomplete will be complete
                info!("no longer chunked!");

                incomplete.extend_payload_from_slice(&message[..remaining_bytes])?;
                out.push(incomplete);

                bytes_to_handle = &bytes_to_handle[remaining_bytes..message.len()];
            } else {
                //still not complete
                info!("still chunked!");
            
                incomplete.payload.extend_from_slice(&message[..message.len()]);
                self.incomplete_command = Some(incomplete);
                return Ok(out);
            }
        }


        while bytes_to_handle.len() >= 5 {
            //If it's bigger than CMD\r\n

            let operand = from_utf8(&bytes_to_handle[0..3]);

            if operand.is_err() {
                info!("Skipping bad operand");
                break;
            }

            let operand = operand.expect("to never fail");

            if operand.is_ascii() && operand.chars().all(|c| c.is_ascii_uppercase()) {
                let terminators_index = bytes_to_handle.windows(2).enumerate().find_map(|(index, content)| {
                    if content[0] as char == '\r' && content[1] as char == '\n' {
                        Some(index+1)
                    } else {
                        None
                    }
                });

                if terminators_index.is_none() {
                    break;
                }
    
                let terminators_index = terminators_index.expect("to never fail");

                let command = from_utf8(&bytes_to_handle[0..=terminators_index-2])?;

                let mut raw_command = RawCommand::from_str(command)?;

                let payload_size = raw_command.get_expected_payload_size();

                let after_term_index = terminators_index+1;

                if payload_size == 0 {
                    bytes_to_handle=&bytes_to_handle[terminators_index+1..bytes_to_handle.len()];
                    out.push(raw_command);
                } else {
                    //We have payload
                    if payload_size > bytes_to_handle.len(){
                        //Chunked
                        let payload = &bytes_to_handle[after_term_index..bytes_to_handle.len()];

                        if !payload.is_empty() {
                            raw_command.extend_payload_from_slice(payload)?;

                        }
                        self.incomplete_command = Some(raw_command);
                        break;
                    } else {
                        //Not chunked
                        let payload = &bytes_to_handle[after_term_index..after_term_index+payload_size];
                        raw_command.extend_payload_from_slice(payload)?;
                        bytes_to_handle=&bytes_to_handle[after_term_index+payload_size..bytes_to_handle.len()];
                        out.push(raw_command);
                    }
                }


            }
        }
        return Ok(out);

    }

    // pub fn parse_message(&mut self, message: &str) -> Result<Vec<RawCommand>, CommandError> {

    //     let mut bytes_to_handle = message;
    //     let mut out: Vec<RawCommand> = Vec::new();

    //     //Handle previous chunks
    //     if let Some(mut incomplete) = self.incomplete_command.pop() {
    //         let remaining_bytes = incomplete.get_missing_bytes_count();

    //         info!("previous message was chunked!");

    //         if message.len() >= remaining_bytes {
    //             //incomplete will be complete
    //             info!("no longer chunked!");

    //             incomplete.payload.extend_from_slice(&message[..remaining_bytes].as_bytes());
    //             out.push(incomplete);
    //             bytes_to_handle = &bytes_to_handle[remaining_bytes..message.len()];
    //         } else {
    //             //still not complete
    //             info!("still chunked!");

    //             incomplete.payload.extend_from_slice(&message[..message.len()].as_bytes());
    //             self.incomplete_command.push(incomplete);
    //             return Ok(out);
    //         }
    //     }

    //     //handle message content

    //     let mut maybe_cap = COMMAND_REGEX.captures(bytes_to_handle);

    //     while let Some(ref mut cap) = maybe_cap {
    //         let mut offset: usize = 0;
    //         let mut command = RawCommand::new(
    //             cap[0][0..cap[0].len() - 2].to_string(),
    //             cap[1].to_string()
    //         );

    //         offset += cap[0].len();

            

    //         let mut payload_size: usize = self.extract_expected_payload_size(&mut command)?;
    //         if offset + payload_size > bytes_to_handle.len() {
    //             //If the payload size is bigger than what we have, don't go past our buffer. Payload is chunked.
    //             payload_size = bytes_to_handle.len() - offset;
    //         }

    //         if payload_size > 0 {
    //             let payload = bytes_to_handle[offset..offset + payload_size].as_bytes().to_vec();
    //             offset += payload.len();
    //             command.payload = payload;
    //         }

    //         if command.is_complete() {
    //             out.push(command);
    //         } else {
    //             info!("message was chunked: {}", &message);
    //             self.incomplete_command.push(command);
    //             break;
    //         }

    //         bytes_to_handle = &bytes_to_handle[offset..bytes_to_handle.len()];
    //         maybe_cap = COMMAND_REGEX.captures(bytes_to_handle);

    //     }

   //     Ok(out)
   // }

   

}

 fn extract_expected_payload_size(split: &[&str]) -> Result<usize, CommandError> {
        if !is_payload_command(split[0]) {
            return Ok(0);
        }

        let expected_payload_size = match split.last() {
            Some(last) => {
                last.parse::<usize>().map_err(|e| CommandError::MalformedPayloadCommand { source: e.into() })?
            },
            _ => {
                return Err(CommandError::MalformedPayloadCommand { source: anyhow!("Payload command did not contain any arguments") });
            }
        };

        Ok(expected_payload_size)
    }

fn is_payload_command(operand: &str) -> bool {
    matches!(operand, "ADL" | "RML" | "UUX" | "UUN" | "MSG")
}

impl FromStr for RawCommand {
    type Err = CommandError;

    fn from_str(command: &str) -> Result<Self, Self::Err> {
        let command_split = split_raw_command_no_arg(command);
        let payload_size: usize = extract_expected_payload_size(command_split.as_slice())?;

        Ok(RawCommand {
            command: command.to_string(),
            command_split: command_split.iter().map(|e| e.to_string()).collect(),
            payload: Vec::with_capacity(payload_size),
            expected_payload_size: payload_size,
        })
    }
}

pub struct RawCommand {
    command: String,
    command_split: Vec<String>,
    expected_payload_size: usize,
    pub payload: Vec<u8>
}

impl RawCommand {
    pub fn without_payload(command: &str) -> Result<Self, CommandError> {
        Self::from_str(command)
    }

    pub fn with_payload(command: &str, payload: Vec<u8>) -> Result<Self, CommandError> {
        let mut command = Self::from_str(command)?;
        
        if command.expected_payload_size == payload.len() {
            command.payload = payload;
            Ok(command)
        } else {
            Err(CommandError::PayloadError(PayloadError::PayloadBytesMissing))
        }
    }

    pub fn get_operand(&self) -> &str {
        &self.command_split[0]
    }

    pub fn get_command(&self) -> &str {
        &self.command
    }

    pub fn get_command_split(&self) -> Vec<&str> {
        self.command_split.iter().map(|e| e.as_str()).collect()
    }

    pub fn get_payload(&self) -> &[u8] {
        self.payload.as_slice()
    }

    pub fn get_expected_payload_size(&self) -> usize {
        self.expected_payload_size
    }

    pub fn is_complete(&self) -> bool {
        self.expected_payload_size == self.payload.len()
    }

    pub fn get_missing_bytes_count(&self) -> usize {
        self.expected_payload_size - self.payload.len()
    }

    pub fn extend_payload_from_slice(&mut self, slice: &[u8]) -> Result<(), PayloadError> {
        debug!("capacity: {} - future size: {}", self.payload.capacity(), self.payload.len() + slice.len());
        if self.payload.len() + slice.len() > self.payload.capacity() {
            Err(PayloadError::PayloadSizeExceed { expected_size: self.expected_payload_size, overflowing_size: self.payload.len() + slice.len(), payload: self.payload.clone() })
        } else {
            self.payload.extend_from_slice(slice);
            Ok(())
        }

    }

}


impl Debug for RawCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.payload.is_empty() {
            write!(f, "{:?}", self.command)
        } else {
            write!(f, "{:?} | {:?}", self.command, self.payload)
        }
    }
}

impl SerializeMsnp for RawCommand {
    fn serialize_msnp(&self) -> Vec<u8> {
        let cmd = if self.expected_payload_size > 0 {
            format!("{} {}\r\n", &self.command, self.expected_payload_size)
        } else {
            format!("{}\r\n", &self.command)
        };
        let mut out = Vec::with_capacity(cmd.len() + self.payload.len());

        out.extend_from_slice(cmd.as_bytes());
        out.extend_from_slice(self.payload.as_slice());

        return out;
    }
}


#[cfg(test)]
mod tests {
    use std::str::from_utf8;

    use crate::msnp::raw_command_parser::RawCommandParser;

    #[test]
    fn test_one_simple_command_old() {
        //Arrange
        let mut parser = RawCommandParser::new();
        let command = String::from("TST 1 TST\r\n");

        //Act
        let parsed = parser.parse_message(command.as_bytes()).unwrap();

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].command, "TST 1 TST");
        assert_eq!(parsed[0].get_operand(), "TST");
    }

    #[test]
    fn test_two_simple_command_old() {
        //Arrange
        let mut parser = RawCommandParser::new();
        let command = String::from("TST 1 TST\r\nMOV 4 WOOWOO\r\n");

        //Act
        let parsed = parser.parse_message(command.as_bytes()).unwrap();

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].command, "TST 1 TST");
        assert_eq!(parsed[0].get_operand(), "TST");

        assert_eq!(parsed[1].command, "MOV 4 WOOWOO");
        assert_eq!(parsed[1].get_operand(), "MOV");
    }

    #[test]
    fn test_payload_command_old() {
        //Arrange
        let mut parser = RawCommandParser::new();
        let command = String::from("ADL 6 15\r\n<ml l=\"1\"></ml>");

        //Act
        let parsed = parser.parse_message(command.as_bytes()).unwrap();

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].command, "ADL 6 15");
        assert_eq!(parsed[0].get_operand(), "ADL");
        assert_eq!(from_utf8(&parsed[0].payload).unwrap(), "<ml l=\"1\"></ml>");
    }

    #[test]
    fn test_malformed_payload_command() {
        //Arrange
        let mut parser = RawCommandParser::new();
        let command = String::from("ADL 6 sdfdasdf\r\n<ml l=\"1\"></ml>");

        //Act
        let parsed = parser.parse_message(command.as_bytes());
        
        assert!(parsed.is_err());
    }

    #[test]
    fn test_payload_command2_old() {
        //Arrange
        let mut parser = RawCommandParser::new();
        let command = String::from("MOV 4 WOOWOO\r\nADL 6 15\r\n<ml l=\"1\"></ml>TST 1 TST\r\n");

        //Act
        let parsed = parser.parse_message(command.as_bytes()).unwrap();

        assert_eq!(parsed.len(), 3);

        assert_eq!(parsed[0].command, "MOV 4 WOOWOO");
        assert_eq!(parsed[0].get_operand(),"MOV");

        assert_eq!(parsed[1].command, "ADL 6 15");
        assert_eq!(parsed[1].get_operand(), "ADL");
        assert_eq!(from_utf8(&parsed[1].payload).unwrap(), "<ml l=\"1\"></ml>");

        assert_eq!(parsed[2].command, "TST 1 TST");
        assert_eq!(parsed[2].get_operand(), "TST");
    }

    #[test]
    fn test_payload_contains_psm() {
        let mut parser = RawCommandParser::new();
        let commands = String::from("BLP 9 AL\r\nUUX 10 224\r\n<Data><PSM>Hi my dude</PSM><CurrentMedia></CurrentMedia><MachineGuid>&#x7B;F52973B6-C926-4BAD-9BA8-7C1E840E4AB0&#x7D;</MachineGuid><DDP></DDP><SignatureSound></SignatureSound><Scene></Scene><ColorScheme></ColorScheme></Data>CHG 11 NLN 2789003324:48 0\r\n");

        let parsed = parser.parse_message(commands.as_bytes()).unwrap();

        assert_eq!(parsed.len(), 3);

        assert_eq!(parsed[0].command, "BLP 9 AL");
        assert_eq!(parsed[0].get_operand(), "BLP");

        assert_eq!(parsed[1].command, "UUX 10 224");
        assert_eq!(parsed[1].get_operand(), "UUX");
        assert_eq!(from_utf8(&parsed[1].payload).unwrap(), "<Data><PSM>Hi my dude</PSM><CurrentMedia></CurrentMedia><MachineGuid>&#x7B;F52973B6-C926-4BAD-9BA8-7C1E840E4AB0&#x7D;</MachineGuid><DDP></DDP><SignatureSound></SignatureSound><Scene></Scene><ColorScheme></ColorScheme></Data>");

        assert_eq!(parsed[2].command, "CHG 11 NLN 2789003324:48 0");
        assert_eq!(parsed[2].get_operand(), "CHG");
    }

    #[test]
    fn test_chunked() {
      //  let command = String::from("MOV 4 WOOWOO\r\nADL 6 15\r\n");
      let mut parser = RawCommandParser::new();
      let command = String::from("MOV 4 WOOWOO\r\nADL 6 15\r\n");
        let chunked_payload = String::from("<ml l=\"1\"></ml>MOV 5 WEEWOO\r\n");

        let parsed = parser.parse_message(command.as_bytes()).unwrap();

        assert!(parsed.len() == 1);


        let mut parsed = parser.parse_message(chunked_payload.as_bytes()).unwrap();

        let payload_command = parsed.pop().unwrap();
        assert!(payload_command.is_complete());
    }

    #[test]
    fn test_weird_chunk_bug() {
        env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

        let mut parser = RawCommandParser::new();

        let command = String::from("ANS 9 aeontest4@shlasouf.local;{F52973B6-C926-4BAD-9BA8-7C1E840E4AB0} IWlIc1N6VHNzZXh6ZWVWV1pjVDpzaGxhc291Zi5sb2NhbDtzeXRfWVdWdmJuUmxjM1EwX09xUklRQktRd0ZFRU1aSE5KY2JiXzBCQjJzcjtAYWVvbnRlc3QzOnNobGFzb3VmLmxvY2Fs 15800445832891040610\r\n");
   
        let parsed = parser.parse_message(command.as_bytes()).unwrap();

        assert_eq!(1, parsed.len());
        assert_eq!("ANS", parsed[0].get_operand());
        assert_eq!(5, parsed[0].get_command_split().len());
        assert_eq!(0, parsed[0].expected_payload_size);
    }

    #[test]
    fn test_utf8() {
        let mut parser = RawCommandParser::new();

        let payload = "MIME-Version: 1.0\r\nContent-Type: text/plain; charset=UTF-8\r\nX-MMS-IM-Format: FN=Segoe%20UI; EF=; CO=0; CS=1; PF=0\r\n\r\nö";
        let first_message = format!("MSG 1 U {payload_size}\r\n{payload}", payload_size = payload.len(), payload = payload);


        let mut parsed = parser.parse_message(first_message.as_bytes()).unwrap();
          let payload_command = parsed.pop().unwrap();

          assert_eq!(&payload_command.payload,payload.as_bytes());

          println!("size in message: {}, size with len(): {}", payload_command.get_expected_payload_size(), payload_command.payload.len());
          assert!(payload_command.is_complete() == true);
    }
}
