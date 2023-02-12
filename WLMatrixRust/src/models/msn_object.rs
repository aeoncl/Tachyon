
#[derive(Clone, Debug)]

pub struct MSNObject {
    //MSN Address of the creator
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
    friendly: String,
    //The SHA1D field contains a SHA1 hash of the images data encoded in Base64. It is also known as the Data Hash or the SHA1 Data Field. 
    sha1d: String,
    /*
    This field contains all previous fields hashed with SHA1, and then encoded in Base64. 
    This field is better known as the Checksum, or SHA1 Checksum Field. 
    The string that must be hashed to get the resulting SHA1C value looks similar to: 
    */
    sha1c: Option<String>
}

impl MSNObject {


}

#[derive(Clone, Debug)]

pub enum MSNObjectType {
    //Unknown but presence since MSN 6.0
    Avatar = 1,
    CustomEmoticon = 2,
    DisplayPicture = 3,
    //Unknown but presence since MSN 6.0
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