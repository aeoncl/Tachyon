pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};

    use crate::soap::abch::msnab_datatypes::{AbHandleType, ArrayOfAnnotation};
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::sharing_service::find_membership_by_role::request::FindMembershipByRoleMessageSoapEnvelope;
    use crate::soap::error::SoapMarshallError;
    use crate::soap::traits::xml::TryFromXml;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapManageWLConnectionMessage {
        #[yaserde(rename = "ManageWLConnection", default)]
        pub body: ManageWLConnectionRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ManageWLConnectionRequestType")]
    pub struct ManageWLConnectionRequestType {
        #[yaserde(rename = "abHandle", default)]
        pub ab_handle: AbHandleType,
        #[yaserde(rename = "contactId", default)]
        pub contact_id: String,
        #[yaserde(rename = "connection", default)]
        pub connection: bool,
        #[yaserde(rename = "presence", default)]
        pub presence: bool,
        #[yaserde(rename = "action", default)]
        pub action: i32,
        #[yaserde(rename = "relationshipType", default)]
        pub relationship_type: i32,
        #[yaserde(rename = "relationshipRole", default)]
        pub relationship_role: i32,
        #[yaserde(rename = "annotations", default)]
        pub annotations: Option<ArrayOfAnnotation>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct ManageWLConnectionMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapManageWLConnectionMessage,
    }

    impl ManageWLConnectionMessageSoapEnvelope {
        pub fn new(body: SoapManageWLConnectionMessage) -> Self {
            ManageWLConnectionMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

    impl TryFromXml for ManageWLConnectionMessageSoapEnvelope {
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
    use crate::soap::error::SoapMarshallError;
    use crate::soap::traits::xml::ToXml;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapManageWLConnectionResponseMessage {
        #[yaserde(rename = "ManageWLConnectionResponseMessage", default)]
        pub body: ManageWLConnectionResponse,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "ManageWLConnectionResponse",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    prefix = "nsi1"
    )]
    pub struct ManageWLConnectionResponse {
        #[yaserde(rename = "ManageWLConnectionResult", default)]
        pub manage_wl_connection_result: ContactType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct ManageWLConnectionResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapManageWLConnectionResponseMessage,
    }

    impl ManageWLConnectionResponseMessageSoapEnvelope {
        pub fn new(body: SoapManageWLConnectionResponseMessage) -> Self {
            ManageWLConnectionResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

    impl ToXml for ManageWLConnectionResponseMessageSoapEnvelope {
        type Error = SoapMarshallError;
        fn to_xml(&self) -> Result<String, Self::Error>  {
            to_string(self).map_err(|e| SoapMarshallError::SerializationError { message: e})
        }

    }



}