use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::str::{from_utf8, FromStr};
use std::thread::sleep;

use anyhow::anyhow;
use bit_array::BitArray;
use byteorder::{BigEndian, ByteOrder, LittleEndian};
use log::{debug, warn};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use yaserde::de::from_str;
use typenum::{U32, U8};
use crate::msnp::error::{CommandError, PayloadError};
use crate::msnp::notification::command::usr::{AuthPolicy, OperationTypeServer, SsoPhaseServer, UsrClient, UsrServer};
use crate::msnp::notification::command::uun::{UserNotificationType, UunSoapStatePayload};
use crate::msnp::raw_command_parser::RawCommand;
use crate::p2p::v2::events::content::message_event_content::MessageEventContent;
use crate::shared::command::ok::OkCommand;
use crate::shared::models::endpoint_id::EndpointId;
use crate::shared::payload::raw_msg_payload::RawMsgPayload;
use crate::shared::traits::{MSNPCommand, MSNPPayload};

pub struct UumClient {
    pub tr_id: u128,
    pub destination: EndpointId,
    pub network_id: NetworkId,
    pub message_type: MessageType,
    pub payload: UumPayload

}
#[derive(FromPrimitive, Eq, PartialEq, Debug)]
pub enum NetworkId {
    WindowsLive = 0x01,
    OfficeCommunicator = 0x02,
    Telephone = 0x04,
    //used by Vodafone
    MobileNetworkInterop = 0x08,
    //Jaguire, Japanese mobile interop
    Smtp = 0x10,
    Yahoo = 0x20
}

#[derive(FromPrimitive, Clone, Eq, PartialEq, Debug)]
pub enum MessageType {
    TextMessage = 1,
    TypingUser = 2,
    Nudge = 3,
    UnknownYet = 4
}

pub enum UumPayload {
    TextMessage(TextMessageContent),
    TypingUser(),
    Nudge(),
    Raw(RawMsgPayload),
}

impl UumPayload {

    fn parse_uum_payload(payload_type: MessageType, payload: Vec<u8>) -> Result<Self, PayloadError> {
        match payload_type {
            MessageType::TextMessage => {


                let mut raw = RawMsgPayload::try_from_bytes(payload)?;
                if "text/plain" != &raw.content_type {
                    return Err(PayloadError::PayloadPropertyParseError {
                        property_name: "Content-Type".to_string(),
                        raw_value: format!("{:?}", raw),
                        payload_type: "UUM TextMessage".to_string(),
                        source: anyhow!("Content Type doesnt match expectation for this type of message"),
                    });
                }

                let format_header = raw.headers.remove("X-MMS-IM-Format").ok_or(PayloadError::MandatoryPartNotFound { name: "X-MMS-IM-Format".to_string(), payload: "".to_string() })?;

                let format_header_split = format_header.split(';');

                let mut format_header_map : HashMap<&str, &str> = HashMap::new();

                for current_header in format_header_split {
                    let (name, value) = current_header.trim().split_once("=").ok_or(PayloadError::StringPayloadParsingError { payload: "".to_string(), source: anyhow!("Invalid X-MMS-IM-Format part, could'nt split at '=' : {}", current_header)})?;
                    format_header_map.insert(name.trim(), value.trim());
                }

                let font_family = format_header_map.get("FN").ok_or(PayloadError::MandatoryPartNotFound{ name: "FN".to_string(), payload: "".to_string() })?;
                let font_family = urlencoding::decode(font_family)?.to_string();

                let right_to_left = format_header_map.get("RL").unwrap_or(&"0") == &"1";

                let font_styles = format_header_map.get("EF").unwrap_or(&"");

                let font_color = FontColor::parse_from_bgr(format_header_map.get("CO").unwrap_or(&"0"))?;

                let body =  String::from_utf8(raw.body)?;

                Ok(UumPayload::TextMessage(
                    TextMessageContent {
                        font_family,
                        right_to_left,
                        font_styles: FontStyles::from_str(font_styles).expect("To be infaillible"),
                        font_color,
                        body,
                    }
                ))
            },
            MessageType::TypingUser => {todo!()},
            MessageType::Nudge => {todo!()},
            MessageType::UnknownYet => {todo!()},
        }

    }

}

pub struct FontColor(u32);

impl FontColor {

    const BGR_RED_MASK: u32 = 0xFF000000;
    const BGR_GREEN_MASK: u32 = 0x00FF0000;
    const BGR_BLUE_MASK: u32 = 0x0000FF00;

    pub fn parse_from_bgr(bgr: &str) -> Result<Self, PayloadError> {

        if(bgr=="0"){
            return Ok(Self(0));
        }

        if(bgr.len() > 6 || bgr.len() % 2 != 0) {
            return Err(PayloadError::AnyError(anyhow!("BGR Color Invalid size: length: {} str: {}", bgr.len(), bgr)));
        }



        let mut bgr_buffer = hex::decode(bgr)?;
        bgr_buffer.resize(4, 0);

        let bgr_parsed= BigEndian::read_u32(&bgr_buffer);


        Ok(Self(bgr_parsed))
    }

