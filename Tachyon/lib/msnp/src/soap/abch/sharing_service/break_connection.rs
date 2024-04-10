pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_datatypes::AbHandleType;
    use crate::soap::abch::request_header::RequestHeaderContainer;

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

}

pub mod response {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::service_header::ServiceHeaderContainer;

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

}