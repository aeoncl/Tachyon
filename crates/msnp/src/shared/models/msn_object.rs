use std::{fmt::{self, Display}, str::FromStr};
use std::fmt::Formatter;

use anyhow::anyhow;
use base64::{Engine, engine::general_purpose, write};
use byteorder::ByteOrder;
use log::{debug, warn};
use sha1::{Digest, Sha1};
use strum_macros::EnumString;
use yaserde::{de::{self, from_str}, ser::to_string_with_config};
use yaserde_derive::{YaDeserialize, YaSerialize};

use crate::{msnp::error::PayloadError, p2p::v2::slp_context::SlpContext};
use crate::msnp::error::CommandError;


// Documentation source: https://wiki.nina.chat/wiki/Protocols/MSNP/MSNC/MSN_Object


#[derive(Clone, Debug)]
pub struct MsnObject {
    //MSN Address of the creator or sender
    pub creator: String,
    //Size in bytes
    pub size: usize,
    //Type
    pub obj_type: MsnObjectType,
    //The Location field contains the filename under which the file will be, or has been, stored. 0 For in the storage service.
    pub location: String,
    /*This field contains the name of the picture in Unicode (UTF-16 Little Endian) format. 
    The string is then encoded with Base64. For most types of descriptors this field is a null character, 
    or AAA= when encoded.*/
    pub friendly: FriendlyName,
    //The SHA1D field contains a SHA1 hash of the images data encoded in Base64. It is also known as the Data Hash or the SHA1 Data Field.
    pub sha1d: String,

    /* The following fields are not included in the sha1c */
    pub contenttype: Option<MsnObjectContentType>,

    // The contentid field contains the identifier of the item which the Content Partner gave it.
    pub contentid: Option<String>,

    /* The partnerid field contains the name of the Content Partner. 
    American Greetings/Blue Mountain for example, is identified by AG. 
    This field applies only for Custom Emoticons. 
    But if this field is not present in the Content XML file when it was downloaded, the value of this field is -1. */
    pub partnerid: Option<String>,

    /* The stamp field contains the Base64 encoded signature of the file. 
    It is a S/MIME signature of the Base64 encoded hash of the Content cabinet file and is signed by the MSN Content Authority. 
    The stamp field is used in the Content XML file to make sure that the file isn't modified. 
    Currently, it's only being included in the MSN Object for dynamic (Flash) contents like Winks and Dynamic Display Pictures.  
    */
    pub stamp: Option<String>,

    /* Down level fields */
    /* The avatarid field indicates what type of avatar is being referenced to. This field is currently only being seen with {6AD16E96-BC60-401B-89E2-5BB545DD2BF0}  */
    pub avatarid: Option<String>,

    /* The avatarcontentid field contains the SHA-1 hash of the cabinet file which contains the content and is later encoded with Base64 as well. */
    pub avatarcontentid: Option<String>,

    //The SHA1C field contains a SHA1 hash of all the properties of the MSMObject, to check integrity.
    //This field is deserialized but computed at serialization.
    pub sha1c: String,

    /* Not serialized  */
    pub compute_sha1c: bool,

}

impl SlpContext for MsnObject {

    fn from_slp_context(bytes: &[u8]) -> Option<Self> {
        let base64_decoded = general_purpose::STANDARD.decode(bytes);
        if let Ok(base64_decoded) = base64_decoded {
            if let Ok(str) = String::from_utf8(base64_decoded){
                if let Ok(msn_obj) = MsnObject::from_str(str.as_str()) {
                    return Some(msn_obj);
                }
            }
        }
        return None;
    }
}

impl FromStr for MsnObject {
    type Err = PayloadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        from_str::<MsnObject>(&s).map_err(|e| PayloadError::StringPayloadParsingError { payload: s.to_string(), source: anyhow!("Could not parse string to MSNObject for : {}", e) })
    }
}

