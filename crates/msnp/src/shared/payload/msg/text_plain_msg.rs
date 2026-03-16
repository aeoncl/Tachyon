use std::cmp::PartialEq;
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;

use anyhow::anyhow;
use byteorder::ByteOrder;
use crate::msnp::error::PayloadError;
use crate::shared::models::font_color::FontColor;
use crate::shared::models::font_name::{DefaultFont, FontName};
use crate::shared::models::font_style::FontStyles;
use crate::shared::payload::msg::raw_msg_payload::MsgContentType::TextPlain;
use crate::shared::payload::msg::raw_msg_payload::{MsgContentType, RawMsgPayload};
use crate::shared::traits::{TryFromRawMsgPayload, TryFromBytes, IntoBytes};
pub struct TextPlainMessagePayload {
    pub font_family: FontName,
    pub right_to_left: bool,
    pub font_styles: FontStyles,
    pub font_color: FontColor,
    pub body : String,
}

impl TryFromRawMsgPayload for TextPlainMessagePayload {
    type Err = PayloadError;

    fn try_from_raw(mut raw: RawMsgPayload) -> Result<Self, Self::Err> where Self: Sized {

        if MsgContentType::TextPlain != raw.get_content_type().unwrap() {
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
        let font_family = FontName::from_str(font_family).map_err(|e| PayloadError::AnyError(anyhow!(e)))?;

        let right_to_left = format_header_map.get("RL").unwrap_or(&"0") == &"1";

        let font_styles = format_header_map.get("EF").unwrap_or(&"");

        let font_color = FontColor::parse_from_bgr(format_header_map.get("CO").unwrap_or(&"0"))?;

        let body =  String::from_utf8(raw.body)?;

        Ok(
            TextPlainMessagePayload {
                font_family,
                right_to_left,
                font_styles: FontStyles::from_str(font_styles).expect("To be infaillible"),
                font_color,
                body,
            }
        )
    }
}

impl IntoBytes for TextPlainMessagePayload {
    fn into_bytes(self) -> Vec<u8> {
        let mut out = RawMsgPayload::new(TextPlain, false);
        out.add_header_owned("X-MMS-IM-Format".into(), self.get_mms_format_header());
        out.set_body_string(self.body);
        out.into_bytes()
    }
}

impl TextPlainMessagePayload {

    pub fn new_with_default_style(body: &str) -> Self {
        Self {
            font_family: FontName::default_font(),
            right_to_left: false,
            font_styles: Default::default(),
            font_color: Default::default(),
            body: body.to_string(),
        }
    }

    pub fn new(font_family: FontName, font_color: FontColor, font_styles: FontStyles, right_to_left: bool, body: &str) -> Self {
        Self{
            font_family,
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
        self.font_styles.value() == 0
    }

    pub fn is_default_font_color(&self) -> bool {
        self.font_color.is_default()
    }

    pub fn is_default_font(&self) -> bool {
        self.font_family == FontName::default_font()
    }

    pub fn get_mms_format_header(&self) -> String {
        let right_left = if self.right_to_left { "1" } else { "0" };

        format!("FN={}; EF={}; CO={}; PF={}; RL={}", self.font_family, self.font_styles, self.font_color, 0, right_left)
    }
}


