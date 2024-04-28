pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};

    use crate::soap::error::SoapMarshallError;
    use crate::soap::passport::rst2::request::RST2RequestMessageSoapEnvelope;
    use crate::soap::rsi::service_header::ServiceHeader;
    use crate::soap::traits::xml::TryFromXml;

    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;

        use crate::soap::rsi::get_message::request::GetMessageMessageSoapEnvelope;

        #[test]
        fn test_deser() {
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

            let deser : GetMessageMessageSoapEnvelope = from_str(req).unwrap();
        }


    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetMessageMessage {
        #[yaserde(rename = "GetMessage", default)]
        pub body: GetMessageRequestType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "GetMessage",
    namespace = "nsi1: http://www.hotmail.msn.com/ws/2004/09/oim/rsi",
    prefix = "nsi1"
    )]
    pub struct GetMessageRequestType {
        #[yaserde(rename = "messageId", prefix="nsi1")]
        pub message_id: String,
        #[yaserde(rename = "alsoMarkAsRead", prefix="nsi1")]
        pub also_mark_as_read: bool,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct GetMessageMessageSoapEnvelope {
        pub header: Option<ServiceHeader>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetMessageMessage,
    }

    impl TryFromXml for GetMessageMessageSoapEnvelope {

        type Error = SoapMarshallError;

        fn try_from_xml(xml_str: &str) -> Result<Self, Self::Error> {
            yaserde::de::from_str::<Self>(&xml_str).map_err(|e| Self::Error::DeserializationError { message: e})
        }
    }

    impl GetMessageMessageSoapEnvelope {
        pub fn new(body: SoapGetMessageMessage) -> Self {
            GetMessageMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }


}

pub mod response {
    use yaserde::ser::to_string;
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::error::SoapMarshallError;
    use crate::soap::rsi::get_metadata::response::GetMetadataResponseMessageSoapEnvelope;

    use crate::soap::rsi::service_header::ServiceHeader;
    use crate::soap::traits::xml::ToXml;

    #[cfg(test)]
    mod tests {
        #[test]
        fn ser_test() {



        }

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetMessageResponseMessage {
        #[yaserde(rename = "GetMessageResponse", default)]
        pub body: GetMessageResponse
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "GetMessageResponse",
    )]
    pub struct GetMessageResponse {
        #[yaserde(rename = "GetMessageResult", default)]
        pub get_message_result: String,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct GetMessageResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<ServiceHeader>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetMessageResponseMessage,
    }

    impl ToXml for GetMessageResponseMessageSoapEnvelope {
        type Error = SoapMarshallError;

        fn to_xml(&self) -> Result<String, Self::Error>  {
            to_string(self).map_err(|e| SoapMarshallError::SerializationError { message: e})
        }
    }

    impl GetMessageResponseMessageSoapEnvelope {
        pub fn new(body: SoapGetMessageResponseMessage) -> Self {
            GetMessageResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }



}