impl yaserde::YaDeserialize for MsnObject {

    
    fn deserialize<R: std::io::Read>(reader: &mut de::Deserializer<R>) -> Result<Self, String> {
        if let xml::reader::XmlEvent::StartElement { name, attributes, namespace } = reader.peek()?.to_owned() {
            let expected_name = "msnobj";
            if name.local_name != expected_name {
                return Err(format!(
                    "Wrong StartElement name: {}, expected: {}",
                    name, expected_name
                  ));
            }

            let mut creator = Option::None;
            let mut size = 0;
            let mut obj_type = Option::None;
            let mut sha1d = Option::None;
            let mut location = Option::None;
            let mut friendly = FriendlyName::default();
            let mut contenttype = Option::None;
            let mut contentid = Option::None;
            let mut partnerid = Option::None;
            let mut stamp = Option::None;
            let mut avatarid = Option::None;
            let mut avatarcontentid = Option::None;
            let mut sha1c = Option::None;


            for attribute in attributes {
                match attribute.name.local_name.as_str() {
                    "Creator"=> {
                        creator = Some(attribute.value);
                    },
                    "Size" => {
                        size = attribute.value.parse::<usize>().map_err(|e| format!("Error while parsing Size attribute: {}", e.to_string()))?
                    },
                    "Type" => {
                       obj_type = Some(MsnObjectType::from_str(attribute.value.as_str()).map_err(|e| e.to_string())?);
                    },
                    "SHA1D" => {
                       sha1d = Some(attribute.value);
                    },
                    "Location" => {
                        location = Some(attribute.value);
                    },
                    "Friendly" => {
                        friendly = FriendlyName::from_str(&attribute.value).map_err(|e| format!("Error while parsing Friendly: {}", e.to_string()))?;
                    },
                    "SHA1C" => {
                        sha1c = Some(attribute.value)
                    },
                    "contenttype" => {
                        contenttype = Some(MsnObjectContentType::from_str(attribute.value.as_str()).map_err(|e| e.to_string())?);
                    },
                    "contentid" => {
                        contentid =  map_empty_string_to_option(attribute.value);
                    },
                    "partnerid" => {
                        partnerid = map_empty_string_to_option(attribute.value);
                    },
                    "stamp" => {
                        stamp = map_empty_string_to_option(attribute.value);
                    },
                    "avatarid" => {
                        avatarid = map_empty_string_to_option(attribute.value);
                    },
                    "avatarcontentid" => {
                        avatarcontentid = map_empty_string_to_option(attribute.value);
                    }
                    _=> {
                        warn!("MSNP|MSNObject Unsupported field for deserialization: {:?}", &attribute);
                    }
                }
            };

            Ok(MsnObject {
                creator: creator.ok_or("Missing mandatory field: Creator")?,
                size,
                obj_type: obj_type.ok_or("Missing mandatory field: ObjType")?,
                location: location.ok_or("Missing mandatory field: Location")?,
                friendly,
                sha1d: sha1d.ok_or("Missing mandatory field: SHA1D")?,
                contenttype,
                contentid,
                partnerid,
                stamp,
                avatarid,
                avatarcontentid,
                sha1c: sha1c.unwrap_or_default(),
                compute_sha1c: false
            })
        
        } else {
            Err("Missing MSNObj XML Start element".into())
        }
    }
}

