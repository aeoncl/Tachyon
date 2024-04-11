pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::storage_service::headers::StorageServiceHeaders;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapShareItemMessage {
        #[yaserde(rename = "ShareItem", default)]
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

}

pub mod response {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::storage_service::fault::SoapFault;
    use crate::soap::storage_service::headers::StorageServiceHeaders;

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


}