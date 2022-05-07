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

                let mut payload_size: usize = command.get_payload_size().try_into().unwrap();
                if offset + payload_size > current_slice.len() {
                    //If the payload size is bigger than what we have, don't go past our buffer. Payload is chunked.
                    payload_size = current_slice.len() - offset;
                }

                if payload_size > 0 {
                    let payload = current_slice.to_string().substring(offset, offset + payload_size).to_string();
                    offset += payload.len();
                    command.payload = payload;
                }

                out.push(command);

                current_slice = &current_slice[offset..current_slice.len()];
                maybe_cap = NORMAL_COMMAND.captures(current_slice);
            }
            return out;
        }
    }

    pub fn parsed_chunked(message: String, command : MSNPCommand) -> (String, MSNPCommand) {
        let mut command = command;
        let remaining_bytes = command.get_payload_size() - command.payload.len() as i32;

        let mut bytes_to_take = remaining_bytes as usize;
        if message.len() < remaining_bytes as usize {
            bytes_to_take = message.len();
        }
        command.payload.push_str(message.substring(0, bytes_to_take));
        let remaining_stuff = message.substring(bytes_to_take, message.len());
       return (remaining_stuff.to_string(), command);
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
        return self.get_payload_size() as usize == self.payload.len();
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

    #[test]
    fn test_chunked() {
      //  let command = String::from("MOV 4 WOOWOO\r\nADL 6 15\r\n");
        let command = String::from("MOV 4 WOOWOO\r\nADL 6 15\r\n");
        let chunked_payload = String::from("<ml l=\"1\"></ml>MOV 5 WEEWOO\r\n");

        let mut parsed = MSNPCommandParser::parse_message(&command);

        let payload_command = parsed.pop().unwrap();
        assert!(payload_command.is_complete() == false);

        let (remaining_msg, payload_command) = MSNPCommandParser::parsed_chunked(chunked_payload, payload_command);

        assert!(payload_command.is_complete() == true);
    }

    #[test]

    fn test_chunked_2() {
        let first_message = "UUN 16 aeontest3@shl.local 12 3538\r\n<sip e=\"base64\" fid=\"0\" i=\"5d01ac3bc76644f7b2d33ef7f5385272\"><msg>SU5WSVRFIHNpcDphZW9udGVzdDNAc2hsLmxvY2FsIFNJUC8yLjANCmM6IGFwcGxpY2F0aW9uL3NkcA0KVXNlci1BZ2VudDogVUNDQVBJLzMuNS42ODc3LjQNCkNTZXE6IDEgSU5WSVRFDQp2OiBTSVAvMi4wL1RDUCAxMjcuMC4wLjE6NjMwOTANCnY6IFNJUC8yLjAvVENQIDEyNy4wLjAuMTo2MzA5OQ0KTVMtS2VlcC1BbGl2ZTogVUFDO2hvcC1ob3A9eWVzDQpBbGxvdzogSU5WSVRFLCBCWUUsIEFDSywgQ0FOQ0VMLCBJTkZPLCBVUERBVEUsIFJFRkVSLCBOT1RJRlksIEJFTk9USUZZLCBPUFRJT05TDQpmOiA8c2lwOmFlb250ZXN0QHNobC5sb2NhbDttZXBpZD1GNTI5NzNCNkM5MjY0QkFEOUJBODdDMUU4NDBFNEFCMD47dGFnPWY0ZjMwZmFmNTQ7ZXBpZD1kZDEzZmY3NDQ4DQppOiA1ZDAxYWMzYmM3NjY0NGY3YjJkMzNlZjdmNTM4NTI3Mg0KTWF4LUZvcndhcmRzOiA3MA0KazogMTAwcmVsDQprOiBSZXBsYWNlcw0KazogbXMtZWFybHktbWVkaWENCms6IG1zLXNlbmRlcg0KazogbXMtc2FmZS10cmFuc2Zlcg0KazogaGlzdGluZm8NCms6IHRpbWVyDQptOiA8c2lwOmFlb250ZXN0QHNobC5sb2NhbDttZXBpZD1GNTI5NzNCNkM5MjY0QkFEOUJBODdDMUU4NDBFNEFCMD47cHJveHk9cmVwbGFjZTsrc2lwLmluc3RhbmNlPSI8dXJuOnV1aWQ6MjRCNUIyMjMtNDMxNi01N0U1LThEMDQtNUY4NDNGQTYwQzQzPiINCnQ6IDxzaXA6YWVvbnRlc3QzQHNobC5sb2NhbD4NCk1TLUNvbnZlcnNhdGlvbi1JRDogZj0wDQpSZWNvcmQtUm91dGU6IDxzaXA6MTI3LjAuMC4xOjYzMDkwO3RyYW5zcG9ydD10Y3A+DQpsOiAxNzI2DQoNCnY9MA0Kbz0tIDAgMCBJTiBJUDQgMTcyLjMxLjI0MC4xDQpzPXNlc3Npb24NCmM9SU4gSVA0IDE3Mi4zMS4yNDAuMQ0KYj1DVDo5OTk4MA0KdD0wIDANCm09YXVkaW8gMTI0MDYgUlRQL0FWUCAxMTQgMTExIDExMiAxMTUgMTE2IDQgOCAwIDk3IDEzIDExOCAxMDENCmE9aWNlLXVmcmFnOmxkMkUNCmE9aWNlLXB3ZDpIaUJWNEFybDhselVmSWRvNzBNQjlyZzUNCmE9Y2FuZGlkYXRlOjEgMSBVRFAgMjEzMDcwNjQzMSAxNzIuMzEuMjQwLjEgMTI0MDYgdHlwIGhvc3QgDQphPWNhbmRpZGF0ZToxIDIgVURQIDIxMzA3MDU5MTggMTcyLjMxLjI0MC4xIDE1NTIzIHR5cCBob3N0IA0KYT1jYW5kaWRhdGU6MiAxIFVEUCAyMTMwNzA1OTE5IDE3Mi4yMC4xNjAuMSA0Mjg4IHR5cCBob3N0IA0KYT1jYW5kaWRhdGU6MiAyIFVEUCAyMTMwNzA1NDA2IDE3Mi4yMC4xNjAuMSAxODUyOSB0eXAgaG9zdCANCmE9Y2FuZGlkYXRlOjMgMSBVRFAgMjEzMDcwNTQwNyAxNzIuMzEuMjI0LjEgMTE3NTYgdHlwIGhvc3QgDQphPWNhbmRpZGF0ZTozIDIgVURQIDIxMzA3MDQ4OTQgMTcyLjMxLjIyNC4xIDU2NTQgdHlwIGhvc3QgDQphPWNhbmRpZGF0ZTo0IDEgVURQIDIxMzA3MDQ4OTUgMTcyLjI1LjIyNC4xIDI4ODY5IHR5cC".to_string();
        let second_message = "Bob3N0IA0KYT1jYW5kaWRhdGU6NCAyIFVEUCAyMTMwNzA0MzgyIDE3Mi4yNS4yMjQuMSAyMDUyNiB0eXAgaG9zdCANCmE9Y2FuZGlkYXRlOjUgMSBVRFAgMjEzMDcwNDM4MyAxNzIuMjIuNDguMSAzMzY1NiB0eXAgaG9zdCANCmE9Y2FuZGlkYXRlOjUgMiBVRFAgMjEzMDcwMzg3MCAxNzIuMjIuNDguMSAxNTQ0NyB0eXAgaG9zdCANCmE9Y2FuZGlkYXRlOjYgMSBVRFAgMjEzMDcwMzg3MSAxOTIuMTY4LjU2LjEgODAyNyB0eXAgaG9zdCANCmE9Y2FuZGlkYXRlOjYgMiBVRFAgMjEzMDcwMzM1OCAxOTIuMTY4LjU2LjEgOTQ0NSB0eXAgaG9zdCANCmE9Y2FuZGlkYXRlOjcgMSBVRFAgMjEzMDcwMzM1OSAxOTIuMTY4LjEuNjIgMjU4NTMgdHlwIGhvc3QgDQphPWNhbmRpZGF0ZTo3IDIgVURQIDIxMzA3MDI4NDYgMTkyLjE2OC4xLjYyIDIzMTczIHR5cCBob3N0IA0KYT1jYW5kaWRhdGU6OSAxIFRDUC1BQ1QgMTY4NDc5NTY0NyAxNzIuMzEuMjQwLjEgMTI0MDYgdHlwIHNyZmx4IHJhZGRyIDE3Mi4zMS4yNDAuMSBycG9ydCAxMjQwNiANCmE9Y2FuZGlkYXRlOjkgMiBUQ1AtQUNUIDE2ODQ3OTUxMzQgMTcyLjMxLjI0MC4xIDEyNDA2IHR5cCBzcmZseCByYWRkciAxNzIuMzEuMjQwLjEgcnBvcnQgMTI0MDYgDQphPW1heHB0aW1lOjIwMA0KYT1ydGNwOjE1NTIzDQphPXJ0cG1hcDoxMTQgeC1tc3J0YS8xNjAwMA0KYT1mbXRwOjExNCBiaXRyYXRlPTI5MDAwDQphPXJ0cG1hcDoxMTEgU0lSRU4vMTYwMDANCmE9Zm10cDoxMTEgYml0cmF0ZT0xNjAwMA0KYT1ydHBtYXA6MTEyIEc3MjIxLzE2MDAwDQphPWZtdHA6MTEyIGJpdHJhdGU9MjQwMDANCmE9cnRwbWFwOjExNSB4LW1zcnRhLzgwMDANCmE9Zm10cDoxMTUgYml0cmF0ZT0xMTgwMA0KYT1ydHBtYXA6MTE2IEFBTDItRzcyNi0zMi84MDAwDQphPXJ0cG1hcDo0IEc3MjMvODAwMA0KYT1ydHBtYXA6OCBQQ01BLzgwMDANCmE9cnRwbWFwOjAgUENNVS84MDAwDQphPXJ0cG1hcDo5NyBSRUQvODAwMA0KYT1ydHBtYXA6MTMgQ04vODAwMA0KYT1ydHBtYXA6MTE4IENOLzE2MDAwDQphPXJ0cG1hcDoxMDEgdGVsZXBob25lLWV2ZW50LzgwMDANCmE9Zm10cDoxMDEgMC0xNg0KYT1lbmNyeXB0aW9uOnJlamVjdGVkDQo=</msg></sip>";

        let mut parsed = MSNPCommandParser::parse_message(&first_message);
  
          let payload_command = parsed.pop().unwrap();
          assert!(payload_command.is_complete() == false);
  
          let (remaining, payload_command) = MSNPCommandParser::parsed_chunked(second_message.to_string(), payload_command);
  
          assert!(payload_command.is_complete() == true);
    }
}