impl yaserde::YaSerialize for &MsnObject {
    fn serialize<W: std::io::Write>(&self, writer: &mut yaserde::ser::Serializer<W>) -> Result<(), String> {
        let size = self.size.to_string();
        let obj_type = self.obj_type.to_string();
        let friendly = self.friendly.to_string();
        let mut elem = xml::writer::XmlEvent::start_element("msnobj")
            .attr("Creator", &self.creator).attr("Type", &obj_type)
            .attr("SHA1D", &self.sha1d).attr("Size", &size)
            .attr("Location", &self.location)
            .attr("Friendly", &friendly);

        let sha1c = self.get_sha1c();
        if self.compute_sha1c {
            elem = elem.attr("SHA1C", &sha1c);
        }

        let contentype_serialized = self.contenttype.as_ref().unwrap_or(&MsnObjectContentType::D).to_string();
        if self.contenttype.is_some() {
            elem = elem.attr("contenttype", contentype_serialized.as_str());
        }

        if let Some(contentid) = self.contentid.as_ref() {
            elem = elem.attr("contentid", contentid.as_str());
        }

        if let Some(partnerid) = self.partnerid.as_ref() {
            elem = elem.attr("partnerid", partnerid.as_str());
        }

        if let Some(stamp) = self.stamp.as_ref() {
            elem = elem.attr("stamp", stamp.as_str());
        }

        if let Some(avatarid) = self.avatarid.as_ref(){
            elem=elem.attr("avatarid", avatarid.as_str());
        }

        if let Some(avatarcontentid) = self.avatarcontentid.as_ref(){
            elem = elem.attr("avatarcontentid", avatarcontentid.as_str());
        }

        let _ret = writer.write(elem);
        let _ret = writer.write(xml::writer::XmlEvent::end_element());
        return Ok(());
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

impl MsnObject {
    pub fn new(creator: String, obj_type: MsnObjectType, location: String, sha1d: String, size: usize, friendly: FriendlyName, contenttype: Option<MsnObjectContentType>, compute_sha1c: bool) -> Self {
        return Self{ creator, size, obj_type, location, friendly, sha1d, contenttype, contentid: None, partnerid: None, stamp: None, avatarid: None, avatarcontentid: None, sha1c: String::default(), compute_sha1c };
    }

    fn get_sha1c(&self) -> String {
        if !self.sha1c.is_empty() {
            return self.sha1c.clone();
        }

        let sha1_input = format!("Creator{creator}Type{obj_type}SHA1D{sha1d}Size{size}Location{location}Friendly{friendly}", creator = &self.creator, size = &self.size, obj_type = self.obj_type.clone() as i32, location = &self.location, friendly = self.friendly, sha1d = &self.sha1d);
        return compute_sha1(sha1_input.as_bytes());
    }

    pub fn to_string_not_encoded(&self) -> String {
        let yaserde_cfg = yaserde::ser::Config{
            perform_indent: false,
            write_document_declaration: false,
            indent_string: None
        };

        to_string_with_config(&self, &yaserde_cfg).unwrap()
    }
}



#[derive(Clone, Debug)]
pub struct FriendlyName(String);

impl FriendlyName {

    pub fn new(name: &str) -> Self{
        FriendlyName(name.to_string())
    }

}

impl Display for FriendlyName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {

        if self.0.is_empty() {
           return write!(f, "{}", "AAA=");
        }

        let utf8_friendly = &self.0;

        let utf16 : Vec<u16> = utf8_friendly.encode_utf16().collect();

        let mut out: Vec<u8> = Vec::with_capacity((utf16.len()*2) + 2);
        for current in utf16 {
            let utf8_array = current.to_le_bytes();
            for current_utf8 in utf8_array {
                out.push(current_utf8);
            }
        }

        //Padding
        out.push(0);
        out.push(0);

        write!(f, "{}", general_purpose::STANDARD.encode(&out))
    }
}

impl FromStr for FriendlyName {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {

        if s.is_empty() || s == "AAA="{
            return Ok(FriendlyName(String::default()));
        }

        let mut decoded = general_purpose::STANDARD.decode(s).map_err(|e| PayloadError::PayloadPropertyParseError {
            property_name: "Friendly".to_string(),
            raw_value: s.to_string(),
            payload_type: "MSNObject".to_string(),
            source: e.into(),
        })?;

        let padding1 = decoded.pop().ok_or(PayloadError::PayloadPropertyParseError {
            property_name: "Friendly".to_string(),
            raw_value: s.to_string(),
            payload_type: "MSNObject".to_string(),
            source: anyhow!("Missing first byte of padding"),
        })?;

        let padding2 = decoded.pop().ok_or(PayloadError::PayloadPropertyParseError {
            property_name: "Friendly".to_string(),
            raw_value: s.to_string(),
            payload_type: "MSNObject".to_string(),
            source: anyhow!("Missing second byte of padding"),
        })?;

        if padding1 != 0 || padding2 != 0 {
            Err(PayloadError::PayloadPropertyParseError {
                property_name: "Friendly".to_string(),
                raw_value: s.to_string(),
                payload_type: "MSNObject".to_string(),
                source: anyhow!("Padding is supposed to be 0s"),
            })?;
        };

        let utf16: Vec<u16> = decoded.chunks_exact(2).map(|s| u16::from_le_bytes(s.try_into().expect("Chunk to be of correct size"))).collect();

        let parsed = String::from_utf16(&utf16).map_err(|e| PayloadError::PayloadPropertyParseError {
            property_name: "Friendly".to_string(),
            raw_value: s.to_string(),
            payload_type: "MSNObject".to_string(),
            source: e.into(),
        })?;

