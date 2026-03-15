use std::fmt::{Display, Formatter};
use anyhow::anyhow;
use byteorder::{ByteOrder, LittleEndian};
use crate::msnp::error::PayloadError;

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

        if hex_str.len() % 2 != 0 {
            hex_str = format!("0{:0>5}", &hex_str);
        } else {
            hex_str = format!("{:0>6}", &hex_str);
        }

        if hex_str.len() > 6  {
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
        if hex_str.len() % 2 != 0 {
            hex_str = format!("0{}", &hex_str);
        }

        if hex_str.len() > 6  {
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

    //FF00FF
    //0
    //
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

#[cfg(test)]
mod tests {
    use crate::shared::models::font_color::{FontColor, OvercomplicatedFontColor};
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