use yaserde_derive::{YaDeserialize, YaSerialize};
use crate::soap::error::SoapMarshallError;
use crate::soap::rsi::get_message::request::{GetMessageMessageSoapEnvelope, SoapGetMessageMessage};
use crate::soap::traits::xml::TryFromXml;

#[cfg(test)]
mod tests {
    use yaserde::de::from_str;
    use crate::soap::rsi::service_header::RSIAuthSoapEnvelope;
    use crate::soap::traits::xml::TryFromXml;

    #[test]
    pub fn deser() {
        let req = r#"<?xml version="1.0" encoding="utf-8"?>
                            <soap:Envelope xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                                xmlns:xsd="http://www.w3.org/2001/XMLSchema"
                                xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/">
                                <soap:Header>
                                    <PassportCookie xmlns="http://www.hotmail.msn.com/ws/2004/09/oim/rsi">
                                        <t>token</t>
                                        <p></p>
                                    </PassportCookie>
                                </soap:Header>
                                <soap:Body>
                                    <GetMessage xmlns="http://www.hotmail.msn.com/ws/2004/09/oim/rsi">
                                        <messageId>Op4qu3</messageId>
                                        <alsoMarkAsRead>false</alsoMarkAsRead>
                                    </GetMessage>
                                </soap:Body>
                            </soap:Envelope>"#;


        let req : RSIAuthSoapEnvelope = from_str(&req).unwrap();
        assert_eq!(req.header.unwrap().passport_cookie.t, "token".to_string());


    }

}

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
default_namespace = "nsi1",
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
    #[yaserde(rename = "Header", prefix = "soapenv")]
    pub header: Option<ServiceHeader>,
}

impl TryFromXml for RSIAuthSoapEnvelope {

    type Error = SoapMarshallError;

    fn try_from_xml(xml_str: &str) -> Result<Self, Self::Error> {
        yaserde::de::from_str::<Self>(&xml_str).map_err(|e| Self::Error::DeserializationError { message: e})
    }
}