        Ok(FriendlyName(parsed))
    }
}

impl Default for FriendlyName {
    fn default() -> Self {
        FriendlyName(String::default())
    }
}


    /**
     * <msnobj Creator="aeoncl1@shlasouf.local" Type="3" SHA1D="1u3ees4vbHswj+6ud7vJ1g24Lz0=" Size="25810" Location="0" Friendly="RgBsAGEAcgBlAAAA" contenttype="D"/>     * 
     */



impl Display for MsnObject {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        let yaserde_cfg = yaserde::ser::Config{
            perform_indent: false,
            write_document_declaration: false,
            indent_string: None
        };

        let serialized = to_string_with_config(&self, &yaserde_cfg).unwrap();
        return write!(f, "{}", serialized.as_str());
    }
}

#[derive(Clone, Debug, EnumString, PartialEq)]
pub enum MsnObjectContentType {
    /* Paid, prevents other users to add it */
    P,
    /* Downloadable, Free */
    D
}

impl fmt::Display for MsnObjectContentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for MsnObjectContentType {
    fn default() -> Self {
        return Self::D;
    }
}




#[derive(Clone, Debug, PartialEq)]
pub enum MsnObjectType {
    //Unknown but present since MSN 6.0
    Avatar = 1,
    CustomEmoticon = 2,
    DisplayPicture = 3,
    //Unknown but present since MSN 6.0
    SharedFile = 4,
    Background = 5,
    //Unknown
    History = 6,
    DynamicDisplayPicture=7,
    Wink=8,
    //A map file contains a list of items in the store
    MapFile=9,
    DynamicBackground=10,
    VoiceClip=11,
    PluginState=12,
    RoamingObject=13,
    SignatureSound=14,
    UnknownYet=15,
    Scene=16,
    WebcamDynamicDisplayPicture=17
}


impl FromStr for MsnObjectType {
    type Err = PayloadError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed: i32 = s.parse()?;
        Ok(MsnObjectType::try_from(parsed)?)
    }
}


impl Display for MsnObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.clone() as i32);   
    }
}

impl TryFrom<i32> for MsnObjectType {
    type Error = PayloadError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            x if x == MsnObjectType::Avatar as i32 => Ok(MsnObjectType::Avatar),
            x if x == MsnObjectType::CustomEmoticon as i32 => Ok(MsnObjectType::CustomEmoticon),
            x if x == MsnObjectType::DisplayPicture as i32 => Ok(MsnObjectType::DisplayPicture),
            x if x == MsnObjectType::SharedFile as i32 => Ok(MsnObjectType::SharedFile),
            x if x == MsnObjectType::Background as i32 => Ok(MsnObjectType::Background),
            x if x == MsnObjectType::History as i32 => Ok(MsnObjectType::History),
            x if x == MsnObjectType::DynamicDisplayPicture as i32 => Ok(MsnObjectType::DynamicDisplayPicture),
            x if x == MsnObjectType::Wink as i32 => Ok(MsnObjectType::Wink),
            x if x == MsnObjectType::MapFile as i32 => Ok(MsnObjectType::MapFile),
            x if x == MsnObjectType::DynamicBackground as i32 => Ok(MsnObjectType::DynamicBackground),
            x if x == MsnObjectType::VoiceClip as i32 => Ok(MsnObjectType::VoiceClip),
            x if x == MsnObjectType::PluginState as i32 => Ok(MsnObjectType::PluginState),
            x if x == MsnObjectType::RoamingObject as i32 => Ok(MsnObjectType::RoamingObject),
            x if x == MsnObjectType::SignatureSound as i32 => Ok(MsnObjectType::SignatureSound),
            x if x == MsnObjectType::UnknownYet as i32 => Ok(MsnObjectType::UnknownYet),
            x if x == MsnObjectType::Scene as i32 => Ok(MsnObjectType::Scene),
            x if x == MsnObjectType::WebcamDynamicDisplayPicture as i32 => Ok(MsnObjectType::WebcamDynamicDisplayPicture),
            _ => {
                Err(PayloadError::EnumParsingError { payload: value.to_string(), source: anyhow!("Couldn't parse int to MSNObjectType") })
            }
        }
    }
}

pub struct MSNObjectFactory;

impl MSNObjectFactory {

