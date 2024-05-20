use std::io::{Read, Write};
use std::str::FromStr;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use xml::attribute::OwnedAttribute;
use xml::namespace::Namespace;
use yaserde::de::Deserializer;
use yaserde::{YaDeserialize, YaSerialize};
use yaserde::ser::Serializer;
use crate::shared::models::oim::OIM;

#[derive(FromPrimitive, Eq, PartialEq, Debug, Clone)]
pub enum NetworkId {
    None = 0,
    WindowsLive = 1,
    OfficeCommunicator = 2,
    Telephone = 4,
    //used by Vodafone
    MobileNetworkInterop = 8,
    Circle = 9,
    TemporaryGroup = 10,
    Cid = 11,
    Connect = 13,
    RemoteNetwork = 14,
    //Jaguire, Japanese mobile interop
    Smtp = 16,
    Yahoo = 32
}

impl YaSerialize for NetworkId {
    fn serialize<W: Write>(&self, writer: &mut Serializer<W>) -> Result<(), String> {
        let _ret = writer.write(xml::writer::XmlEvent::characters(
            format!("{}", self.clone() as u32).as_str(),
        ));

        Ok(())
    }

    fn serialize_attributes(&self, attributes: Vec<OwnedAttribute>, namespace: Namespace) -> Result<(Vec<OwnedAttribute>, Namespace), String> {
        Ok((attributes, namespace))
    }
}

impl YaDeserialize for NetworkId {
    fn deserialize<R: Read>(reader: &mut Deserializer<R>) -> Result<Self, String> {
        if let xml::reader::XmlEvent::StartElement { name, attributes, namespace } = reader.peek()?.to_owned() {
            let _next = reader.next_event();
        }
        if let xml::reader::XmlEvent::Characters(text) = reader.peek()?.to_owned() {
            let raw_network_id = u32::from_str(&text).map_err(|e| e.to_string())?;
            NetworkId::from_u32(raw_network_id).ok_or(format!("Couldnt parse raw Network Id to network Id enum: {}", raw_network_id))
        } else {
            Err("Characters missing".to_string())
        }
    }
}


impl Default for NetworkId {
    fn default() -> Self {
        NetworkId::None
    }
}