pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::storage_service::headers::StorageServiceHeaders;
    use crate::soap::storage_service::msnstorage_datatypes::{DocumentBaseType, Handle};

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapCreateDocumentMessage {
        #[yaserde(rename = "CreateDocument", default)]
        pub body: CreateDocumentRequestType
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "CreateDocument",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
    prefix = "nsi1"
    )]
    pub struct CreateDocumentRequestType {
        #[yaserde(rename = "parentHandle", prefix = "nsi1")]
        pub parent_handle: Handle,
        #[yaserde(rename = "document", prefix = "nsi1")]
        pub document: DocumentBaseType,
        #[yaserde(rename = "relationshipName", prefix = "nsi1")]
        pub relationship_name: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct CreateDocumentMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: Option<StorageServiceHeaders>,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapCreateDocumentMessage,
    }

    impl CreateDocumentMessageSoapEnvelope {
        pub fn new(body: SoapCreateDocumentMessage) -> Self {
            CreateDocumentMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

}

pub mod response {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::storage_service::fault::SoapFault;
    use crate::soap::storage_service::headers::StorageServiceHeaders;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapCreateDocumentResponseMessage {
        #[yaserde(rename = "CreateDocumentResponse", default)]
        pub body: CreateDocumentResponseType,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "CreateDocumentResponse",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
    prefix = "nsi1"
    )]
    pub struct CreateDocumentResponseType {
        #[yaserde(rename = "CreateDocumentResult", prefix = "nsi1")]
        pub create_document_result: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct CreateDocumentResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: StorageServiceHeaders,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapCreateDocumentResponseMessage,
    }

    impl CreateDocumentResponseMessageSoapEnvelope {
        pub fn new(body: SoapCreateDocumentResponseMessage) -> Self {
            CreateDocumentResponseMessageSoapEnvelope {
                body,
                header: StorageServiceHeaders::default(),
            }
        }
    }

}