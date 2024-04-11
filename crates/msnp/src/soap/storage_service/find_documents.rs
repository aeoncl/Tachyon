pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::storage_service::headers::StorageServiceHeaders;
    use crate::soap::storage_service::msnstorage_datatypes::Handle;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapFindDocumentsMessage {
        #[yaserde(rename = "FindDocuments", default)]
        pub body: FindDocumentsRequestType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "FindDocuments",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
    prefix = "nsi1"
    )]
    pub struct FindDocumentsRequestType {
        #[yaserde(rename = "objectHandle", default)]
        pub object_handle: Handle,
        #[yaserde(rename = "documentAttributes", default)]
        pub document_attributes: DocumentAttributes,
        #[yaserde(rename = "documentFilter", default)]
        pub document_filter: DocumentFilter,
        #[yaserde(rename = "documentSort", default)]
        pub document_sort: DocumentSort,
        #[yaserde(rename = "findContext", default)]
        pub find_context: FindContext,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "documentAttributes",
    )]
    pub struct DocumentAttributes {
        #[yaserde(rename = "ResourceID", default)]
        pub resource_id: bool,
        #[yaserde(rename = "Name", default)]
        pub name: bool,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "documentFilter",
    )]
    pub struct DocumentFilter {
        #[yaserde(rename = "FilterAttributes", default)]
        pub filter_attributes: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "documentSort",
    )]
    pub struct DocumentSort {
        #[yaserde(rename = "SortBy", default)]
        pub sort_by: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "findContext",
    )]
    pub struct FindContext {
        #[yaserde(rename = "FindMethod", default)]
        pub find_method: String,
        #[yaserde(rename = "ChunkSize", default)]
        pub chunk_size: i32,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct FindDocumentsMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: StorageServiceHeaders,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapFindDocumentsMessage,
    }

    impl FindDocumentsMessageSoapEnvelope {
        pub fn new(body: SoapFindDocumentsMessage) -> Self {
            FindDocumentsMessageSoapEnvelope {
                body,
                header: StorageServiceHeaders::default(),
            }
        }
    }

}

pub mod response {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::storage_service::fault::SoapFault;
    use crate::soap::storage_service::headers::StorageServiceHeaders;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapFindDocumentsResponseMessage {
        #[yaserde(rename = "FindDocumentsResponse", default)]
        pub body: FindDocumentsResultType,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "FindDocumentsResponse",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
    prefix = "nsi1"
    )]
    pub struct FindDocumentsResultType {
        #[yaserde(rename = "Document", prefix = "nsi1")]
        pub document: Document,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "Document",
    )]
    pub struct Document {
        #[yaserde(rename = "ResourceID", default)]
        pub resource_id: String,
        #[yaserde(rename = "Name", default)]
        pub name: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct FindDocumentsResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: StorageServiceHeaders,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapFindDocumentsResponseMessage,
    }

    impl FindDocumentsResponseMessageSoapEnvelope {
        pub fn new(body: SoapFindDocumentsResponseMessage) -> Self {
            FindDocumentsResponseMessageSoapEnvelope {
                body,
                header: StorageServiceHeaders::default(),
            }
        }
    }


}