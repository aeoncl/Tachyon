use num_derive::FromPrimitive;

#[derive(Clone, Debug, FromPrimitive, Eq, PartialEq)]
pub enum AppID {
    FileTransfer = 2,
    CustomEmoticonTransfer = 11,
    DisplayPictureTransfer = 12,
    PhotoSharing = 35,
    Webcam = 4
}