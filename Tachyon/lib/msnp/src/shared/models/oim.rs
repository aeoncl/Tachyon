use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "MetadataMessage",
)]
pub struct MetadataMessage {
    #[yaserde(rename = "T", default)]
    pub t: i32,
    #[yaserde(rename = "S", default)]
    pub s: i32,
    #[yaserde(rename = "RT", default)]
    pub rt: Option<String>,
    #[yaserde(rename = "RS", default)]
    pub rs: bool,
    #[yaserde(rename = "SZ", default)]
    pub sz: i32,
    #[yaserde(rename = "E", default)]
    pub e: String,
    #[yaserde(rename = "I", default)]
    pub i: String,
    #[yaserde(rename = "F", default)]
    pub f: String,
    #[yaserde(rename = "N", default)]
    pub n: String,
    #[yaserde(rename = "SU", default)]
    pub su: String,
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "Q",
)]
pub struct Q {
    #[yaserde(rename = "QTM", default)]
    pub qtm: i32,
    #[yaserde(rename = "QNM", default)]
    pub qnm: i32,
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "MetaData",
)]
pub struct MetaData {
    #[yaserde(rename = "M", default)]
    pub m: Vec<MetadataMessage>,
    #[yaserde(rename = "Q", default)]
    pub q: Q,
}