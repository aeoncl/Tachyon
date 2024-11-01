pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::storage_service::headers::StorageServiceHeaders;
    use crate::soap::storage_service::msnstorage_datatypes::Handle;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapDeleteRelationshipsMessage {
        #[yaserde(rename = "DeleteRelationships", default)]
        pub body: DeleteRelationshipsRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "DeleteRelationships",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
    prefix = "nsi1"
    )]
    pub struct DeleteRelationshipsRequestType {
        #[yaserde(rename = "sourceHandle", prefix= "nsi1")]
        pub source_handle: Handle,
        #[yaserde(rename = "targetHandles", prefix= "nsi1")]
        pub target_handles: TargetHandles,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "targetHandles",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
    prefix = "nsi1"
    )]
    pub struct TargetHandles {
        #[yaserde(rename = "ObjectHandle", prefix= "nsi1")]
        pub object_handle: Vec<Handle>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct DeleteRelationshipsMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: StorageServiceHeaders,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapDeleteRelationshipsMessage,
    }

    impl DeleteRelationshipsMessageSoapEnvelope {
        pub fn new(body: SoapDeleteRelationshipsMessage) -> Self {
            DeleteRelationshipsMessageSoapEnvelope {
                body,
                header: StorageServiceHeaders::default(),
            }
        }
    }

}

pub mod response {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::storage_service::headers::StorageServiceHeaders;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapDeleteRelationshipsResponseMessage {
        #[yaserde(rename = "DeleteRelationshipsResponse", default)]
        pub body: DeleteRelationshipsResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }


    //TODO check in wsdl
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "DeleteRelationshipsResponse",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
    prefix = "nsi1"
    )]
    pub struct DeleteRelationshipsResponseMessage {
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct DeleteRelationshipsResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: StorageServiceHeaders,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapDeleteRelationshipsResponseMessage,
    }

    impl DeleteRelationshipsResponseMessageSoapEnvelope {
        pub fn new(body: SoapDeleteRelationshipsResponseMessage) -> Self {
            DeleteRelationshipsResponseMessageSoapEnvelope {
                body,
                header: StorageServiceHeaders::default(),
            }
        }
    }

}