use std::{fmt::Display, str::FromStr};
use std::num::ParseIntError;
use anyhow::anyhow;

use crate::msnp::error::PayloadError;


/** source: https://wiki.nina.chat/wiki/Protocols/MSNP/MSNC/Client_Capabilities */

pub enum Capabilities {

    // This means you are running a Windows Mobile device. The official client changes the little icon to a little man with a phone, and puts the status 'Phone' next to your name.
    MobileOnline = 0x01,
    // This value is set if you are a MSN Explorer 8 user, but it is sometimes used when the client resets its capabilities
    MSN8User = 0x02,
    //  Your client can send/receive Ink (GIF format)
    RendersGifs = 0x04,
    // Your client can send/recieve Ink (ISF format)
    RendersIsf = 0x08,
    // This option is set when you are able to participate in video conversations. In reality, it is only set when you have a webcam connected and have it set to 'shared'.
    WebcamDetected = 0x10,
    // This value is being used with Multi-Packet Messaging.
    SupportsChunking = 0x20,
    // This is used when the client is running on a MSN Mobile device. This is equivalent to the MOB setting in the BPR list.
    MobileEnabled = 0x40,
    // This is used when the client is running on a MSN Direct device. This is equivalent to the WWE setting in the BPR list.
    DirectDevice = 0x80,
    // This is used when someone signs in on the official Web-based MSN Messenger. It will show a new icon in other people's contact list.
    WebIMClient = 0x200,
    // Internal Microsoft client and/or Microsoft Office Live client (TGWClient).
    ConnectedViaTGW = 0x800,
    // This means you have a MSN Space.
    HasSpace = 0x1000,
    // This means you are using Windows XP Media Center Edition.
    MCEUser = 0x2000,
    // This means you support 'DirectIM' (creating direct connections for conversations rather than using the traditional switchboard)
    SupportsDirectIM = 0x4000,
    // This means you support Winks receiving (If not set the official Client will warn with 'contact has an older client and is not capable of receiving Winks')
    SupportsWinks = 0x8000,
    // Your client supports the MSN Search feature
    SupportsMSNSearch = 0x10000,
    // The client is bot (provisioned account)
    IsBot = 0x20000,
    // This means you support Voice Clips receiving
    SupportsVoiceIM = 0x40000,
    // This means you support Secure Channel Communications
    SupportsSChannel = 0x80000,
    // Supports SIP Invitations
    SupportsSipInvite = 0x100000,
    // Supports Tunneled SIP
    SupportsTunneledSip = 0x200000,
    // Sharing Folders
    SupportsSDrive = 0x400000,
    // The client has OneCare
    HasOneCare = 0x1000000,
    // Supports P2P TURN
    SupportsP2PTurn = 0x2000000,
    // Supports P2P Bootstrap via UUN
    SupportsP2PBootstrapViaUUN = 0x4000000,
    // (MSN Msgr 6.0)
    MsgrVersion1 = 0x10000000,
    // (MSN Msgr 6.1)
    MsgrVersion2 = 0x20000000,
    // (MSN Msgr 6.2)
    MsgrVersion3 = 0x30000000,
    // (MSN Msgr 7.0)
    MsgrVersion4 = 0x40000000,
    // (MSN Msgr 7.5)
    MsgrVersion5 = 0x50000000,
    // (WL Msgr 8.0)
    MsgrVersion6 = 0x60000000,
    // (WL Msgr 8.1)
    MsgrVersion7 = 0x70000000,
    // (WL Msgr 8.5)
    MsgrVersion8 = 0x80000000,
    // (WL Msgr 9.0)
    MsgrVersion9 = 0x90000000,
    // (WL Msgr 14.0)
    MsgrVersion10 = 0xA0000000

}

