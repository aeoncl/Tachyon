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

#[derive(Debug)]
pub struct OvercomplicatedFontColor {
    red: u8,
    green: u8,
    blue: u8
}

impl OvercomplicatedFontColor {

    const BGR_RED_MASK: u32 = 0xFF000000;
    const BGR_GREEN_MASK: u32 = 0x00FF0000;
    const BGR_BLUE_MASK: u32 = 0x0000FF00;

    fn parse_hex_str(hex_str: &str) -> Result<u32, PayloadError> {
        let mut hex_str = hex_str.to_string();
        if(hex_str.len() % 2 != 0){
            hex_str = format!("0{}", &hex_str);
        }

        if(hex_str.len() > 6) {
            return Err(PayloadError::AnyError(anyhow!("Hex String Color Invalid size: length: {} str: {}", hex_str.len(), hex_str)));
        }

        let mut hex_str_buffer = hex::decode(hex_str)?;
        if hex_str_buffer.len() < 4 {
            let mut padding: Vec<u8> = vec![0; 4 - hex_str_buffer.len()];
            padding.append(&mut hex_str_buffer);
            hex_str_buffer = padding;
        }

        Ok(LittleEndian::read_u32(&hex_str_buffer))
    }

    pub fn parse_from_rgb(rgb: &str) -> Result<Self, PayloadError> {

        let color_flags = OvercomplicatedFontColor::parse_hex_str(rgb)?;

        let red =  ((color_flags & Self::BGR_BLUE_MASK) >> 8) as u8;

        let blue =  ((color_flags & Self::BGR_RED_MASK) >> 24) as u8;

        let green = ((color_flags& Self::BGR_GREEN_MASK) >> 16) as u8;

        Ok(Self{
            red,
            green,
            blue,
        })
    }

    pub fn parse_from_bgr(bgr: &str) -> Result<Self, PayloadError> {

        let color_flags = OvercomplicatedFontColor::parse_hex_str(bgr)?;

        let red =  ((color_flags & Self::BGR_RED_MASK) >> 24) as u8;

        let blue =  ((color_flags & Self::BGR_BLUE_MASK) >> 8) as u8;

        let green = ((color_flags& Self::BGR_GREEN_MASK) >> 16) as u8;

        Ok(Self{
            red,
            green,
            blue,
        })
    }

    pub fn get_red(&self) -> u8{
        self.red
    }

    pub fn get_green(&self) -> u8{
        self.green
    }

    pub fn get_blue(&self) -> u8{
        self.blue
    }

    pub fn serialize_bgr(&self) -> String {
        hex::encode(vec![self.blue, self.green, self.red])
    }

    pub fn serialize_rgb(&self) -> String {
        hex::encode(vec![self.red, self.green, self.blue])
    }

}

impl Display for OvercomplicatedFontColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.serialize_bgr())
    }
}

#[derive(Debug)]
pub struct FontColor(String);

impl FontColor{

    pub fn is_black(&self) -> bool {
        &self.0 == "000000"
    }

    pub fn parse_from_rgb(rgb: &str) -> Result<Self, PayloadError> {
        Self::parse_from_bgr(&format!("{}{}{}", &rgb[4..6], &rgb[2..4], &rgb[0..2]))
    }

    pub fn parse_from_bgr(bgr: &str) -> Result<Self, PayloadError> {

        let mut hex_str = bgr.to_string();

        if(hex_str.len() % 2 != 0){
            hex_str = format!("0{:0>5}", &hex_str);
        } else {
            hex_str = format!("{:0>6}", &hex_str);
        }

        if(hex_str.len() > 6) {
            return Err(PayloadError::AnyError(anyhow!("Hex String Color Invalid size: length: {} str: {}", hex_str.len(), hex_str)));
        }

        let _hex_str_buffer = hex::decode(hex_str.clone())?;

        Ok(Self(hex_str.to_lowercase()))
    }

    pub fn serialize_bgr(&self) -> String {
        self.0.to_string()
    }

    pub fn serialize_rgb(&self) -> String {
        format!("{}{}{}", &self.0[4..6], &self.0[2..4], &self.0[0..2])
    }
}

impl Display for FontColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.serialize_bgr())
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

impl TextMessageContent {

    pub fn is_styling_default(&self) -> bool {
       self.is_default_font_color() && self.is_default_font_styles() && self.is_default_font()
    }

    pub fn is_default_font_styles(&self) -> bool {
        self.font_styles.0 == 0
    }

    pub fn is_default_font_color(&self) -> bool {
        self.font_color.is_black()
    }

