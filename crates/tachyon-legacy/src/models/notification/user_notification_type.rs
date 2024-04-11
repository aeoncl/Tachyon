use num_derive::FromPrimitive;

#[derive(Clone, Debug, FromPrimitive)]
pub enum UserNotificationType {
    XmlData = 1,
    SipInvite = 2,
    P2PData = 3,
    Disconnect = 4,
    ClosedConversation = 5,
    Resynchronize = 6,
    DismissUserInvite = 7,
    GTFO = 8,
    RTCActivity = 11,
    TunneledSip = 12
}