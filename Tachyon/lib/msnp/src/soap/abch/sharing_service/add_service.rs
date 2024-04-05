pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::sharing_service::find_membership::response::InfoType;
    use crate::soap::abch::msnab_sharingservice::{SOAP_ENCODING};
    use crate::soap::abch::request_header::RequestHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAddServiceMessage {
        #[yaserde(rename = "AddService", default)]
        pub body: AddServiceRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "AddService")]
    pub struct AddServiceRequestType {
        #[yaserde(rename = "serviceInfo", default)]
        pub service_info: InfoType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct AddServiceMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAddServiceMessage,
    }

    impl AddServiceMessageSoapEnvelope {
        pub fn new(body: SoapAddServiceMessage) -> Self {
            AddServiceMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }



}

pub mod response {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::msnab_sharingservice::{SOAP_ENCODING};
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::service_header::ServiceHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAddServiceResponseMessage {
        #[yaserde(rename = "AddServiceResponseMessage", default)]
        pub body: AddServiceResponseType,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "AddServiceResponseType")]
    pub struct AddServiceResponseType {
        #[yaserde(rename = "AddServiceResult", default)]
        pub add_service_result: i64,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct AddServiceResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAddServiceResponseMessage,
    }

    impl AddServiceResponseMessageSoapEnvelope {
        pub fn new(body: SoapAddServiceResponseMessage) -> Self {
            AddServiceResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }


}