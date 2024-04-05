pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_datatypes::{CallerInfoType, ContentInfoType};
    use crate::soap::abch::msnab_sharingservice::{SOAP_ENCODING};
    use crate::soap::abch::request_header::RequestHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapCreateCircleMessage {
        #[yaserde(rename = "CreateCircle", default)]
        pub body: CreateCircleRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "CreateCircleRequestType")]
    pub struct CreateCircleRequestType {
        #[yaserde(rename = "properties", default)]
        pub properties: ContentInfoType,
        #[yaserde(rename = "callerInfo", default)]
        pub caller_info: CallerInfoType,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct CreateCircleMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapCreateCircleMessage,
    }

    impl CreateCircleMessageSoapEnvelope {
        pub fn new(body: SoapCreateCircleMessage) -> Self {
            CreateCircleMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

}

pub mod response {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_datatypes::Guid;
    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::msnab_sharingservice::{SOAP_ENCODING};
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::service_header::ServiceHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapCreateCircleResponseMessage {
        #[yaserde(rename = "CreateCircleResponseMessage", default)]
        pub body: CreateCircleResponse,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "CreateCircleResponse",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    prefix = "nsi1"
    )]
    pub struct CreateCircleResponse {
        #[yaserde(rename = "CreateCircleResult", default)]
        pub create_circle_result: CreateCircleResponseType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "CreateCircleResponseType")]
    pub struct CreateCircleResponseType {
        #[yaserde(rename = "Id", default)]
        pub id: Guid,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct CreateCircleResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapCreateCircleResponseMessage,
    }

    impl CreateCircleResponseMessageSoapEnvelope {
        pub fn new(body: SoapCreateCircleResponseMessage) -> Self {
            CreateCircleResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }


}