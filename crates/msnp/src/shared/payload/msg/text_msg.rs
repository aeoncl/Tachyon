use std::collections::HashMap;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use anyhow::anyhow;
use byteorder::{ByteOrder, LittleEndian};
use log::warn;

use crate::msnp::error::PayloadError;
use crate::msnp::notification::command::uum::UumPayload;
use crate::shared::payload::msg::raw_msg_payload::factories::RawMsgPayloadFactory;
use crate::shared::payload::msg::raw_msg_payload::RawMsgPayload;
use crate::shared::traits::{MSGPayload, MSNPPayload};

#[cfg(test)]
mod tests {
    use super::{FontColor, OvercomplicatedFontColor};
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

pub struct TextMessageContent {
    pub font_family: String,
    pub right_to_left: bool,
    pub font_styles: FontStyles,
    pub font_color: FontColor,
    pub body : String,
}

impl MSGPayload for TextMessageContent {
    type Err = PayloadError;

    fn try_from_raw(mut raw: RawMsgPayload) -> Result<Self, Self::Err> where Self: Sized {

        if "text/plain" != &raw.content_type {
            return Err(PayloadError::PayloadPropertyParseError {
                property_name: "Content-Type".to_string(),
                raw_value: format!("{:?}", raw),
                payload_type: "MSG".to_string(),
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

        Ok(
            TextMessageContent {
                font_family,
                right_to_left,
                font_styles: FontStyles::from_str(font_styles).expect("To be infaillible"),
                font_color,
                body,
            }
        )
    }

    fn into_bytes(self) -> Vec<u8> {
        let mut out = RawMsgPayload::new("text/plain");

        let font_family = urlencoding::encode(&self.font_family);
        let right_left = if self.right_to_left { "1" } else { "0" };

        out.add_header_owned("X-MMS-IM-Format".into(), format!("FN={}; EF={}; CO={}; PF={}; RL={}", font_family, self.font_styles, self.font_color, 0, right_left));
        out.set_body_string(self.body);
        out.disable_trailing_terminators();
        out.into_bytes()
    }
}

impl TextMessageContent {

    pub fn new_with_default_style(body: &str) -> Self {
        Self {
            font_family: "Segoe UI".to_string(),
            right_to_left: false,
            font_styles: Default::default(),
            font_color: Default::default(),
            body: body.to_string(),
        }
    }

    pub fn new(font_family: &str, font_color: FontColor, font_styles: FontStyles, right_to_left: bool, body: &str) -> Self {
        Self{
            font_family: font_family.to_string(),
            right_to_left,
            font_styles,
            font_color,
            body: body.to_string(),
        }
    }

    pub fn is_styling_default(&self) -> bool {
        self.is_default_font_color() && self.is_default_font_styles() && self.is_default_font()
    }

    pub fn is_default_font_styles(&self) -> bool {
        self.font_styles.0 == 0
    }

    pub fn is_default_font_color(&self) -> bool {
        self.font_color.is_default()
    }

    pub fn is_default_font(&self) -> bool {
        &self.font_family == "Segoe UI"
    }
}

pub struct FontStyles(u32);

impl FontStyles {
    pub fn matches(&self, font_style: FontStyle) -> bool {
        let font_style_as_int = font_style as u32;
        let and = self.0 & font_style_as_int;
        return and == font_style_as_int
    }

}

impl Default for FontStyles {
    fn default() -> Self {
        Self(0)
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

pub enum FontStyle {
    Bold = 0x1,
    Italic = 0x2,
    Underline = 0x4,
    StrikeThrough = 0x8
}


#[derive(Debug)]
pub struct FontColor(String);

impl FontColor{

    pub fn is_default(&self) -> bool {
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

impl Default for FontColor {
    fn default() -> Self {
        Self("000000".into())
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

    pub fn is_default(&self) -> bool {
        self.green == 0 && self.red == 0 && self.blue == 0
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

impl Default for OvercomplicatedFontColor {
    fn default() -> Self {
        Self{
            red: 0,
            green: 0,
            blue: 0,
        }
    }
}

impl Display for OvercomplicatedFontColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.serialize_bgr())
    }
}

