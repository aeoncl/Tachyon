pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::rsi::service_header::ServiceHeader;

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