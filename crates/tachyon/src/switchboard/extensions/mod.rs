use msnp::shared::models::font_color::FontColor;
use msnp::shared::models::font_name::FontName;
use msnp::shared::models::font_style::{FontStyle, FontStyles};
use msnp::shared::payload::msg::text_plain_msg::TextPlainMessagePayload;

pub trait CustomStyles {
    fn new_with_notice_style(body: &str) -> Self;

}

impl CustomStyles for TextPlainMessagePayload {
    fn new_with_notice_style(body: &str) -> Self {
        TextPlainMessagePayload::new(FontName::default(), FontColor::parse_from_rgb("b8b8b8").unwrap(), FontStyles::new(&[FontStyle::Bold, FontStyle::Italic]), false, body)
    }
}