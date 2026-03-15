use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use log::warn;

pub struct FontStyles(u32);

impl FontStyles {
    pub fn matches(&self, font_style: FontStyle) -> bool {
        let font_style_as_int = font_style as u32;
        let and = self.0 & font_style_as_int;
        and == font_style_as_int
    }

    pub fn new(styles: &[FontStyle]) -> Self {
        let styles = styles.iter().fold(0, |acc, style| acc | (*style as u32));
        Self(styles)
    }
    
    pub fn value(&self) -> u32 {
        self.0
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Bold = 0x1,
    Italic = 0x2,
    Underline = 0x4,
    StrikeThrough = 0x8
}