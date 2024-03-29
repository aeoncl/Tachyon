use std::{fmt::{self, Display}, str::FromStr};

use anyhow::anyhow;
use base64::{Engine, engine::general_purpose};
use byteorder::ByteOrder;
use log::warn;
use sha1::{Digest, Sha1};
use strum_macros::EnumString;
use substring::Substring;
use yaserde::{de::{self, from_str}, ser::to_string_with_config};
use yaserde_derive::{YaDeserialize, YaSerialize};

use crate::models::tachyon_error::PayloadError;
use crate::utils::identifiers::compute_sha1;
use crate::utils::string::map_empty_string_to_option;

use super::p2p::slp_context::SlpContext;

// Documentation sauce: https://wiki.nina.chat/wiki/Protocols/MSNP/MSNC/MSN_Object


#[derive(Clone, Debug)]
pub struct MSNObject {
    //MSN Address of the creator or sender
    pub creator: String,
    //Size in bytes
    pub size: u32,
    //Type
    pub obj_type: MSNObjectType,
    //The Location field contains the filename under which the file will be, or has been, stored. 0 For in the storage service.
    pub location: String,
    /*This field contains the name of the picture in Unicode (UTF-16 Little Endian) format. 
    The string is then encoded with Base64. For most types of descriptors this field is a null character, 
    or AAA= when encoded.*/
    pub friendly: Option<String>,
    //The SHA1D field contains a SHA1 hash of the images data encoded in Base64. It is also known as the Data Hash or the SHA1 Data Field.
    pub sha1d: String,

    /* The following fields are not included in the sha1c */
    pub contenttype: Option<MSNObjectContentType>,

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

    /* Not serialized  */
    pub compute_sha1c: bool,

}

impl SlpContext for MSNObject {

    fn from_slp_context(bytes: &Vec<u8>) -> Option<Box<Self>> { 
        let base64_decoded = general_purpose::STANDARD.decode(bytes);
        if let Ok(base64_decoded) = base64_decoded {
            if let Ok(str) = String::from_utf8(base64_decoded){
                if let Ok(msn_obj) = MSNObject::from_str(str.as_str()) {
                    return Some(Box::new(msn_obj));
                }
            }
        }
        return None;
    }
}

impl FromStr for MSNObject {
    type Err = PayloadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        from_str::<MSNObject>(s).map_err(|e| PayloadError::StringPayloadParsingError { payload: s.to_string(), sauce: anyhow!("Could not parse string to MSNObject for : {}", e) })
    }
}

impl yaserde::YaDeserialize for MSNObject {

    
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
            let mut friendly = Option::None;
            let mut contenttype = Option::None;
            let mut contentid = Option::None;
            let mut partnerid = Option::None;
            let mut stamp = Option::None;
            let mut avatarid = Option::None;
            let mut avatarcontentid = Option::None;

            for attribute in attributes {
                match attribute.name.local_name.as_str() {
                    "Creator"=> {
                        creator = Some(attribute.value);
                    },
                    "Size" => {
                        size = attribute.value.parse::<u32>().expect("Size to be an unsigned number");
                    },
                    "Type" => {
                        let test = attribute.value.as_str();
                       obj_type = Some(MSNObjectType::from_str(attribute.value.as_str()).map_err(|e| e.to_string())?);
                    },
                    "SHA1D" => {
                       sha1d = Some(attribute.value);
                    },
                    "Location" => {
                        location = Some(attribute.value);
                    },
                    "Friendly" => {
                        let decoded = Self::decode_friendly(&attribute.value);
                        friendly = map_empty_string_to_option(decoded);
                    },
                    "SHA1C" => {
                        //Do nothing with this for now
                    },
                    "contenttype" => {
                        contenttype = Some(MSNObjectContentType::from_str(attribute.value.as_str()).map_err(|e| e.to_string())?);
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
                        warn!("Unsupported MSNObj field for deserialization: {:?}", &attribute);
                    }
                }
            };

            return Ok(MSNObject {
                creator: creator.expect("Creator to be present in a MSNObject"),
                size,
                obj_type: obj_type.expect("MSNObj to have a type"),
                location: location.expect("MSNObj to have a location"),
                friendly,
                sha1d: sha1d.expect("MSNObject to have SHA1D"),
                contenttype,
                contentid,
                partnerid,
                stamp,
                avatarid,
                avatarcontentid,
                compute_sha1c: false
            });
        
        };

        return Err("No start element?? Is this even XML ?? ARE YOU EVEN TRYING ???".into());

    }
}

