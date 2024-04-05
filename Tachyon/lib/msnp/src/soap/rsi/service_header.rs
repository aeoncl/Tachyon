use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "Header",
namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
prefix = "soapenv"
)]
pub struct ServiceHeader {
    #[yaserde(rename = "PassportCookie", default)]
    pub passport_cookie: PassportCookie,
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "PassportCookie",
namespace = "nsi1: http://www.hotmail.msn.com/ws/2004/09/oim/rsi",
prefix = "nsi1",
)]
pub struct PassportCookie {
    #[yaserde(rename = "t", prefix="nsi1")]
    pub t: String,
    #[yaserde(rename = "p", prefix="nsi1")]
    pub p: String,
}