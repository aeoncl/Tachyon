use std::{fmt::Display, convert::Infallible};

use sha1::{Sha1, Digest};
use base64::{Engine, engine::general_purpose};
use strum_macros::EnumString;

use super::errors::Errors;

// Documentation source: https://wiki.nina.chat/wiki/Protocols/MSNP/MSNC/MSN_Object


#[derive(Clone, Debug)]

pub struct MSNObject {
    //MSN Address of the creator or sender
    creator: String,
    //Size in bytes
    size: usize,
    //Type
    obj_type: MSNObjectType,
    //The Location field contains the filename under which the file will be, or has been, stored. 0 For in the storage service.
    location: String,
    /*This field contains the name of the picture in Unicode (UTF-16 Little Endian) format. 
    The string is then encoded with Base64. For most types of descriptors this field is a null character, 
    or AAA= when encoded.*/ 
    friendly: Option<String>,
    //The SHA1D field contains a SHA1 hash of the images data encoded in Base64. It is also known as the Data Hash or the SHA1 Data Field. 
    sha1d: String,

    /* The following fields are not included in the sha1c */
    contenttype: Option<MSNObjectContentType>,

    // The contentid field contains the identifier of the item which the Content Partner gave it.
    contentid: Option<String>,

    /* The partnerid field contains the name of the Content Partner. 
    American Greetings/Blue Mountain for example, is identified by AG. 
    This field applies only for Custom Emoticons. 
    But if this field is not present in the Content XML file when it was downloaded, the value of this field is -1. */
    partnerid: Option<String>,

    /* The stamp field contains the Base64 encoded signature of the file. 
    It is a S/MIME signature of the Base64 encoded hash of the Content cabinet file and is signed by the MSN Content Authority. 
    The stamp field is used in the Content XML file to make sure that the file isn't modified. 
    Currently, it's only being included in the MSN Object for dynamic (Flash) contents like Winks and Dynamic Display Pictures.  
    */
    stamp: Option<String>,

    /* Down level fields */
    /* The avatarid field indicates what type of avatar is being referenced to. This field is currently only being seen with {6AD16E96-BC60-401B-89E2-5BB545DD2BF0}  */
    avatarid: Option<String>,

    /* The avatarcontentid field contains the SHA-1 hash of the cabinet file which contains the content and is later encoded with Base64 as well. */
    avatarcontentid: Option<String>,

    /* Not serialized  */
    computeSha1c: bool,

}

impl MSNObject {
    pub fn new(creator: String, obj_type: MSNObjectType, location: String, sha1d: String, size: usize, friendly: Option<String>, contenttype: Option<MSNObjectContentType>, computeSha1c: bool) -> Self {
        return Self{ creator, size, obj_type, location, friendly, sha1d, contenttype, contentid: None, partnerid: None, stamp: None, avatarid: None, avatarcontentid: None, computeSha1c };
    }

    fn get_friendly_base64_or_default(&self) -> String {
        if self.friendly.is_none() {
            return String::from("AAA=");
        }
        return general_purpose::STANDARD.encode(&self.friendly.unwrap().as_bytes());
    }

    fn get_sha1c(&self) -> String {
        let sha1Input = format!("Creator{creator}Size{size}Type{obj_type}Location{location}Friendly{friendly}SHA1D{sha1d}", creator = &self.creator, size = &self.size, obj_type = self.obj_type.clone() as i32, location = &self.location, friendly = self.get_friendly_base64_or_default(), sha1d = &self.sha1d);
        return MSNObjectFactory::compute_sha1(sha1Input.as_bytes());
    }
}

    /**
     * <msnobj Creator="aeoncl1@shlasouf.local" Type="3" SHA1D="1u3ees4vbHswj+6ud7vJ1g24Lz0=" Size="25810" Location="0" Friendly="RgBsAGEAcgBlAAAA" contenttype="D"/>     * 
     */

impl Display for MSNObject {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!("use yaserde");
        return write!(f, "{}", out);
    }
}

#[derive(Clone, Debug, EnumString)]
pub enum MSNObjectContentType {
    /* Paid, prevents other users to add it */
    P,
    /* Downloadable, Free */
    D
}


#[derive(Clone, Debug)]
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

    pub fn get_display_picture(image: &[u8], creator_msn_addr: String, location: String) -> MSNObject {
        let sha1d = Self::compute_sha1(&image);
        todo!();
    }

    pub(self) fn compute_sha1(data: &[u8]) -> String {
        let mut hasher = Sha1::new();
        hasher.update(data);
        let result = hasher.finalize();
        return general_purpose::STANDARD.encode(&result);
    }


}

mod tests {
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
        let str = "%3Cmsnobj%20Creator%3D%22aeoncl1%40shlasouf.local%22%20Type%3D%223%22%20SHA1D%3D%221u3ees4vbHswj%2B6ud7vJ1g24Lz0%3D%22%20Size%3D%2225810%22%20Location%3D%220%22%20Friendly%3D%22RgBsAGEAcgBlAAAA%22%20contenttype%3D%22D%22%2F%3E";
        todo!();
    }

}