impl yaserde::YaSerialize for &MSNObject {
    fn serialize<W: std::io::Write>(&self, writer: &mut yaserde::ser::Serializer<W>) -> Result<(), String> {
        let size = self.size.to_string();
        let obj_type = self.obj_type.to_string();
        let friendly = self.get_friendly_base64_or_default();
        let mut elem = xml::writer::XmlEvent::start_element("msnobj")
            .attr("Creator", &self.creator).attr("Type", &obj_type)
            .attr("SHA1D", &self.sha1d).attr("Size", &size)
            .attr("Location", &self.location)
            .attr("Friendly", &friendly);

        let sha1c = self.get_sha1c();

        if self.compute_sha1c {
            elem = elem.attr("SHA1C", &sha1c);
        }

        let contentype_serialized = self.contenttype.as_ref().unwrap_or(&MSNObjectContentType::D).to_string();
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

impl MSNObject {
    pub fn new(creator: String, obj_type: MSNObjectType, location: String, sha1d: String, size: usize, friendly: Option<String>, contenttype: Option<MSNObjectContentType>, compute_sha1c: bool) -> Self {
        return Self{ creator, size: size.try_into().unwrap(), obj_type, location, friendly, sha1d, contenttype, contentid: None, partnerid: None, stamp: None, avatarid: None, avatarcontentid: None, compute_sha1c };
    }

    fn get_friendly_base64_or_default(&self) -> String {
        if self.friendly.is_none() {
            return String::from("AAA=");
        }

        let utf8_friendly = self.friendly.as_ref().unwrap();
        
        if utf8_friendly.is_empty() {
            return String::from("AAA=");
        }

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
        return general_purpose::STANDARD.encode(&out);
    }

    fn decode_friendly(friendly: &str) -> String {
        if friendly == "AAA="{
            return String::default();
        }

       let mut decoded = general_purpose::STANDARD.decode(friendly).unwrap(); //TODO HANDLE ERRORS OH MY GOD WHY ARE YOU ALWAYS GIVING FUTURE YOU SO MUCH WORK
       let padding1 = decoded.pop().expect("Padding 1 to be present");
       let padding2 = decoded.pop().expect("Padding 2 to be present");

       if padding1 != 0 || padding2 != 0 {
         panic!("WHERE IS MY PADDING GODDAMMIT");
       };

        let utf16: Vec<u16> = decoded.chunks_exact(2).map(|s| u16::from_le_bytes(s.try_into().expect("Exact chunk size to be exact"))).collect();
        return String::from_utf16(&utf16).unwrap();
    }


    fn get_sha1c(&self) -> String {
        let sha1_input = format!("Creator{creator}Type{obj_type}SHA1D{sha1d}Size{size}Location{location}Friendly{friendly}", creator = &self.creator, size = &self.size, obj_type = self.obj_type.clone() as i32, location = &self.location, friendly = self.get_friendly_base64_or_default(), sha1d = &self.sha1d);
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


    /**
     * <msnobj Creator="aeoncl1@shlasouf.local" Type="3" SHA1D="1u3ees4vbHswj+6ud7vJ1g24Lz0=" Size="25810" Location="0" Friendly="RgBsAGEAcgBlAAAA" contenttype="D"/>     * 
     */



impl Display for MSNObject {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        let yaserde_cfg = yaserde::ser::Config{
            perform_indent: false,
            write_document_declaration: false,
            indent_string: None
        };

        let serialized = to_string_with_config(&self, &yaserde_cfg).unwrap();
        return write!(f, "{}", urlencoding::encode(serialized.as_str()));
    }
}

#[derive(Clone, Debug, EnumString, PartialEq)]
pub enum MSNObjectContentType {
    /* Paid, prevents other users to add it */
    P,
    /* Downloadable, Free */
    D
}

impl fmt::Display for MSNObjectContentType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for MSNObjectContentType {
    fn default() -> Self {
        return Self::D;
    }
}


#[derive(Clone, Debug, PartialEq)]
pub enum MSNObjectType {
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


impl FromStr for MSNObjectType {
    type Err = PayloadError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed: i32 = s.parse()?;
        Ok(MSNObjectType::try_from(parsed)?)
    }
}


impl Display for MSNObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.clone() as i32);   
    }
}