    fn print_array(&self, str: &str, value: u32) {
        let mut buf: [u8;4] = [0;4];
        BigEndian::write_u32(&mut buf, value);
        let array = BitArray::<u8, U32>::from_bytes(&buf);
        println!("{}: {:?}", str, array);
    }

    pub fn get_blue(&self) -> u8{
        println!("--BLUE--");
        self.print_array("Global Array", self.0);

        self.print_array("Mask:       ",  Self::BGR_BLUE_MASK);
        self.print_array("Mask And:   ", (self.0 & Self::BGR_BLUE_MASK) as u32);
        let bitshift =  ((self.0 & Self::BGR_BLUE_MASK));

        self.print_array("Bitshift:   ", bitshift);

        bitshift as u8
    }

    pub fn get_red(&self) -> u8{
        println!("--RED--");

        self.print_array("Global Array", self.0);
        self.print_array("Mask:       ",  Self::BGR_RED_MASK);
        self.print_array("Mask And:   ", (self.0 & Self::BGR_RED_MASK) as u32);
        self.print_array("Bitshift:   ", ((self.0 & Self::BGR_RED_MASK)) as u32);

        let bitshift = ((self.0 & Self::BGR_RED_MASK) >> 24);
        self.print_array("Bitshift:   ", bitshift);


        bitshift as u8
    }

    pub fn get_green(&self) -> u8{
        println!("--GREEN--");

        self.print_array("Global Array", self.0);
        self.print_array("Mask:       ",  Self::BGR_GREEN_MASK);
        self.print_array("Mask And:   ", (self.0 & Self::BGR_GREEN_MASK) as u32);

        let bitshift = ((self.0 & Self::BGR_GREEN_MASK) >> 16);

        self.print_array("Bitshift:   ", bitshift);

        bitshift as u8
    }

}

impl Display for FontColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {

        let mut buf :[u8;4] = [0;4];
        LittleEndian::write_u32(&mut buf, self.0);

        write!(f, "{}", hex::encode(buf))
    }
}



pub enum FontStyle {
    Bold = 0x1,
    Italic = 0x2,
    Underline = 0x4,
    StrikeThrough = 0x8
}

pub struct FontStyles(u32);

impl FontStyles {
    pub fn matches(&self, font_style: FontStyle) -> bool {
        let font_style_as_int = font_style as u32;
        let and = self.0 & font_style_as_int;
        return and == font_style_as_int
    }

}

impl FromStr for FontStyles {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut flags = 0;
        s.to_uppercase().chars().for_each(|c| {
           match c {
               'B' => flags += FontStyle::Bold as u32,
               'I' => flags += FontStyle::Italic as u32,
               'U' => flags += FontStyle::Underline as u32,
               'S' => flags += FontStyle::StrikeThrough as u32,
               _ => {
                   warn!("Unhandled TextMessage formatting qualifier: {}", c);
               }
           }
        });
        Ok(FontStyles(flags))
    }
}

impl Display for FontStyles {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {

        let mut out = String::with_capacity(4);

        if self.matches(FontStyle::Bold) {
            out.push('B');
        }

        if self.matches(FontStyle::Italic) {
            out.push('I');
        }

        if self.matches(FontStyle::Underline) {
            out.push('U');
        }

        if self.matches(FontStyle::StrikeThrough) {
            out.push('S');
        }

        write!(f, "{}", out)
    }
}

pub struct TextMessageContent {
    pub font_family: String,
    pub right_to_left: bool,
    pub font_styles: FontStyles,
    pub font_color: FontColor,
    pub body : String,
}

pub struct TypingUserMessageContent(RawMsgPayload);
pub struct NudgeMessageContent(RawMsgPayload);

impl MSNPCommand for UumClient {
    type Err = CommandError;

    fn try_from_raw(raw: RawCommand) -> Result<Self, Self::Err> {

        debug!("{}\r\n{}", raw.command, from_utf8(raw.payload.as_slice()).unwrap());

        let mut split = raw.command_split;
        let _operand = split.pop_front();

        let raw_tr_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "tr_id".into(), 1))?;
        let tr_id = u128::from_str(&raw_tr_id)?;

        let raw_destination = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "destination".into(), 2))?;
        let destination = EndpointId::from_str(&raw_destination)?;

        let raw_network_id = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "network-id".into(), 3))?;
        let network_id = NetworkId::from_u32(u32::from_str(&raw_network_id)?).ok_or(CommandError::ArgumentParseError {
            argument: "network-id".into(),
            command: raw.command.clone(),
            source: anyhow!("Unknown network-id: {}", raw_network_id),
        })?;

        let raw_notification_type = split.pop_front().ok_or(CommandError::MissingArgument(raw.command.clone(), "message-type".into(), 4))?;

        let message_type = MessageType::from_u32(u32::from_str(&raw_notification_type)?).ok_or(CommandError::ArgumentParseError { argument: raw_notification_type.to_string(), command: raw.command.clone(), source: anyhow!("Couldn't parse int to UserNotificationType") })?;

        let payload = UumPayload::parse_uum_payload(message_type.clone(), raw.payload)?;

        Ok(Self {
            tr_id,
            destination,
            network_id,
            message_type,
            payload,
        })
    }

    fn to_bytes(self) -> Vec<u8> {
        todo!()
    }
}

