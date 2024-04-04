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
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::shared::models::oim::MetaData;
    use crate::soap::rsi::service_header::ServiceHeader;


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

    impl GetMetadataResponseMessageSoapEnvelope {
        pub fn new(body: SoapGetMetadataResponseMessage) -> Self {
            GetMetadataResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

}