impl TryFrom<i32> for MSNObjectType {
    type Error = PayloadError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            x if x == MSNObjectType::Avatar as i32 => Ok(MSNObjectType::Avatar),
            x if x == MSNObjectType::CustomEmoticon as i32 => Ok(MSNObjectType::CustomEmoticon),
            x if x == MSNObjectType::DisplayPicture as i32 => Ok(MSNObjectType::DisplayPicture),
            x if x == MSNObjectType::SharedFile as i32 => Ok(MSNObjectType::SharedFile),
            x if x == MSNObjectType::Background as i32 => Ok(MSNObjectType::Background),
            x if x == MSNObjectType::History as i32 => Ok(MSNObjectType::History),
            x if x == MSNObjectType::DynamicDisplayPicture as i32 => Ok(MSNObjectType::DynamicDisplayPicture),
            x if x == MSNObjectType::Wink as i32 => Ok(MSNObjectType::Wink),
            x if x == MSNObjectType::MapFile as i32 => Ok(MSNObjectType::MapFile),
            x if x == MSNObjectType::DynamicBackground as i32 => Ok(MSNObjectType::DynamicBackground),
            x if x == MSNObjectType::VoiceClip as i32 => Ok(MSNObjectType::VoiceClip),
            x if x == MSNObjectType::PluginState as i32 => Ok(MSNObjectType::PluginState),
            x if x == MSNObjectType::RoamingObject as i32 => Ok(MSNObjectType::RoamingObject),
            x if x == MSNObjectType::SignatureSound as i32 => Ok(MSNObjectType::SignatureSound),
            x if x == MSNObjectType::UnknownYet as i32 => Ok(MSNObjectType::UnknownYet),
            x if x == MSNObjectType::Scene as i32 => Ok(MSNObjectType::Scene),
            x if x == MSNObjectType::WebcamDynamicDisplayPicture as i32 => Ok(MSNObjectType::WebcamDynamicDisplayPicture),
            _ => {
                Err(PayloadError::EnumParsingError { payload: value.to_string(), sauce: anyhow!("Couldn't parse int to MSNObjectType") })
            }
        }
    }
}

pub struct MSNObjectFactory;

impl MSNObjectFactory {

    pub fn get_display_picture(image: &[u8], creator_msn_addr: String, location: String, friendly: Option<String>) -> MSNObject {
        let sha1d = compute_sha1(&image);

        return MSNObject::new(creator_msn_addr, MSNObjectType::DisplayPicture, location, sha1d, image.len(), friendly, Some(MSNObjectContentType::D), false);
    }

    pub fn get_me_display_picture(image: &[u8], creator_msn_addr: String, friendly: Option<String>) -> MSNObject {
        let sha1d = compute_sha1(&image);
        return MSNObject::new(creator_msn_addr, MSNObjectType::DisplayPicture, "0".into(), sha1d, image.len(), friendly, None, false);
    }

    pub fn get_voice_message(data: &[u8], creator_msn_addr: String, friendly: Option<String>) -> MSNObject {
        let sha1d = compute_sha1(&data);
        return MSNObject::new(creator_msn_addr, MSNObjectType::VoiceClip,"0".into(), sha1d, data.len(),  friendly, None, false);
    }

    pub fn get_contact_display_picture(image: &[u8], creator_msn_addr: String, location: String, friendly: Option<String>) -> MSNObject {
        let sha1d = compute_sha1(&image);

        return MSNObject::new(creator_msn_addr, MSNObjectType::DisplayPicture, location, sha1d, image.len(), friendly, Some(MSNObjectContentType::D), false);
    }
}

mod tests {
    use std::str::FromStr;

    use crate::models::{msn_object::{MSNObject, MSNObjectType, MSNObjectContentType}, p2p::slp_context::SlpContext};
    use crate::utils::identifiers::compute_sha1;

    use super::MSNObjectFactory;

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

        let obj = MSNObjectFactory::get_display_picture(&AVATAR_BYTES, String::from("aeoncl1@shlasouf.local"), String::from("0"), Some(String::from("Flare")));
        let friendly_b64 = obj.get_friendly_base64_or_default();
        
        let obj_serialized = obj.to_string();

