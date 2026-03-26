pub mod request {
    use crate::soap::error::SoapMarshallError;
    use crate::soap::storage_service::headers::StorageServiceHeaders;
    use crate::soap::storage_service::update_profile::request::UpdateProfileMessageSoapEnvelope;
    use crate::soap::traits::xml::TryFromXml;
    use yaserde_derive::{YaDeserialize, YaSerialize};

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapShareItemMessage {
        #[yaserde(rename = "ShareItem", prefix = "nsi1")]
        pub body: ShareItemRequestType
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "ShareItem",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
    prefix = "nsi1"
    )]
    pub struct ShareItemRequestType {
        #[yaserde(rename = "resourceID", prefix = "nsi1")]
        pub resource_id: String,
        #[yaserde(rename = "displayName", prefix = "nsi1")]
        pub display_name: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct ShareItemMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: StorageServiceHeaders,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapShareItemMessage,
    }

    impl ShareItemMessageSoapEnvelope {
        pub fn new(body: SoapShareItemMessage) -> Self {
            ShareItemMessageSoapEnvelope {
                body,
                header: StorageServiceHeaders::default(),
            }
        }
    }

    impl TryFromXml for ShareItemMessageSoapEnvelope {
        type Error = SoapMarshallError;

        fn try_from_xml(xml_str: &str) -> Result<Self, Self::Error> {
            yaserde::de::from_str::<Self>(&xml_str).map_err(|e| Self::Error::DeserializationError { message: e})
        }
    }

}

pub mod response {
    use crate::soap::error::SoapMarshallError;
    use crate::soap::storage_service::fault::SoapFault;
    use crate::soap::storage_service::headers::StorageServiceHeaders;
    use crate::soap::storage_service::upate_document::response::UpdateDocumentResponseMessageSoapEnvelope;
    use crate::soap::traits::xml::ToXml;
    use yaserde::ser::to_string;
    use yaserde_derive::{YaDeserialize, YaSerialize};

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapShareItemResponseMessage {
        #[yaserde(rename = "ShareItemResponse", default)]
        pub body: ShareItemResponseType,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "ShareItemResponse",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
    prefix = "nsi1"
    )]
    pub struct ShareItemResponseType {
        #[yaserde(rename = "ShareItemResult", prefix = "nsi1")]
        pub share_item_result: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct ShareItemResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: StorageServiceHeaders,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapShareItemResponseMessage,
    }

    impl ShareItemResponseMessageSoapEnvelope {
        pub fn new(body: SoapShareItemResponseMessage) -> Self {
            ShareItemResponseMessageSoapEnvelope {
                body,
                header: StorageServiceHeaders::default(),
            }
        }
    }

    impl ToXml for ShareItemResponseMessageSoapEnvelope {
        type Error = SoapMarshallError;
        fn to_xml(&self) -> Result<String, Self::Error>  {
            to_string(self).map_err(|e| SoapMarshallError::SerializationError { message: e})
        }

    }

}