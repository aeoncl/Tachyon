use yaserde_derive::{YaDeserialize, YaSerialize};
use crate::soap::error::SoapMarshallError;
use crate::soap::rsi::get_message::request::{GetMessageMessageSoapEnvelope, SoapGetMessageMessage};
use crate::soap::traits::xml::TryFromXml;

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

#[derive(Debug, Default, YaSerialize, YaDeserialize)]
#[yaserde(
rename = "Envelope",
namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
namespace = "xsd: http://www.w3.org/2001/XMLSchema",
prefix = "soapenv"
)]
pub struct RSIAuthSoapEnvelope {
    pub header: Option<ServiceHeader>,
}

impl TryFromXml for RSIAuthSoapEnvelope {

    type Error = SoapMarshallError;

    fn try_from_xml(xml_str: &str) -> Result<Self, Self::Error> {
        yaserde::de::from_str::<Self>(&xml_str).map_err(|e| Self::Error::DeserializationError { message: e})
    }
}