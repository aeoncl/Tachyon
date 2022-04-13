

use std::fmt;

use regex::Regex;
use lazy_static::lazy_static;

pub struct MSNPCommandParser {
}


impl MSNPCommandParser {
    pub fn parse_message(message: String) -> Vec<MSNPCommand> {
        let mut out: Vec<MSNPCommand> = Vec::new();
        lazy_static! {
            static ref NORMAL_COMMAND: Regex = Regex::new(r"([A-Z]{3}).*").unwrap();
        }

        for cap in NORMAL_COMMAND.captures_iter(&message) {
            let command = MSNPCommand::new(cap[0].to_string(), cap[1].to_string(), String::new());
            out.push(command);
        }

        return out;
    }

    pub fn parse_payload_message(message: String, empty_payload_command: MSNPCommand) -> Vec<MSNPCommand> {
        let payload_size = empty_payload_command.get_payload_size();
        let (payload, other_stuff) = message.split_at(payload_size.try_into().unwrap());
        let mut payload_command = empty_payload_command;
        
        payload_command.payload = payload.to_string();

        let mut other_commands = MSNPCommandParser::parse_message(other_stuff.to_string());
        other_commands.insert(0, payload_command);
        
        return other_commands;
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
        return write!(f, "MSNPCommand - command: {}, payload: {}", self.command, self.payload);
    }
}

#[cfg(test)]
mod tests {
    use crate::sockets::msnp_command::MSNPCommandParser;


    #[test]
    fn test_one_simple_command() {

        //Arrange
        let command = String::from("TST 1 TST");

        //Act
        let parsed = MSNPCommandParser::parse_message(command.clone());
        
        assert_eq!(parsed.len(),1);
        assert_eq!(parsed[0].command, "TST 1 TST");
        assert_eq!(parsed[0].operand, "TST");
        assert_eq!(parsed[0].get_payload_size(), 0);
    }

    #[test]
    fn test_payload_command_with_something_else() {

        //Arrange
        let command = String::from("ADL 6 15");
        let payload_and_other_stuff = String::from("<ml l=\"1\"></ml>TST 1 TST");

        let parsed_command = MSNPCommandParser::parse_message(command.clone());
        
        assert_eq!(parsed_command.len(),1);
        assert_eq!(parsed_command[0].command, "ADL 6 15");
        assert_eq!(parsed_command[0].operand, "ADL");
        assert_eq!(parsed_command[0].is_complete(), false);

        let parsed_payload_and_stuff = MSNPCommandParser::parse_payload_message(payload_and_other_stuff, parsed_command[0].clone());
        
        assert_eq!(parsed_payload_and_stuff.len(), 2);
        assert_eq!(parsed_payload_and_stuff[0].command, "ADL 6 15");
        assert_eq!(parsed_payload_and_stuff[0].operand, "ADL");
        assert_eq!(parsed_payload_and_stuff[0].payload, "<ml l=\"1\"></ml>");
        assert_eq!(parsed_payload_and_stuff[0].is_complete(), true);

        assert_eq!(parsed_payload_and_stuff[1].command, "TST 1 TST");
        assert_eq!(parsed_payload_and_stuff[1].operand, "TST");
        assert_eq!(parsed_payload_and_stuff[1].is_complete(), true);
    }

    #[test]
    fn test_payload_command() {

        //Arrange
        let command = String::from("ADL 6 15");
        let payload_and_other_stuff = String::from("<ml l=\"1\"></ml>");

        let parsed_command = MSNPCommandParser::parse_message(command.clone());
        
        assert_eq!(parsed_command.len(),1);
        assert_eq!(parsed_command[0].command, "ADL 6 15");
        assert_eq!(parsed_command[0].operand, "ADL");
        assert_eq!(parsed_command[0].is_complete(), false);

        let parsed_payload_and_stuff = MSNPCommandParser::parse_payload_message(payload_and_other_stuff, parsed_command[0].clone());
        
        assert_eq!(parsed_payload_and_stuff.len(), 1);
        assert_eq!(parsed_payload_and_stuff[0].command, "ADL 6 15");
        assert_eq!(parsed_payload_and_stuff[0].operand, "ADL");
        assert_eq!(parsed_payload_and_stuff[0].payload, "<ml l=\"1\"></ml>");
        assert_eq!(parsed_payload_and_stuff[0].is_complete(), true);
    }
}