impl UumClient {
    pub fn get_ok_response(&self) -> OkCommand {
        OkCommand {tr_id: self.tr_id, operand: "UUM".to_string()}
    }
}


#[cfg(test)]
mod tests {
    use crate::msnp::notification::command::uum::{FontColor, FontStyle, MessageType, NetworkId, UumClient, UumPayload};
    use crate::msnp::raw_command_parser::{RawCommand, RawCommandParser};
    use crate::shared::traits::MSNPCommand;

    #[test]
    fn uun_client_text_message_deser() {
        let mut command_parser = RawCommandParser::new();
        let raw = "UUM 12 bob@yahoo.com 32 1 144\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=utf-8\r\nX-MMS-IM-Format: FN=Microsoft%20Sans%20Serif; EF=B; CO=0; CS=0; PF=22\r\n\r\nHello Bob !";
        let raw_command = command_parser.parse_message(raw.as_bytes()).unwrap().pop().unwrap();
        let uum_client = UumClient::try_from_raw(raw_command).unwrap();

        assert_eq!(12, uum_client.tr_id);
        assert_eq!("bob@yahoo.com", uum_client.destination.email_addr.as_str());
        assert!(uum_client.destination.endpoint_guid.is_none());
        assert_eq!(NetworkId::Yahoo, uum_client.network_id);

        assert!(matches!( uum_client.payload, UumPayload::TextMessage(_)));

        if let UumPayload::TextMessage(content) = uum_client.payload {
          //  assert_eq!(0, content.font_color);
            assert_eq!("Microsoft Sans Serif", &content.font_family);
            assert_eq!("Hello Bob !", &content.body);
            assert_eq!(false, content.right_to_left);
            assert!(content.font_styles.matches(FontStyle::Bold));
            assert!(!content.font_styles.matches(FontStyle::Italic));
        }
    }

    #[test]
    pub fn uun_client_text_message_deser_with_color() {
        let mut command_parser = RawCommandParser::new();
        let raw = "UUM 27 aeoncl@shlasouf.local 1 1 160\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=UTF-8\r\nX-MMS-IM-Format: FN=Segoe%20UI%20Semibold; EF=IU; CO=ff00ff; CS=0; PF=22\r\nDest-Agent: client\r\n\r\nTEST";
        let raw_command = command_parser.parse_message(raw.as_bytes()).unwrap().pop().unwrap();
        let uum_client = UumClient::try_from_raw(raw_command).unwrap();

        assert_eq!(27, uum_client.tr_id);
        assert_eq!("aeoncl@shlasouf.local", uum_client.destination.email_addr.as_str());
        assert!(uum_client.destination.endpoint_guid.is_none());
        assert_eq!(NetworkId::WindowsLive, uum_client.network_id);

        if let UumPayload::TextMessage(content) = uum_client.payload {
           // assert_eq!(0, content.font_color);
            assert_eq!("Microsoft Sans Serif", &content.font_family);
            assert_eq!("Hello Bob !", &content.body);
            assert_eq!(false, content.right_to_left);
            assert!(content.font_styles.matches(FontStyle::Italic));
            assert!(content.font_styles.matches(FontStyle::Underline));
        }

    }

    fn font_color_ser_tests() {

    }

    #[test]
    pub fn font_color_deser_tests() {
        // let black = FontColor::parse_from_bgr("0").expect("to be black");
        // assert_eq!(0, black.get_blue());
        // assert_eq!(0, black.get_green());
        // assert_eq!(0, black.get_red());

        // let purple=FontColor::parse_from_bgr("ff00ff").expect("to be purple");
        // assert_eq!(255, purple.get_blue());
        // assert_eq!(0, purple.get_green());
        // assert_eq!(255, purple.get_red());

        // let red=FontColor::parse_from_bgr("ff").expect("to be red");
        // assert_eq!(0, red.get_blue());
        // assert_eq!(0, red.get_green());
        // assert_eq!(255, red.get_red());
        //
        // let blue=FontColor::parse_from_bgr("ff0000").expect("to be blue");
        // assert_eq!(255, blue.get_blue());
        // assert_eq!(0, blue.get_green());
        // assert_eq!(0, blue.get_red());
        //
         let green=FontColor::parse_from_bgr("ff00").expect("to be green");
         assert_eq!(0, green.get_blue());
         assert_eq!(255, green.get_green());
         assert_eq!(0, green.get_red());




        // let light_brown = FontColor::parse_from_bgr("008080").expect("to be light brown");
        // assert_eq!(0, light_brown.get_blue());
        // assert_eq!(128, light_brown.get_green());
        // assert_eq!(128, light_brown.get_red());

        // let light_brown = FontColor::parse_from_bgr("8080").expect("to be light brown");
        // assert_eq!(0, light_brown.get_blue());
        // assert_eq!(128, light_brown.get_green());
        // assert_eq!(128, light_brown.get_red());
    }

}
