use crate::shared::models::url_encoded_string::UrlEncodedString;

pub type FontFamily = UrlEncodedString;

pub trait DefaultFont {
    fn default_font() -> Self;
}

impl DefaultFont for FontFamily {
    fn default_font() -> Self {
        Self::new_from_ref("Segoe UI")
    }
}