    pub fn is_default_font(&self) -> bool {
        &self.font_family == "Segoe UI"
    }

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
    use crate::msnp::notification::command::uum::{OvercomplicatedFontColor, FontStyle, MessageType, NetworkId, UumClient, UumPayload, FontColor};
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
            assert_eq!("Segoe UI Semibold", &content.font_family);
            assert_eq!("TEST", &content.body);
            assert_eq!(false, content.right_to_left);
            assert!(content.font_styles.matches(FontStyle::Italic));
            assert!(content.font_styles.matches(FontStyle::Underline));
        }

    }

    #[test]
    pub fn complicated_font_color_tests_rgb() {
        let some_purple = OvercomplicatedFontColor::parse_from_rgb("762EE1").expect("to be kinda purple");

        println!("{:?}", &some_purple);
        assert_eq!(118, some_purple.get_red());
        assert_eq!(46, some_purple.get_green());
        assert_eq!(225, some_purple.get_blue());
        let some_purple_str = some_purple.serialize_rgb();
        assert_eq!("762ee1", &some_purple_str);
    }

    #[test]
    pub fn complicated_font_color_tests_bgr() {
        let black = OvercomplicatedFontColor::parse_from_bgr("0").expect("to be black");
        assert_eq!(0, black.get_blue());
        assert_eq!(0, black.get_green());
        assert_eq!(0, black.get_red());
        let black_str = black.to_string();
        assert_eq!("000000", &black_str);

        let black = OvercomplicatedFontColor::parse_from_bgr("1").expect("to be black");
        assert_eq!(0, black.get_blue());
        assert_eq!(0, black.get_green());
        assert_eq!(1, black.get_red());
        let black_str = black.to_string();
        assert_eq!("000001", &black_str);

        let purple= OvercomplicatedFontColor::parse_from_bgr("ff00ff").expect("to be purple");
        assert_eq!(255, purple.get_blue());
        assert_eq!(0, purple.get_green());
        assert_eq!(255, purple.get_red());
        let purple_str = purple.to_string();
        assert_eq!("ff00ff", &purple_str);

        let red= OvercomplicatedFontColor::parse_from_bgr("ff").expect("to be red");
        assert_eq!(0, red.get_blue());
        assert_eq!(0, red.get_green());
        assert_eq!(255, red.get_red());
        let red_str = red.to_string();
        assert_eq!("0000ff", &red_str);

        let blue= OvercomplicatedFontColor::parse_from_bgr("ff0000").expect("to be blue");
        assert_eq!(255, blue.get_blue());
        assert_eq!(0, blue.get_green());
        assert_eq!(0, blue.get_red());
        let blue_str = blue.to_string();
        assert_eq!("ff0000", &blue_str);

         let green= OvercomplicatedFontColor::parse_from_bgr("ff00").expect("to be green");
         assert_eq!(0, green.get_blue());
         assert_eq!(255, green.get_green());
         assert_eq!(0, green.get_red());
        let green_str = green.to_string();
        assert_eq!("00ff00", &green_str);

        let light_brown = OvercomplicatedFontColor::parse_from_bgr("8080").expect("to be light brown");
        assert_eq!(0, light_brown.get_blue());
        assert_eq!(128, light_brown.get_green());
        assert_eq!(128, light_brown.get_red());
        let light_brown_str = light_brown.to_string();
        assert_eq!("008080", &light_brown_str);

        let some_purple = OvercomplicatedFontColor::parse_from_bgr("E12E76").expect("to be kinda purple");

        println!("{:?}", &some_purple);
        assert_eq!(118, some_purple.get_red());
        assert_eq!(46, some_purple.get_green());
        assert_eq!(225, some_purple.get_blue());
        let some_purple_str = some_purple.serialize_bgr();
        assert_eq!("e12e76", &some_purple_str);
    }

    #[test]
    pub fn font_color_tests_rgb() {
        let some_purple = FontColor::parse_from_rgb("762EE1").expect("to be kinda purple");

        println!("{:?}", &some_purple);
        let some_purple_str = some_purple.serialize_rgb();
        assert_eq!("762ee1", &some_purple_str);
    }

    #[test]
    pub fn font_color_tests_bgr() {
        let black = FontColor::parse_from_bgr("0").expect("to be black");
        let black_str = black.to_string();
        assert_eq!("000000", &black_str);

        let black = FontColor::parse_from_bgr("1").expect("to be black");
        let black_str = black.to_string();
        assert_eq!("000001", &black_str);

        let purple= FontColor::parse_from_bgr("ff00ff").expect("to be purple");
        let purple_str = purple.to_string();
        assert_eq!("ff00ff", &purple_str);

        let red= FontColor::parse_from_bgr("ff").expect("to be red");
        let red_str = red.to_string();
        assert_eq!("0000ff", &red_str);

        let blue= FontColor::parse_from_bgr("ff0000").expect("to be blue");
        let blue_str = blue.to_string();
        assert_eq!("ff0000", &blue_str);

        let green= FontColor::parse_from_bgr("ff00").expect("to be green");
        let green_str = green.to_string();
        assert_eq!("00ff00", &green_str);

        let light_brown = FontColor::parse_from_bgr("8080").expect("to be light brown");
        let light_brown_str = light_brown.to_string();
        assert_eq!("008080", &light_brown_str);

        let some_purple = FontColor::parse_from_bgr("E12E76").expect("to be kinda purple");

        println!("{:?}", &some_purple);
        let some_purple_str = some_purple.serialize_bgr();
        assert_eq!("e12e76", &some_purple_str);
    }

}