    pub fn get_display_picture(image: &[u8], creator_msn_addr: String, location: String, friendly: FriendlyName) -> MsnObject {
        let sha1d = compute_sha1(&image);

        return MsnObject::new(creator_msn_addr, MsnObjectType::DisplayPicture, location, sha1d, image.len(), friendly, Some(MsnObjectContentType::D), false);
    }

    pub fn get_me_display_picture(image: &[u8], creator_msn_addr: String, friendly: FriendlyName) -> MsnObject {
        let sha1d = compute_sha1(&image);
        return MsnObject::new(creator_msn_addr, MsnObjectType::DisplayPicture, "0".into(), sha1d, image.len(), friendly, None, false);
    }

    pub fn get_voice_message(data: &[u8], creator_msn_addr: String, friendly: FriendlyName) -> MsnObject {
        let sha1d = compute_sha1(&data);
        return MsnObject::new(creator_msn_addr, MsnObjectType::VoiceClip,"0".into(), sha1d, data.len(),  friendly, None, false);
    }

    pub fn get_contact_display_picture(image: &[u8], creator_msn_addr: String, location: String, friendly: FriendlyName) -> MsnObject {
        let sha1d = compute_sha1(&image);

        return MsnObject::new(creator_msn_addr, MsnObjectType::DisplayPicture, location, sha1d, image.len(), friendly, Some(MsnObjectContentType::D), false);
    }
}


fn compute_sha1(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let result = hasher.finalize();
    return general_purpose::STANDARD.encode(&result);
}

