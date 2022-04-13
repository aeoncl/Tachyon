use substring::Substring;

use super::msnp_command::MSNPCommand;

struct NotificationCommandHandler {
    protocol_version: i32,
}

struct SwitchboardCommandHandler {
    protocol_version: i32,
}

trait CommandHandler {
    fn new() -> Self;
    fn handle_command(&mut self, command: MSNPCommand) -> String;
}

impl CommandHandler for NotificationCommandHandler {
    fn new() -> NotificationCommandHandler {
        return NotificationCommandHandler {
            protocol_version: -1,
        };
    }

    fn handle_command(&mut self, command: MSNPCommand) -> String {
        let split = command.split();
        match command.operand.as_str() {
            "VER" => {
                // 0  1    2      3     4
                //=>VER 1 MSNP18 MSNP17 CVR0\r\n
                let ver: i32 = split[2]
                    .substring(4, split[2].chars().count())
                    .parse::<i32>()
                    .unwrap();
                self.protocol_version = ver;
                //<=VER 1 MSNP18\r\n
                return format!("VER {} MSNP{}\r\n", split[1], ver);
            }
            "CVR" => {
                //    0  1    2     3     4    5      6          7          8          9
                //=> CVR 2 0x0409 winnt 6.0.0 i386 MSNMSGR 14.0.8117.0416 msmsgs login@email.com
                let _msn_login = split[9];
                let tr_id = split[1];
                let version = split[7];
                //<= CVR 2 14.0.8117.0416 14.0.8117.0416 14.0.8117.0416 localhost localhost
                return format!(
                    "CVR {tr_id} {version} {version} {version} {host} {host}\r\n",
                    tr_id = tr_id,
                    version = version,
                    host = "localhost"
                );
            }
            "USR" => {
                /*
                I phase :
                        0   1  2  3      4
                    >>> USR 3 SSO I login@test.com
                    <<< USR 3 SSO S MBI_KEY_OLD LAhAAUzdC+JvuB33nooLSa6Oh0oDFCbKrN57EVTY0Dmca8Reb3C1S1czlP12N8VU
                S phase :
                        0   1  2  3     4                    5
                    >>> USR 4 SSO S t=ssotoken {55192CF5-588E-4ABE-9CDF-395B616ED85B}
                    <<< USR 4 OK login@test.com 1 0
                */
                let tr_id = split[1];
                let auth_type = split[2];
                let phase = split[3];
                
                if auth_type == "SHA" {
                    return format!("USR {tr_id} OK {email} 1 0\r\n", tr_id=tr_id, email="TODO");
                } else if auth_type == "SSO" {
                    if phase == "I" {
                        let login = split[4];
                        //TODO
                    } else if phase == "S" {

                    }
                }

                return "TODO".to_string();
            }
            _ => {
                return String::new();
            }
        }
    }
}

impl CommandHandler for SwitchboardCommandHandler {
    fn new() -> SwitchboardCommandHandler {
        return SwitchboardCommandHandler {
            protocol_version: -1,
        };
    }

    fn handle_command(&mut self, command: MSNPCommand) -> String {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::sockets::msnp_command::MSNPCommandParser;
    use crate::sockets::msnp_command_handlers::{CommandHandler, NotificationCommandHandler};

    #[test]
    fn test_ver_command() {
        //Arrange
        let command = String::from("VER 1 MSNP18 MSNP17 CVR0\r\n");
        let parsed = MSNPCommandParser::parse_message(command.clone());
        let mut handler = NotificationCommandHandler::new();

        //Act
        let result = handler.handle_command(parsed[0].clone());

        //Assert
        assert_eq!(result, "VER 1 MSNP18\r\n");
    }

    #[test]
    fn test_cvr_command() {
        //Arrange
        let command = String::from(
            "CVR 2 0x0409 winnt 6.0.0 i386 MSNMSGR 14.0.8117.0416 msmsgs login@email.com\r\n",
        );
        let parsed = MSNPCommandParser::parse_message(command.clone());
        let mut handler = NotificationCommandHandler::new();

        //Act
        let result = handler.handle_command(parsed[0].clone());

        //Assert
        assert_eq!(
            result,
            "CVR 2 14.0.8117.0416 14.0.8117.0416 14.0.8117.0416 localhost localhost\r\n"
        );
    }
}
