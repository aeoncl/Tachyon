pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};

    use crate::soap::abch::msnab_datatypes::{AbHandleType, ContactHandleType, ContactInfoType};
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::sharing_service::create_circle::request::CreateCircleMessageSoapEnvelope;
    use crate::soap::error::SoapMarshallError;
    use crate::soap::traits::xml::TryFromXml;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapCreateContactMessage {
        #[yaserde(rename = "CreateContact", default)]
        pub body: CreateContactType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "CreateContactType")]
    pub struct CreateContactType {
        #[yaserde(rename = "abHandle", default)]
        pub ab_handle: AbHandleType,
        #[yaserde(rename = "contactHandle", default)]
        pub contact_handle: ContactHandleType,
        #[yaserde(rename = "contactInfo", default)]
        pub contact_info: Option<ContactInfoType>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct CreateContactMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapCreateContactMessage,
    }

    impl CreateContactMessageSoapEnvelope {
        pub fn new(body: SoapCreateContactMessage) -> Self {
            CreateContactMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

    impl TryFromXml for CreateContactMessageSoapEnvelope {
        type Error = SoapMarshallError;

        fn try_from_xml(xml_str: &str) -> Result<Self, Self::Error> {
            yaserde::de::from_str::<Self>(&xml_str).map_err(|e| Self::Error::DeserializationError { message: e})
        }
    }

}

pub mod response {
    use yaserde::ser::to_string;
    use yaserde_derive::{YaDeserialize, YaSerialize};

    use crate::soap::abch::msnab_datatypes::ContactType;
    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::service_header::ServiceHeaderContainer;
    use crate::soap::abch::sharing_service::create_circle::response::CreateCircleResponseMessageSoapEnvelope;
    use crate::soap::error::SoapMarshallError;
    use crate::soap::traits::xml::ToXml;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapCreateContactResponseMessage {
        #[yaserde(rename = "CreateContactResponseMessage", default)]
        pub body: CreateContactResponse,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "CreateContactResponse",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    prefix = "nsi1"
    )]
    pub struct CreateContactResponse {
        #[yaserde(rename = "CreateContactResult", default)]
        pub create_contact_result: ContactType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct CreateContactResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapCreateContactResponseMessage,
    }

    impl CreateContactResponseMessageSoapEnvelope {
        pub fn new(body: SoapCreateContactResponseMessage) -> Self {
            CreateContactResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

    impl ToXml for CreateContactResponseMessageSoapEnvelope {
        type Error = SoapMarshallError;
        fn to_xml(&self) -> Result<String, Self::Error>  {
            to_string(self).map_err(|e| SoapMarshallError::SerializationError { message: e})
        }

    }





}