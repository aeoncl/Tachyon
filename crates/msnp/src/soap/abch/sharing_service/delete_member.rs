pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};

    use crate::soap::abch::msnab_datatypes::{ContentHandleType, HandleType};
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::sharing_service::delete_contact::request::DeleteContactMessageSoapEnvelope;
    use crate::soap::abch::sharing_service::find_membership::response::Memberships;
    use crate::soap::error::SoapMarshallError;
    use crate::soap::traits::xml::TryFromXml;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapDeleteMemberMessage {
        #[yaserde(rename = "DeleteMember", default)]
        pub body: DeleteMemberRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "DeleteMemberRequestType",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1")]
    pub struct DeleteMemberRequestType {
        #[yaserde(rename = "serviceHandle", default)]
        pub service_handle: HandleType,
        #[yaserde(rename = "memberships", default)]
        pub memberships: Memberships,
        #[yaserde(rename = "nsHandle", default)]
        pub ns_handle: Option<ContentHandleType>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct DeleteMemberMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapDeleteMemberMessage,
    }

    impl DeleteMemberMessageSoapEnvelope {
        pub fn new(body: SoapDeleteMemberMessage) -> Self {
            DeleteMemberMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

    impl TryFromXml for DeleteMemberMessageSoapEnvelope {
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
    use crate::soap::abch::service_header::ServiceHeaderContainer;
    use crate::soap::abch::sharing_service::delete_contact::response::DeleteContactResponseMessageSoapEnvelope;
    use crate::soap::error::SoapMarshallError;
    use crate::soap::traits::xml::ToXml;

    #[cfg(test)]
    mod tests {
        use yaserde::ser::to_string;

        use crate::soap::abch::sharing_service::delete_member::response::DeleteMemberResponseMessageSoapEnvelope;

        #[test]
        fn test_delete_member_response() {
            let response = DeleteMemberResponseMessageSoapEnvelope::new("cachekey");

            let response_serialized = to_string(&response).unwrap();
            println!("{}", response_serialized);
        }

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapDeleteMemberResponseMessage {
        #[yaserde(rename = "DeleteMemberResponse", default)]
        pub body: DeleteMemberResponse,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "DeleteMemberResponse"
    )]
    pub struct DeleteMemberResponse {}

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct DeleteMemberResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapDeleteMemberResponseMessage,
    }

    impl DeleteMemberResponseMessageSoapEnvelope {
        pub fn new(cache_key: &str) -> DeleteMemberResponseMessageSoapEnvelope {

            let body = SoapDeleteMemberResponseMessage{
                body: DeleteMemberResponse {},
                fault: None,
            };
            Self {body, header: Some(ServiceHeaderContainer::new(cache_key))}
        }
    }

    impl ToXml for DeleteMemberResponseMessageSoapEnvelope {
        type Error = SoapMarshallError;
        fn to_xml(&self) -> Result<String, Self::Error>  {
            to_string(self).map_err(|e| SoapMarshallError::SerializationError { message: e})
        }

    }

}