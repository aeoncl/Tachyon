pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};

    use crate::soap::abch::msnab_datatypes::Guid;
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::sharing_service::create_contact::request::CreateContactMessageSoapEnvelope;
    use crate::soap::error::SoapMarshallError;
    use crate::soap::traits::xml::TryFromXml;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapDeleteContactMessage {
        #[yaserde(rename = "DeleteContact", default)]
        pub body: DeleteContactRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "DeleteContactRequestType")]
    pub struct DeleteContactRequestType {
        #[yaserde(rename = "contactId", default)]
        pub contact_id: Guid,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct DeleteContactMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapDeleteContactMessage,
    }

    impl DeleteContactMessageSoapEnvelope {
        pub fn new(body: SoapDeleteContactMessage) -> Self {
            DeleteContactMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

    impl TryFromXml for DeleteContactMessageSoapEnvelope {
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
    use crate::soap::abch::sharing_service::create_contact::response::CreateContactResponseMessageSoapEnvelope;
    use crate::soap::error::SoapMarshallError;
    use crate::soap::traits::xml::ToXml;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapDeleteContactResponseMessage {
        #[yaserde(rename = "DeleteContactResponse", default)]
        pub body: DeleteContactResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "DeleteContactResponse")]
    pub struct DeleteContactResponseMessage {
        #[yaserde(default)]
        pub delete_contact_response: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct DeleteContactResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapDeleteContactResponseMessage,
    }

    impl DeleteContactResponseMessageSoapEnvelope {
        pub fn new(body: SoapDeleteContactResponseMessage) -> Self {
            DeleteContactResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

    impl ToXml for DeleteContactResponseMessageSoapEnvelope {
        type Error = SoapMarshallError;
        fn to_xml(&self) -> Result<String, Self::Error>  {
            to_string(self).map_err(|e| SoapMarshallError::SerializationError { message: e})
        }

    }

}