pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};

    use crate::soap::abch::msnab_datatypes::AbHandleType;
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::sharing_service::batch_recent_activity::request::GetBatchRecentActivityMessageSoapEnvelope;
    use crate::soap::error::SoapMarshallError;
    use crate::soap::traits::xml::TryFromXml;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapBreakConnectionMessage {
        #[yaserde(rename = "BreakConnection", default)]
        pub body: BreakConnectionRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "BreakConnectionRequestType")]
    pub struct BreakConnectionRequestType {
        #[yaserde(rename = "abHandle", default)]
        pub ab_handle: AbHandleType,
        #[yaserde(rename = "contactId", default)]
        pub contact_id: String,
        #[yaserde(rename = "deleteContact", default)]
        pub delete_contact: bool,
        #[yaserde(rename = "blockContact", default)]
        pub block_contact: bool,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct BreakConnectionMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapBreakConnectionMessage,
    }

    impl BreakConnectionMessageSoapEnvelope {
        pub fn new(body: SoapBreakConnectionMessage) -> Self {
            BreakConnectionMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

    impl TryFromXml for BreakConnectionMessageSoapEnvelope {
        type Error = SoapMarshallError;

        fn try_from_xml(xml_str: &str) -> Result<Self, Self::Error> {
            yaserde::de::from_str::<Self>(&xml_str).map_err(|e| Self::Error::DeserializationError { message: e})
        }
    }

}

pub mod response {
    use yaserde::ser::to_string;
    use yaserde_derive::{YaDeserialize, YaSerialize};

    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::service_header::ServiceHeaderContainer;
    use crate::soap::abch::sharing_service::batch_recent_activity::response::GetBatchRecentActivityResponseMessageSoapEnvelope;
    use crate::soap::error::SoapMarshallError;
    use crate::soap::traits::xml::ToXml;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapBreakConnectionResponseMessage {
        #[yaserde(rename = "BreakConnectionResponseMessage", default)]
        pub body: BreakConnectionResponseType,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "BreakConnectionResponseType")]
    pub struct BreakConnectionResponseType {}

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct BreakConnectionResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapBreakConnectionResponseMessage,
    }

    impl BreakConnectionResponseMessageSoapEnvelope {
        pub fn new(body: SoapBreakConnectionResponseMessage) -> Self {
            BreakConnectionResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

    impl ToXml for BreakConnectionResponseMessageSoapEnvelope {
        type Error = SoapMarshallError;
        fn to_xml(&self) -> Result<String, Self::Error>  {
            to_string(self).map_err(|e| SoapMarshallError::SerializationError { message: e})
        }

    }

}