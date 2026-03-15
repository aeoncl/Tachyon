use crate::shared::models::url_encoded_string::UrlEncodedString;

pub type FontName = UrlEncodedString;

pub trait DefaultFont {
    fn default_font() -> Self;
}

impl DefaultFont for FontName {
    fn default_font() -> Self {
        Self::new_from_ref("Segoe UI Emoji")
    }
}