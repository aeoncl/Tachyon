
use num_derive::FromPrimitive;

#[derive(Clone, Debug, FromPrimitive, Eq, PartialEq)]
pub enum AppID {
    FILE_TRANSFER = 2,
    CUSTOM_EMOTICON_TRANSFER = 11,
    DISPLAY_PICTURE_TRANSFER = 12,
    PHOTO_SHARING = 35,
    WEBCAM = 4
}