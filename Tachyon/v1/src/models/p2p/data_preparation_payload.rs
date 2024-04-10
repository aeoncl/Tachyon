use crate::models::msn_object::MSNObject;

#[derive(Clone, Debug, YaDeserialize, YaSerialize, Default)]
#[yaserde(rename = "map")]
pub struct Map {

    #[yaserde(rename = "h")]
    pub header: MapHeader,

    #[yaserde(rename = "m")]
    pub body: MapBody

}

#[derive(Clone, Debug, YaDeserialize, YaSerialize, Default)]
#[yaserde(rename = "h")]
pub struct MapHeader {

    #[yaserde(rename = "aid", attribute)]
    pub aid: MapAid,

    #[yaserde(rename = "op", attribute)]
    pub op: String,

    #[yaserde(rename = "ver", attribute)]
    pub ver: String
}

#[derive(Clone, Debug, YaDeserialize, YaSerialize, Default)]
#[yaserde(rename = "m")]
pub struct MapBody {
    #[yaserde(rename = "guid", attribute)]
    pub guid: String,

    #[yaserde(rename = "hash", attribute)]
    pub hash: String,

    #[yaserde(rename = "si", attribute)]
    pub si: u32,

    //Thumbnail URLEncoded MSN Object
    #[yaserde(rename = "tospath", attribute)]
    pub tospath: MSNObject,

    //Main URLEncoded MSN Object
    #[yaserde(rename = "mospath", attribute)]
    pub mospath: MSNObject,

    #[yaserde(rename = "dispn", attribute)]
    pub display_name: String
}


#[derive(Clone, Debug, YaDeserialize, YaSerialize, Default)]
pub enum MapAid {
    #[default]
    PS
}