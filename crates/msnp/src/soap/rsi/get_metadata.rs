pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::error::SoapMarshallError;
    use crate::soap::rsi::get_message::request::GetMessageMessageSoapEnvelope;
    use crate::soap::rsi::service_header::ServiceHeader;
    use crate::soap::traits::xml::TryFromXml;

    #[cfg(test)]
    mod tests {
        use crate::soap::rsi::get_metadata::request::GetMetadataMessageSoapEnvelope;
        use crate::soap::traits::xml::TryFromXml;

        #[test]
        fn deser_test() {
            let req = r#"<?xml version= "1.0" encoding= "utf-8"?>
                               <soap:Envelope xmlns:xsi= "http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd= "http://www.w3.org/2001/XMLSchema" xmlns:soap= "http://schemas.xmlsoap.org/soap/envelope/">
                                   <soap:Header>
                                       <PassportCookie xmlns= "http://www.hotmail.msn.com/ws/2004/09/oim/rsi">
                                           <t>ticket_token</t>
                                           <p></p>
                                       </PassportCookie>
                                   </soap:Header>
                                   <soap:Body>
                                       <GetMetadata xmlns= "http://www.hotmail.msn.com/ws/2004/09/oim/rsi" />
                                   </soap:Body>
                               </soap:Envelope>"#;

            let test = GetMetadataMessageSoapEnvelope::try_from_xml(req).unwrap();

        }


    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetMetadataMessage {
        #[yaserde(rename = "GetMetadata", default)]
        pub body: GetMetadataRequestType
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "GetMetadata",
    )]
    pub struct GetMetadataRequestType {}

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct GetMetadataMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<ServiceHeader>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetMetadataMessage,
    }

    impl TryFromXml for GetMetadataMessageSoapEnvelope {

        type Error = SoapMarshallError;

        fn try_from_xml(xml_str: &str) -> Result<Self, Self::Error> {
            yaserde::de::from_str::<Self>(&xml_str).map_err(|e| Self::Error::DeserializationError { message: e})
        }
    }

    impl GetMetadataMessageSoapEnvelope {
        pub fn new(body: SoapGetMetadataMessage) -> Self {
            GetMetadataMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }


}

pub mod response {
    use yaserde::ser::to_string;
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::shared::models::oim::MetaData;
    use crate::soap::error::SoapMarshallError;
    use crate::soap::passport::rst2::response::RST2ResponseMessageSoapEnvelope;
    use crate::soap::rsi::service_header::ServiceHeader;
    use crate::soap::traits::xml::ToXml;

    #[cfg(test)]
    mod tests {
        use crate::soap::rsi::get_metadata::response::GetMetadataResponseMessageSoapEnvelope;
        use crate::soap::traits::xml::ToXml;

        #[test]
        fn ser_test() {
           let resp = GetMetadataResponseMessageSoapEnvelope::new(Default::default());
           let ser = resp.to_xml().unwrap();
        }

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetMetadataResponseMessage {
        #[yaserde(rename = "GetMetadataResponse", default)]
        pub body: GetMetadataResponseType
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "GetMetadataResponse",
    )]
    pub struct GetMetadataResponseType {
        #[yaserde(rename = "MD", default)]
        pub md: MetaData,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct GetMetadataResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<ServiceHeader>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetMetadataResponseMessage,
    }

    impl ToXml for GetMetadataResponseMessageSoapEnvelope {
        type Error = SoapMarshallError;

        fn to_xml(&self) -> Result<String, Self::Error>  {
            to_string(self).map_err(|e| SoapMarshallError::SerializationError { message: e})
        }
    }

    impl GetMetadataResponseMessageSoapEnvelope {
        pub fn new(metadata: MetaData) -> Self {
            let response = GetMetadataResponseType {
                md: metadata,
            };
            
            let message = SoapGetMetadataResponseMessage {
                body: response,
            };

            GetMetadataResponseMessageSoapEnvelope {
                body: message,
                header: None,
            }
        }
    }

}