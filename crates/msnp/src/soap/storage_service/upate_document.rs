pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::error::SoapMarshallError;
    use crate::soap::storage_service::headers::StorageServiceHeaders;
    use crate::soap::storage_service::msnstorage_datatypes::DocumentBaseType;
    use crate::soap::traits::xml::TryFromXml;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapUpdateDocumentMessage {
        #[yaserde(rename = "UpdateDocument", default)]
        pub body: UpdateDocumentRequestType
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "UpdateDocument",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
    prefix = "nsi1"
    )]
    pub struct UpdateDocumentRequestType {
        #[yaserde(rename = "document", prefix = "nsi1")]
        pub document: DocumentBaseType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct UpdateDocumentMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: StorageServiceHeaders,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapUpdateDocumentMessage,
    }

    impl UpdateDocumentMessageSoapEnvelope {
        pub fn new(body: SoapUpdateDocumentMessage) -> Self {
            UpdateDocumentMessageSoapEnvelope {
                body,
                header: StorageServiceHeaders::default(),
            }
        }
    }

    impl TryFromXml for UpdateDocumentMessageSoapEnvelope {
        type Error = SoapMarshallError;

        fn try_from_xml(xml_str: &str) -> Result<Self, Self::Error> {
            yaserde::de::from_str::<Self>(&xml_str).map_err(|e| Self::Error::DeserializationError { message: e})
        }
    }

}

pub mod response {
    use yaserde::ser::to_string;
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::error::SoapMarshallError;
    use crate::soap::storage_service::fault::SoapFault;
    use crate::soap::storage_service::headers::{AffinityCacheHeader, StorageServiceHeaders};
    use crate::soap::storage_service::update_profile::response::{SoapUpdateProfileResponseMessage, UpdateProfileResponseMessageSoapEnvelope};
    use crate::soap::traits::xml::ToXml;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapUpdateDocumentResponseMessage {
        #[yaserde(rename = "UpdateDocumentResponse", default)]
        pub body: UpdateDocumentResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }
    
    //TODO Check WSDL
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "UpdateDocumentResponse",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
    prefix = "nsi1"
    )]
    pub struct UpdateDocumentResponseMessage {
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct UpdateDocumentResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: StorageServiceHeaders,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapUpdateDocumentResponseMessage,
    }

    impl UpdateDocumentResponseMessageSoapEnvelope {
        pub fn new(cache_key: String) -> Self {
            let affinity_cache_header = AffinityCacheHeader{ cache_key: Some(cache_key)};

            let headers = StorageServiceHeaders{ storage_application: None, storage_user: None, affinity_cache: Some(affinity_cache_header) };

            UpdateDocumentResponseMessageSoapEnvelope {
                body: SoapUpdateDocumentResponseMessage::default(),
                header: headers,
            }
        }
    }

    impl ToXml for UpdateDocumentResponseMessageSoapEnvelope {
        type Error = SoapMarshallError;
        fn to_xml(&self) -> Result<String, Self::Error>  {
            to_string(self).map_err(|e| SoapMarshallError::SerializationError { message: e})
        }

    }



}