        let str = "%3Cmsnobj%20Creator%3D%22aeoncl1%40shlasouf.local%22%20Type%3D%223%22%20SHA1D%3D%221u3ees4vbHswj%2B6ud7vJ1g24Lz0%3D%22%20Size%3D%2225810%22%20Location%3D%220%22%20Friendly%3D%22RgBsAGEAcgBlAAAA%22%20contenttype%3D%22D%22%20%2F%3E";
        assert_eq!(obj_serialized, str);
    }

    #[test]
    fn friendly_not_empty() {

        let obj = MSNObjectFactory::get_display_picture(&AVATAR_BYTES, String::from("aeoncl1@shlasouf.local"), String::from("0"), Some(String::from("Flare")));
        let friendly_b64 = obj.get_friendly_base64_or_default();
        
        assert_eq!(&friendly_b64, "RgBsAGEAcgBlAAAA");

        let str = "%3Cmsnobj%20Creator%3D%22aeoncl1%40shlasouf.local%22%20Type%3D%223%22%20SHA1D%3D%221u3ees4vbHswj%2B6ud7vJ1g24Lz0%3D%22%20Size%3D%2225810%22%20Location%3D%220%22%20Friendly%3D%22RgBsAGEAcgBlAAAA%22%20contenttype%3D%22D%22%2F%3E";
    }

    #[test]
    fn friendly_empty() {
        let obj = MSNObjectFactory::get_display_picture(&AVATAR_BYTES, String::from("aeoncl1@shlasouf.local"), String::from("0"), Some(String::new()));
        let friendly_b64 = obj.get_friendly_base64_or_default();
        
        assert_eq!(&friendly_b64, "AAA=");
    }

    #[test]
    fn friendly_none() {

        let obj = MSNObjectFactory::get_display_picture(&AVATAR_BYTES, String::from("aeoncl1@shlasouf.local"), String::from("0"), None);
        let friendly_b64 = obj.get_friendly_base64_or_default();
        
        assert_eq!(&friendly_b64, "AAA=");
    }

    #[test]
    fn deserialize_test() {
        let base64_context = String::from("PG1zbm9iaiBDcmVhdG9yPSJidWRkeTFAaG90bWFpbC5jb20iIFNpemU9IjI0NTM5IiBUeXBlPSIzIiBMb2NhdGlvbj0iVEZSMkMudG1wIiBGcmllbmRseT0iQUFBPSIgU0hBMUQ9InRyQzhTbEZ4MnNXUXhaTUlCQVdTRW5YYzhvUT0iIFNIQTFDPSJVMzJvNmJvc1p6bHVKcTgyZUF0TXB4NWRJRUk9Ii8+DQoA");
        let msn_obj = MSNObject::from_slp_context(&base64_context.as_bytes().to_vec()).unwrap();
        assert_eq!(msn_obj.avatarcontentid, None);
        assert_eq!(msn_obj.avatarid, None);
        assert_eq!(msn_obj.compute_sha1c, false);
        assert_eq!(msn_obj.contentid, None);
        assert!(msn_obj.contenttype == None);
        assert_eq!(msn_obj.creator.as_str(), "buddy1@hotmail.com");
        assert_eq!(msn_obj.size, 24539);
        assert!(msn_obj.obj_type == MSNObjectType::DisplayPicture);
        assert_eq!(msn_obj.location.as_str(), "TFR2C.tmp");
        assert_eq!(msn_obj.sha1d.as_str(), "trC8SlFx2sWQxZMIBAWSEnXc8oQ=");
        println!("{:?}", &msn_obj);
    }

    #[test]
    fn testttt() {
        let str = "bQBzAG4AbQBzAGcAcgBfADIAMAAyADMAXwAxADEAXwAxAF8AMQAyAF8AMwA4AF8ANAAxAF8ANQA4ADEAXwAyAAAA";
        let test = MSNObject::decode_friendly(str);
        println!("Friendly: {}", test);

    }
    #[test]
    fn serialize_deserialize_test() {

        let msn_obj = MSNObject {
            creator: "xx-aeon-xx@lukewarmail.com".into(),
            size: 1989,
            obj_type: MSNObjectType::DynamicDisplayPicture,
            location: "0".into(),
            friendly: Some("Offspring Skull on fire".into()),
            sha1d: "weUn1teT0Wr1te0urC0d3".into(),
            contenttype: Some(MSNObjectContentType::D),
            contentid: Some("xobubble".into()),
            partnerid: Some("gianthard corporations".into()),
            stamp: Some("stampo".into()),
            avatarid: Some("aang".into()),
            avatarcontentid: Some("fire nation attacks".into()),
            compute_sha1c: true,
        };

        let serialized = msn_obj.to_string_not_encoded();
        let deserialized = MSNObject::from_str(serialized.as_str()).unwrap();

        assert_eq!(deserialized.avatarcontentid, Some("fire nation attacks".into()));
        assert_eq!(deserialized.avatarid, Some("aang".into()));
        assert_eq!(deserialized.compute_sha1c, false);
        assert_eq!(deserialized.contentid, Some("xobubble".into()));
        assert_eq!(deserialized.contenttype, Some(MSNObjectContentType::D));
        assert_eq!(deserialized.creator.as_str(), "xx-aeon-xx@lukewarmail.com");
        assert_eq!(deserialized.size, 1989);
        assert_eq!(deserialized.obj_type, MSNObjectType::DynamicDisplayPicture);
        assert_eq!(deserialized.location.as_str(), "0");
        assert_eq!(deserialized.sha1d.as_str(), "weUn1teT0Wr1te0urC0d3");


        println!("Original: {:?}", msn_obj);

        println!("Serialized: {:?}", serialized);

        println!("Deserialized: {:?}", &deserialized);
    }

}