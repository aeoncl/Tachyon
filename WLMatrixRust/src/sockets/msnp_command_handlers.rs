use substring::Substring;

use super::msnp_command::MSNPCommand;



struct NotificationCommandHandler {
    protocol_version: i32
}

struct SwitchboardCommandHandler {
    protocol_version: i32
}

trait CommandHandler {
    fn new() -> Self;
    fn handle_command(&mut self, command: MSNPCommand) -> String; 
}

impl CommandHandler for NotificationCommandHandler {
    
    fn new() -> NotificationCommandHandler {
        return NotificationCommandHandler { protocol_version: -1 };
    }

    fn handle_command(&mut self, command: MSNPCommand) -> String {
        let split = command.split();
        match command.operand.as_str() {
            "VER" => {
                // 0  1    2      3     4
                //=>VER 1 MSNP18 MSNP17 CVR0\r\n
                let ver : i32 = split[2].substring(4, split[2].len()).parse::<i32>().unwrap();
                self.protocol_version = ver;
                //<=VER 1 MSNP18\r\n
                return format!("VER {} MSNP{}\r\n", split[1], ver);
            }
            _=> {
                return String::new();
            }
        }
    }
}

impl CommandHandler for SwitchboardCommandHandler {
    
    fn new() -> SwitchboardCommandHandler {
        return SwitchboardCommandHandler { protocol_version: -1 };
    }

    fn handle_command(&mut self, command: MSNPCommand) -> String{
        todo!()
    }
}


#[cfg(test)]
mod tests {
    use crate::sockets::msnp_command::MSNPCommandParser;
    use crate::sockets::msnp_command_handlers::{NotificationCommandHandler, CommandHandler};


    #[test]
    fn test_ver_command() {

        //Arrange
        let command = String::from("VER 1 MSNP18 MSNP17 CVR0\r\n");
        let parsed = MSNPCommandParser::parse_message(command.clone());
        let mut handler = NotificationCommandHandler::new();

        //Act
        let result = handler.handle_command(parsed[0].clone());

        //Assert
        assert_eq!(result,"VER 1 MSNP18\r\n");

    }
}