pub enum ExtendedCapabilities {
    None = 0x0,
    IsSmsOnly = 0x01,
    SupportsVoiceOverMsnp = 0x02,
    SupportsUucpSipStack = 0x04,
    SupportsApplicationMessages = 0x08,
    // RTC Video enabled
    RTCVideoEnabled = 0x10,
    // Supports P2PV2
    SupportsP2PV2 = 0x20,
    IsAuthenticatedWebIMUser = 0x40,
    Supports1On1ViaGroup = 0x80,
    SupportsOfflineIM = 0x100,
    SupportsSharingVideo = 0x200,
    SupportsNudges = 0x400,
    CircleVoiceIMEnabled = 0x800,
    SharingEnabled = 0x1000,
    MobileSuspendIMFanoutDisable = 0x2000,
    _0x4000 = 0x4000,
    SupportsPeerToPeerMixerRelay = 0x8000,
    _0x10000 = 0x10000,
    ConvWindowFileTransfer = 0x20000,
    VideoCallSupports16x9 = 0x40000,
    SupportsPeerToPeerEnveloping = 0x80000,
    _0x100000 = 0x100000,
    _0x200000 = 0x200000,
    YahooIMDisabled = 0x400000,
    SIPTunnelVersion2 = 0x800000,
    VoiceClipSupportsWMAFormat = 0x1000000,
    VoiceClipSupportsCircleIM = 0x2000000,
    SupportsSocialNewsObjectTypes = 0x4000000,
    CustomEmoticonsCapable = 0x8000000,
    SupportsUTF8MoodMessages = 0x10000000,
    FTURNCapable = 0x20000000,
    SupportsP4Activity = 0x40000000,
    SupportsMultipartyConversations = 0x80000000,
}
#[derive(Clone, Debug)]

pub struct ClientCapabilities {
    capabilities: u32,
    extended_capabilities: u32
}

impl Default for ClientCapabilities {
    fn default() -> Self {
      const STANDARD_CAP: u32 = Capabilities::RendersGifs as u32
      + Capabilities::RendersIsf as u32
      + Capabilities::WebcamDetected as u32
      + Capabilities::SupportsTunneledSip as u32
      + Capabilities::SupportsChunking as u32
      + Capabilities::HasSpace as u32
      + Capabilities::SupportsWinks as u32
      + Capabilities::SupportsMSNSearch as u32
      + Capabilities::SupportsVoiceIM as u32
      + Capabilities::SupportsSipInvite as u32
      + Capabilities::MsgrVersion10 as u32
      + Capabilities::SupportsP2PTurn as u32
      + Capabilities::SupportsP2PBootstrapViaUUN as u32;

  const EXTENDED_CAP: u32 = ExtendedCapabilities::RTCVideoEnabled as u32
      + ExtendedCapabilities::SupportsP2PV2 as u32
      + ExtendedCapabilities::Supports1On1ViaGroup  as u32
      + ExtendedCapabilities::SupportsOfflineIM as u32
      + ExtendedCapabilities::SupportsSharingVideo as u32
      + ExtendedCapabilities::SupportsNudges as u32
      + ExtendedCapabilities::SharingEnabled as u32
      + ExtendedCapabilities::_0x4000 as u32
      + ExtendedCapabilities::SupportsPeerToPeerMixerRelay as u32
      + ExtendedCapabilities::_0x10000 as u32
      + ExtendedCapabilities::ConvWindowFileTransfer as u32
      + ExtendedCapabilities::SupportsPeerToPeerEnveloping as u32
      + ExtendedCapabilities::_0x100000 as u32
      + ExtendedCapabilities::SIPTunnelVersion2 as u32
      + ExtendedCapabilities::SupportsSocialNewsObjectTypes as u32
      + ExtendedCapabilities::CustomEmoticonsCapable as u32
      + ExtendedCapabilities::SupportsUTF8MoodMessages as u32
      + ExtendedCapabilities::SupportsP4Activity as u32
      + ExtendedCapabilities::SupportsMultipartyConversations as u32
      + ExtendedCapabilities::VoiceClipSupportsCircleIM as u32;


  //ClientCapabilities::new(STANDARD_CAP, EXTENDED_CAP)
        ClientCapabilities::new(2789003324, 48)
  }
}

