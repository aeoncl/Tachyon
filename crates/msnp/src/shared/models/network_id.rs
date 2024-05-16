use num_derive::FromPrimitive;

#[derive(FromPrimitive, Eq, PartialEq, Debug)]
pub enum NetworkId {
    WindowsLive = 0x01,
    OfficeCommunicator = 0x02,
    Telephone = 0x04,
    //used by Vodafone
    MobileNetworkInterop = 0x08,
    //Jaguire, Japanese mobile interop
    Smtp = 0x10,
    Yahoo = 0x20
}
