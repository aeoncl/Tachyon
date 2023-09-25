use std::{fmt::{self, Display}, str::FromStr};

use base64::{Engine, engine::general_purpose};
use byteorder::ByteOrder;
use sha1::{Digest, Sha1};
use strum_macros::EnumString;
use substring::Substring;
use yaserde::{de::{self, from_str}, ser::to_string_with_config};
use yaserde_derive::{YaDeserialize, YaSerialize};

use super::{errors::Errors, p2p::slp_context::SlpContext};

// Documentation source: https://wiki.nina.chat/wiki/Protocols/MSNP/MSNC/MSN_Object


#[derive(Clone, Debug, YaDeserialize, Default)]
#[yaserde(rename = "msnobj")]
pub struct MSNObject {
    //MSN Address of the creator or sender
    #[yaserde(rename = "Creator", attribute)]
    pub creator: String,
    //Size in bytes
    #[yaserde(rename = "Size", attribute)]
    pub size: u32,
    //Type
    #[yaserde(rename = "Type", attribute)]
    pub obj_type: MSNObjectType,
    //The Location field contains the filename under which the file will be, or has been, stored. 0 For in the storage service.
    #[yaserde(rename = "Location", attribute)]
    pub location: String,
    /*This field contains the name of the picture in Unicode (UTF-16 Little Endian) format. 
    The string is then encoded with Base64. For most types of descriptors this field is a null character, 
    or AAA= when encoded.*/
    #[yaserde(rename = "Friendly", attribute)]
    pub friendly: Option<String>,
    //The SHA1D field contains a SHA1 hash of the images data encoded in Base64. It is also known as the Data Hash or the SHA1 Data Field.
    #[yaserde(rename = "SHA1D", attribute)]
    pub sha1d: String,

    /* The following fields are not included in the sha1c */
    #[yaserde(rename = "contenttype", attribute)]
    pub contenttype: Option<MSNObjectContentType>,

    // The contentid field contains the identifier of the item which the Content Partner gave it.
    #[yaserde(rename = "contentid", attribute)]
    pub contentid: Option<String>,

    /* The partnerid field contains the name of the Content Partner. 
    American Greetings/Blue Mountain for example, is identified by AG. 
    This field applies only for Custom Emoticons. 
    But if this field is not present in the Content XML file when it was downloaded, the value of this field is -1. */
    #[yaserde(rename = "partnerid", attribute)]
    pub partnerid: Option<String>,

    /* The stamp field contains the Base64 encoded signature of the file. 
    It is a S/MIME signature of the Base64 encoded hash of the Content cabinet file and is signed by the MSN Content Authority. 
    The stamp field is used in the Content XML file to make sure that the file isn't modified. 
    Currently, it's only being included in the MSN Object for dynamic (Flash) contents like Winks and Dynamic Display Pictures.  
    */
    #[yaserde(rename = "stamp", attribute)]
    pub stamp: Option<String>,

    /* Down level fields */
    /* The avatarid field indicates what type of avatar is being referenced to. This field is currently only being seen with {6AD16E96-BC60-401B-89E2-5BB545DD2BF0}  */
    #[yaserde(rename = "avatarid", attribute)]
    pub avatarid: Option<String>,

    /* The avatarcontentid field contains the SHA-1 hash of the cabinet file which contains the content and is later encoded with Base64 as well. */
    #[yaserde(rename = "avatarcontentid", attribute)]
    pub avatarcontentid: Option<String>,

    /* Not serialized  */
    pub computeSha1c: bool,

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
    type Err = Errors;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(deserialized) = from_str::<MSNObject>(s) {
            return Ok(deserialized);
          } else {
              return Err(Errors::PayloadDeserializeError);
          }    }
}

