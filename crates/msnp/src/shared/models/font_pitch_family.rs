use anyhow::anyhow;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::fmt::Display;
use std::str::FromStr;


/*
The PF family defines the category that the font specified in the FN parameter falls into. This parameter is used by the receiving client if it does not have the specified font installed. The value is a two-digit hexadecimal number. When programming with Windows APIs, this value is the PitchAndFamily value in RichEdit and LOGFONT.
The first digit of the value represents the font family. Below is a list of numbers for the first digit and the font families they represent.

Below are some PF values and example fonts that fit the category.

12
    Times New Roman, MS Serif, Bitstream Vera Serif
22
    Arial, Verdana, MS Sans Serif, Bitstream Vera Sans
31
    Courier New, Courier
42
    Comic Sans MS
*/

#[derive(FromPrimitive, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FontFamily {
    DontCare = 0,
    // Specifies a generic family name. This name is used when information about a font does not exist or does not matter. The default font is used.
    Roman = 1,
    // Specifies a proportional (variable-width) font with serifs. An example is Times New Roman.
    Swiss = 2,
    // Specifies a proportional (variable-width) font without serifs. An example is Arial.
    Modern = 3,
    // Specifies a monospace font with or without serifs. Monospace fonts are usually modern; examples include Pica, Elite, and Courier New.
    Script = 4,
    // Specifies a font that is designed to look like handwriting; examples include Script and Cursive.
    Decorative = 5
    // Specifies a novelty font. An example is Old English.
}

impl Default for FontFamily {
    fn default() -> Self {
        Self::DontCare
    }
}

#[derive(FromPrimitive, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FontPitch {
    Default = 0,
    // Specifies a generic font pitch. This name is used when information about a font does not exist or does not matter. The default font pitch is used.
    Fixed = 1,
    // Specifies a fixed-width (monospace) font. Examples are Courier New and Bitstream Vera Sans Mono.
    Variable = 2
    // Specifies a variable-width (proportional) font. Examples are Times New Roman and Arial.
}

impl Default for FontPitch {
    fn default() -> Self {
        Self::Default
    }
}

struct FontPitchFamily {
    pub pitch: FontPitch,
    pub family: FontFamily,
}

impl Default for FontPitchFamily {
    fn default() -> Self {
        Self::new(FontPitch::Default, FontFamily::DontCare)
    }
}

impl FontPitchFamily {
    pub fn new(pitch: FontPitch, family: FontFamily) -> Self {
        FontPitchFamily { pitch, family }
    }
}

impl Display for FontPitchFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.pitch as u8, self.family as u8)
    }
}

impl FromStr for FontPitchFamily {
    type Err = anyhow::Error;


    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err(anyhow!("FontPitchFamily string must be exactly 2 characters long"));
        }

        let raw_pitch = u8::from_str(&s[0..1])?;
        let raw_family = u8::from_str(&s[0..2])?;

        let pitch = FontPitch::from_u8(raw_pitch).ok_or(anyhow!("Invalid FontPitch value: {}", raw_pitch))?;
        let family = FontFamily::from_u8(raw_family).ok_or(anyhow!("Invalid FontFamily value: {}", raw_family))?;

        Ok(FontPitchFamily::new(pitch, family))
    }
}