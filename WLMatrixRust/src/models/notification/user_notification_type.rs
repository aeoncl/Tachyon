use num_derive::FromPrimitive;

#[derive(Clone, Debug, FromPrimitive)]
pub enum UserNotificationType {
    XML_DATA = 1,
    SIP_INVITE = 2,
    P2P_DATA = 3,
    DISCONNECT = 4,
    CLOSED_CONVERSATION = 5,
    RESYNCHRONIZE = 6,
    DISMISS_USER_INVITE = 7,
    GTFO = 8,
    RTC_ACTIVITY = 11,
    TUNNELED_SIP = 12
}