fn map_empty_string_to_option(value: String) -> Option<String> {
    if !value.is_empty() {Some(value)} else {None}
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use lazy_static_include::lazy_static_include_bytes;
    use crate::{p2p::v2::slp_context::SlpContext, shared::models::msn_object::{compute_sha1, MsnObject, MsnObjectContentType, MSNObjectFactory, MsnObjectType}};
    use crate::shared::models::msn_object::FriendlyName;


    lazy_static_include_bytes! {
        AVATAR_BYTES => "assets/img/avatar.jpg"
    }

    /**
     * <msnobj Creator="aeoncl1@shlasouf.local" Type="3" SHA1D="1u3ees4vbHswj+6ud7vJ1g24Lz0=" Size="25810" Location="0" Friendly="RgBsAGEAcgBlAAAA" contenttype="D"/>     * 
     */

    #[test]
    fn compute_sha1d() {
        let sha1d = compute_sha1(&AVATAR_BYTES);
        assert_eq!("1u3ees4vbHswj+6ud7vJ1g24Lz0=", &sha1d);
    }


    #[test]
    fn get_display_picture() {

        let obj = MSNObjectFactory::get_display_picture(&AVATAR_BYTES, String::from("aeoncl1@shlasouf.local"), String::from("0"), FriendlyName::new("Flare"));
        let friendly_b64 = obj.friendly.to_string();
        
        let obj_serialized = obj.to_string();

        let str = "%3Cmsnobj%20Creator%3D%22aeoncl1%40shlasouf.local%22%20Type%3D%223%22%20SHA1D%3D%221u3ees4vbHswj%2B6ud7vJ1g24Lz0%3D%22%20Size%3D%2225810%22%20Location%3D%220%22%20Friendly%3D%22RgBsAGEAcgBlAAAA%22%20contenttype%3D%22D%22%20%2F%3E";
        assert_eq!(obj_serialized, str);
    }

    #[test]
    fn friendly_not_empty() {

        let obj = MSNObjectFactory::get_display_picture(&AVATAR_BYTES, String::from("aeoncl1@shlasouf.local"), String::from("0"), FriendlyName::new("Flare"));
        let friendly_b64 = obj.friendly.to_string();
        
        assert_eq!(&friendly_b64, "RgBsAGEAcgBlAAAA");

        let str = "%3Cmsnobj%20Creator%3D%22aeoncl1%40shlasouf.local%22%20Type%3D%223%22%20SHA1D%3D%221u3ees4vbHswj%2B6ud7vJ1g24Lz0%3D%22%20Size%3D%2225810%22%20Location%3D%220%22%20Friendly%3D%22RgBsAGEAcgBlAAAA%22%20contenttype%3D%22D%22%2F%3E";
    }

    #[test]
    fn friendly_empty() {
        let obj = MSNObjectFactory::get_display_picture(&AVATAR_BYTES, String::from("aeoncl1@shlasouf.local"), String::from("0"), FriendlyName::default());
        let friendly_b64 = obj.friendly.to_string();
        
        assert_eq!(&friendly_b64, "AAA=");
    }

    #[test]
    fn friendly_none() {

        let obj = MSNObjectFactory::get_display_picture(&AVATAR_BYTES, String::from("aeoncl1@shlasouf.local"), String::from("0"), FriendlyName::default());
        let friendly_b64 = obj.friendly.to_string();
        
        assert_eq!(&friendly_b64, "AAA=");
    }

    #[test]
    fn deserialize_test() {
        let base64_context = String::from("PG1zbm9iaiBDcmVhdG9yPSJidWRkeTFAaG90bWFpbC5jb20iIFNpemU9IjI0NTM5IiBUeXBlPSIzIiBMb2NhdGlvbj0iVEZSMkMudG1wIiBGcmllbmRseT0iQUFBPSIgU0hBMUQ9InRyQzhTbEZ4MnNXUXhaTUlCQVdTRW5YYzhvUT0iIFNIQTFDPSJVMzJvNmJvc1p6bHVKcTgyZUF0TXB4NWRJRUk9Ii8+DQoA");
        let msn_obj = MsnObject::from_slp_context(&base64_context.into_bytes()).unwrap();
        assert_eq!(msn_obj.avatarcontentid, None);
        assert_eq!(msn_obj.avatarid, None);
        assert_eq!(msn_obj.compute_sha1c, false);
        assert_eq!(msn_obj.contentid, None);
        assert!(msn_obj.contenttype == None);
        assert_eq!(msn_obj.creator.as_str(), "buddy1@hotmail.com");
        assert_eq!(msn_obj.size, 24539);
        assert!(msn_obj.obj_type == MsnObjectType::DisplayPicture);
        assert_eq!(msn_obj.location.as_str(), "TFR2C.tmp");
        assert_eq!(msn_obj.sha1d.as_str(), "trC8SlFx2sWQxZMIBAWSEnXc8oQ=");
        println!("{:?}", &msn_obj);
    }

    #[test]
    fn testttt() {
        let str = "bQBzAG4AbQBzAGcAcgBfADIAMAAyADMAXwAxADEAXwAxAF8AMQAyAF8AMwA4AF8ANAAxAF8ANQA4ADEAXwAyAAAA";
        let test = FriendlyName::from_str(str).expect("To have worked");
        println!("Friendly: {}", test);

    }
    #[test]
    fn serialize_deserialize_test() {

        let msn_obj = MsnObject {
            creator: "xx-aeon-xx@lukewarmail.com".into(),
            size: 1989,
            obj_type: MsnObjectType::DynamicDisplayPicture,
            location: "0".into(),
            friendly: FriendlyName::new("Offspring Skull on fire"),
            sha1d: "weUn1teT0Wr1te0urC0d3".into(),
            contenttype: Some(MsnObjectContentType::D),
            contentid: Some("xobubble".into()),
            partnerid: Some("gianthard corporations".into()),
            stamp: Some("stampo".into()),
            avatarid: Some("aang".into()),
            avatarcontentid: Some("fire nation attacks".into()),
            sha1c: String::default(),
            compute_sha1c: true,
        };

        let serialized = msn_obj.to_string_not_encoded();
        let deserialized = MsnObject::from_str(serialized.as_str()).unwrap();

        assert_eq!(deserialized.avatarcontentid, Some("fire nation attacks".into()));
        assert_eq!(deserialized.avatarid, Some("aang".into()));
        assert_eq!(deserialized.compute_sha1c, false);
        assert_eq!(deserialized.contentid, Some("xobubble".into()));
        assert_eq!(deserialized.contenttype, Some(MsnObjectContentType::D));
        assert_eq!(deserialized.creator.as_str(), "xx-aeon-xx@lukewarmail.com");
        assert_eq!(deserialized.size, 1989);
        assert_eq!(deserialized.obj_type, MsnObjectType::DynamicDisplayPicture);
        assert_eq!(deserialized.location.as_str(), "0");
        assert_eq!(deserialized.sha1d.as_str(), "weUn1teT0Wr1te0urC0d3");


        println!("Original: {:?}", msn_obj);

        println!("Serialized: {:?}", serialized);

        println!("Deserialized: {:?}", &deserialized);
    }

}