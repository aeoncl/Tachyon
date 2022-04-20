use std::fmt;

use lazy_static::lazy_static;
use regex::Regex;
use substring::Substring;
pub struct MSNPCommandParser {}

impl MSNPCommandParser {
    pub fn parse_message(message: &String) -> Vec<MSNPCommand> {
        let mut out: Vec<MSNPCommand> = Vec::new();

        lazy_static! {
            static ref NORMAL_COMMAND: Regex = Regex::new(r"([A-Z]{3}).*[\r\n]").unwrap();
        }

        let mut current_slice = message.as_str();
        let mut maybe_cap = NORMAL_COMMAND.captures(current_slice);
        {
            while let Some(ref mut cap) = maybe_cap {
                let mut offset: usize = 0;

                let mut command = MSNPCommand::new(
                    cap[0].substring(0, cap[0].len() - 2).to_string(),
                    cap[1].to_string(),
                    String::new(),
                );

                offset += cap[0].len();

                let payload_size: usize = command.get_payload_size().try_into().unwrap();
                if payload_size > 0 {
                    let payload = current_slice.to_string().substring(offset, offset + payload_size).to_string();
                    command.payload = payload;
                    offset += payload_size;
                }

                out.push(command);

                current_slice = &current_slice[offset..current_slice.len()];
                maybe_cap = NORMAL_COMMAND.captures(current_slice);
            }
            return out;
        }
    }
}
#[derive(Clone)]
pub struct MSNPCommand {
    pub command: String,
    pub payload: String,
    pub operand: String,
}

impl MSNPCommand {
    pub fn new(command: String, operand: String, payload: String) -> MSNPCommand {
        return MSNPCommand {
            command: command,
            operand: operand,
            payload: payload,
        };
    }

    pub fn split(&self) -> Vec<&str> {
        return self.command.split_whitespace().collect::<Vec<&str>>();
    }

    pub fn get_payload_size(&self) -> i32 {
        let split = self.split();
        let last = split.last().unwrap();
        return last.parse::<i32>().unwrap_or_default();
    }

    pub fn is_complete(&self) -> bool {
        return self.get_payload_size() as usize == self.payload.chars().count();
    }
}

impl fmt::Display for MSNPCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.payload.is_empty() {
            return write!(f, "{}", self.command);
        } else {
            return write!(f, "{} | {}", self.command, self.payload);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::sockets::msnp_command::MSNPCommandParser;

    #[test]
    fn test_one_simple_command_old() {
        //Arrange
        let command = String::from("TST 1 TST\r\n");

        //Act
        let parsed = MSNPCommandParser::parse_message(&command);

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].command, "TST 1 TST");
        assert_eq!(parsed[0].operand, "TST");
    }

    #[test]
    fn test_two_simple_command_old() {
        //Arrange
        let command = String::from("TST 1 TST\r\nMOV 4 WOOWOO\r\n");

        //Act
        let parsed = MSNPCommandParser::parse_message(&command);

        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].command, "TST 1 TST");
        assert_eq!(parsed[0].operand, "TST");

        assert_eq!(parsed[1].command, "MOV 4 WOOWOO");
        assert_eq!(parsed[1].operand, "MOV");
    }

    #[test]
    fn test_payload_command_old() {
        //Arrange
        let command = String::from("ADL 6 15\r\n<ml l=\"1\"></ml>");

        //Act
        let parsed = MSNPCommandParser::parse_message(&command);

        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].command, "ADL 6 15");
        assert_eq!(parsed[0].operand, "ADL");
        assert_eq!(parsed[0].payload, "<ml l=\"1\"></ml>");
    }

    #[test]
    fn test_payload_command2_old() {
        //Arrange
        let command = String::from("MOV 4 WOOWOO\r\nADL 6 15\r\n<ml l=\"1\"></ml>TST 1 TST\r\n");

        //Act
        let parsed = MSNPCommandParser::parse_message(&command);

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
        let commands = String::from("BLP 9 AL\r\nUUX 10 224\r\n<Data><PSM>Hi my dude</PSM><CurrentMedia></CurrentMedia><MachineGuid>&#x7B;F52973B6-C926-4BAD-9BA8-7C1E840E4AB0&#x7D;</MachineGuid><DDP></DDP><SignatureSound></SignatureSound><Scene></Scene><ColorScheme></ColorScheme></Data>CHG 11 NLN 2789003324:48 0\r\n");

        let parsed = MSNPCommandParser::parse_message(&commands);

        assert_eq!(parsed.len(), 3);

        assert_eq!(parsed[0].command, "BLP 9 AL");
        assert_eq!(parsed[0].operand, "BLP");

        assert_eq!(parsed[1].command, "UUX 10 224");
        assert_eq!(parsed[1].operand, "UUX");
        assert_eq!(parsed[1].payload, "<Data><PSM>Hi my dude</PSM><CurrentMedia></CurrentMedia><MachineGuid>&#x7B;F52973B6-C926-4BAD-9BA8-7C1E840E4AB0&#x7D;</MachineGuid><DDP></DDP><SignatureSound></SignatureSound><Scene></Scene><ColorScheme></ColorScheme></Data>");

        assert_eq!(parsed[2].command, "CHG 11 NLN 2789003324:48 0");
        assert_eq!(parsed[2].operand, "CHG");
    }
}
