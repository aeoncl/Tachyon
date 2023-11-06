use std::str::from_utf8;
use anyhow::anyhow;
use base64::Engine;
use base64::engine::general_purpose;
use crate::models::tachyon_error::PayloadError;

pub fn encode_utf16<B>(buf: &mut Vec<u8>, s: &str)
where
    B: byteorder::ByteOrder,
{
    for c in s.encode_utf16() {
        buf.extend(std::iter::repeat(0x0).take(2));
        let s = buf.len() - 2;
        B::write_u16(&mut buf[s..], c);
    }
}

pub fn map_empty_string_to_option(value: String) -> Option<String> {
    if !value.is_empty() {Some(value)} else {None}
}

pub fn encode_base64(value: String) -> String {
    general_purpose::STANDARD.encode(value.as_bytes())
}

pub fn decode_base64(value: &str) -> Result<String,PayloadError> {
    Ok(
        from_utf8(&general_purpose::STANDARD.decode(value)
            .map_err(|e| PayloadError::StringPayloadParsingError { payload: value.to_string(), sauce: e.into() })?
        ).map_err(|e| PayloadError::StringPayloadParsingError { payload: value.to_string(), sauce: e.into() })?
    .to_string()
    )
}

pub fn decode_url(value: &str) -> Result<String,PayloadError>   {
    match urlencoding::decode(value) {
        Ok(encoded_str) => {
            Ok(encoded_str.to_string())
        },
        Err(error) => {
            Err(PayloadError::StringPayloadParsingError { payload: value.to_string(), sauce: anyhow!(error) })
        }
    }
}