impl ClientCapabilities {

    pub fn new(capabilities: u32, extended_capabilities: u32) -> Self{
        return ClientCapabilities{ capabilities, extended_capabilities };
    }

    pub fn supports(&self, capability: Capabilities) -> bool {
       let cap_as_int = capability as u32;
       let and = self.capabilities & cap_as_int;
       return and == cap_as_int
    }

    pub fn supports_ext(&self, capability: ExtendedCapabilities) -> bool {
        let cap_as_int = capability as u32;
        let and = self.extended_capabilities & cap_as_int;
        return and == cap_as_int;
    }

}

impl FromStr for ClientCapabilities {
    type Err = PayloadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = s.split(":").collect();
        let cap : u32 = split.get(0).unwrap_or(&"0").parse().map_err(|e : ParseIntError| PayloadError::StringPayloadParsingError { payload: s.to_string(), source: anyhow!(e) } )?;
        let cap_ext : u32 = split.get(1).unwrap_or(&"0").parse().map_err(|e: ParseIntError| PayloadError::StringPayloadParsingError { payload: s.to_string(), source: anyhow!(e) } )?;
        return Ok(ClientCapabilities::new(cap, cap_ext));
    }
}

impl Display for ClientCapabilities {
    
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}:{}", &self.capabilities, &self.extended_capabilities);
    }

}

impl yaserde::YaDeserialize for ClientCapabilities {
    
    fn deserialize<R: std::io::Read>(reader: &mut yaserde::de::Deserializer<R>) -> Result<Self, String> {
        if let xml::reader::XmlEvent::StartElement { name, .. } = reader.peek()?.to_owned() {
            let expected_name = "Capabilities".to_owned();
            if name.local_name != expected_name {
              return Err(format!(
                "Wrong StartElement name: {}, expected: {}",
                name, expected_name
              ));
            }
            let _next = reader.next_event();
          } else {
            return Err("StartElement missing".to_string());
          }    

          if let xml::reader::XmlEvent::Characters(text) = reader.peek()?.to_owned() {

            let text_parsed : ClientCapabilities = FromStr::from_str(text.as_str()).or(Err("Couldn't deserialize capabilities"))?;
  
            Ok(text_parsed)
          } else {
            Err("Characters missing".to_string())
          }
        }

}

impl yaserde::YaSerialize for ClientCapabilities {
    fn serialize<W: std::io::Write>(&self, writer: &mut yaserde::ser::Serializer<W>) -> Result<(), String> {
        let _ret = writer.write(xml::writer::XmlEvent::start_element("Capabilities"));
        let _ret = writer.write(xml::writer::XmlEvent::characters(
          &self.to_string(),
        ));
        let _ret = writer.write(xml::writer::XmlEvent::end_element());
        Ok(())    
    }

    fn serialize_attributes(
    &self,
    attributes: Vec<xml::attribute::OwnedAttribute>,
    namespace: xml::namespace::Namespace,
  ) -> Result<
    (
      Vec<xml::attribute::OwnedAttribute>,
      xml::namespace::Namespace,
    ),
    String,
  > {
    Ok((attributes, namespace))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;



    use crate::shared::models::capabilities::{Capabilities, ExtendedCapabilities};

    use super::ClientCapabilities;

    #[test]
    fn test_deserialize() {


       let result = ClientCapabilities::from_str("2789003324:48").unwrap();

        assert!(result.supports(Capabilities::MsgrVersion10));
        assert!(result.supports(Capabilities::MobileOnline) == false);
        assert!(result.supports_ext(ExtendedCapabilities::RTCVideoEnabled));

    }

    #[test]
    fn test_serialize() {

        let result = ClientCapabilities::default();
        assert!(result.supports(Capabilities::MsgrVersion10));

    }
}