impl yaserde::YaSerialize for &MSNObject {
    fn serialize<W: std::io::Write>(&self, writer: &mut yaserde::ser::Serializer<W>) -> Result<(), String> {
        let size = self.size.to_string();
        let obj_type = self.obj_type.to_string();
        let friendly = self.get_friendly_base64_or_default();
        let mut elem = xml::writer::XmlEvent::start_element("msnobj").attr("Creator", &self.creator).attr("Type", &obj_type).attr("SHA1D", &self.sha1d).attr("Size", &size).attr("Location", &self.location).attr("Friendly", &friendly);
        let sha1c = self.get_sha1c();
        if self.computeSha1c {
            elem = elem.attr("SHA1C", &sha1c);
        }

        let content_type = self.contenttype.as_ref().unwrap().to_string();

        if self.contenttype.is_some() {
            elem = elem.attr("contenttype", &content_type);
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
    pub fn new(creator: String, obj_type: MSNObjectType, location: String, sha1d: String, size: usize, friendly: Option<String>, contenttype: Option<MSNObjectContentType>, computeSha1c: bool) -> Self {
        return Self{ creator, size: size.try_into().unwrap(), obj_type, location, friendly, sha1d, contenttype, contentid: None, partnerid: None, stamp: None, avatarid: None, avatarcontentid: None, computeSha1c };
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

    fn get_sha1c(&self) -> String {
        let sha1Input = format!("Creator{creator}Type{obj_type}SHA1D{sha1d}Size{size}Location{location}Friendly{friendly}", creator = &self.creator, size = &self.size, obj_type = self.obj_type.clone() as i32, location = &self.location, friendly = self.get_friendly_base64_or_default(), sha1d = &self.sha1d);
        return MSNObjectFactory::compute_sha1(sha1Input.as_bytes());
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

#[derive(Clone, Debug, EnumString, YaSerialize, YaDeserialize)]
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


#[derive(Clone, Debug, YaSerialize, Default)]
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
    WebcamDynamicDisplayPicture=17,
    #[default]
    Default=-1
}

impl yaserde::YaDeserialize for MSNObjectType {

    fn deserialize<R: std::io::Read>(reader: &mut de::Deserializer<R>) -> Result<Self, String> {
        reader.peek();
        
        if let xml::reader::XmlEvent::Characters(obj_type) = reader.inner_next()?.to_owned() {
            let text_parsed : i32 = FromStr::from_str(obj_type.as_str()).unwrap();
            return Ok(MSNObjectType::try_from(text_parsed).unwrap());
        }

        return Err("Characters missing".to_string());

    }


}

impl Display for MSNObjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.clone() as i32);   
    }
}

impl TryFrom<i32> for MSNObjectType {
    type Error = Errors;

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
                Err(Errors::PayloadDeserializeError)
            }
        }
    }
}

pub struct MSNObjectFactory;

impl MSNObjectFactory {

    pub fn get_display_picture(image: &[u8], creator_msn_addr: String, location: String, friendly: Option<String>) -> MSNObject {
        let sha1d = Self::compute_sha1(&image);

        return MSNObject::new(creator_msn_addr, MSNObjectType::DisplayPicture, location, sha1d, image.len(), friendly, Some(MSNObjectContentType::D), false);
    }

    pub fn get_contact_display_picture(image: &[u8], creator_msn_addr: String, location: String, friendly: Option<String>) -> MSNObject {
        let sha1d = Self::compute_sha1(&image);

        return MSNObject::new(creator_msn_addr, MSNObjectType::DisplayPicture, location, sha1d, image.len(), friendly, Some(MSNObjectContentType::D), false);
    }

    pub(self) fn compute_sha1(data: &[u8]) -> String {
        let mut hasher = Sha1::new();
        hasher.update(data);
        let result = hasher.finalize();
        return general_purpose::STANDARD.encode(&result);
    }


}

mod tests {
    use crate::models::{msn_object::MSNObject, p2p::slp_context::SlpContext};

    use super::MSNObjectFactory;

    lazy_static_include_bytes! {
        AVATAR_BYTES => "assets/img/avatar.jpg"
    }

    /**
     * <msnobj Creator="aeoncl1@shlasouf.local" Type="3" SHA1D="1u3ees4vbHswj+6ud7vJ1g24Lz0=" Size="25810" Location="0" Friendly="RgBsAGEAcgBlAAAA" contenttype="D"/>     * 
     */

    #[test]
    fn compute_sha1d() {
        let sha1d = MSNObjectFactory::compute_sha1(&AVATAR_BYTES);
        assert_eq!("1u3ees4vbHswj+6ud7vJ1g24Lz0=", &sha1d);
    }


    #[test]
    fn get_display_picture() {

        let obj = MSNObjectFactory::get_display_picture(&AVATAR_BYTES, String::from("aeoncl1@shlasouf.local"), String::from("0"), Some(String::from("Flare")));
        let friendly_b64 = obj.get_friendly_base64_or_default();
        
        let obj_serialized = obj.to_string();

        let str = "%3Cmsnobj%20Creator%3D%22aeoncl1%40shlasouf.local%22%20Type%3D%223%22%20SHA1D%3D%221u3ees4vbHswj%2B6ud7vJ1g24Lz0%3D%22%20Size%3D%2225810%22%20Location%3D%220%22%20Friendly%3D%22RgBsAGEAcgBlAAAA%22%20contenttype%3D%22D%22%2F%3E";
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
        println!("{:?}", &msn_obj);
    }

}