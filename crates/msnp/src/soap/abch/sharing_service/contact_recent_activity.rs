pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};

    use crate::soap::abch::msnab_datatypes::{EntityHandle, Locale};
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::sharing_service::break_connection::request::BreakConnectionMessageSoapEnvelope;
    use crate::soap::error::SoapMarshallError;
    use crate::soap::traits::xml::TryFromXml;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct GetContactsRecentActivityMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapGetContactsRecentActivityMessage,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetContactsRecentActivityMessage {
        #[yaserde(rename = "GetContactsRecentActivity", default)]
        pub body: GetContactsRecentActivityRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    impl GetContactsRecentActivityMessageSoapEnvelope {
        pub fn new(body: SoapGetContactsRecentActivityMessage) -> Self {
            GetContactsRecentActivityMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "GetContactsRecentActivityRequestType")]
    pub struct GetContactsRecentActivityRequestType {
        #[yaserde(rename = "entityHandle", default)]
        pub entity_handle: EntityHandle,
        #[yaserde(rename = "locales", default)]
        pub locales: Locale,
        #[yaserde(rename = "count", default)]
        pub count: i32,
    }

    impl TryFromXml for GetContactsRecentActivityMessageSoapEnvelope {
        type Error = SoapMarshallError;

        fn try_from_xml(xml_str: &str) -> Result<Self, Self::Error> {
            yaserde::de::from_str::<Self>(&xml_str).map_err(|e| Self::Error::DeserializationError { message: e})
        }
    }

}

pub mod response {
    use yaserde::ser::to_string;
    use yaserde_derive::{YaDeserialize, YaSerialize};

    use crate::soap::abch::msnab_datatypes::{Activities, CollapseConditionType, RecentActivityTemplateType};
    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::service_header::ServiceHeaderContainer;
    use crate::soap::abch::sharing_service::break_connection::response::BreakConnectionResponseMessageSoapEnvelope;
    use crate::soap::error::SoapMarshallError;
    use crate::soap::traits::xml::ToXml;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetContactsRecentActivityResponseMessage {
        #[yaserde(rename = "GetContactsRecentActivityResponse", default)]
        pub body: GetContactsRecentActivityResponse,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct GetContactsRecentActivityResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapGetContactsRecentActivityResponseMessage,
    }

    impl GetContactsRecentActivityResponseMessageSoapEnvelope {
        pub fn new(body: SoapGetContactsRecentActivityResponseMessage) -> Self {
            GetContactsRecentActivityResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "GetContactsRecentActivityResponse",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    prefix = "nsi1"
    )]
    pub struct GetContactsRecentActivityResponse {
        #[yaserde(rename = "GetContactsRecentActivityResult", default)]
        pub get_contacts_recent_activity_result: GetContactsRecentActivityResultType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "GetContactsRecentActivityResultType")]
    pub struct GetContactsRecentActivityResultType {
        #[yaserde(rename = "Activities", default)]
        pub activities: Activities,
        #[yaserde(rename = "Templates", default)]
        pub templates: Templates,
        #[yaserde(rename = "FeedUrl", default)]
        pub feed_url: String,
    }



    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "Templates")]
    pub struct Templates {
        #[yaserde(rename = "RecentActivityTemplateContainer", default)]
        pub recent_activity_template_container: Vec<RecentActivityTemplateContainerType>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "RecentActivityTemplateContainerType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct RecentActivityTemplateContainerType {
        #[yaserde(rename = "ApplicationId", prefix = "nsi1")]
        pub application_id: String,
        #[yaserde(rename = "ApplicationName", prefix = "nsi1")]
        pub application_name: String,
        #[yaserde(rename = "ChangeType", prefix = "nsi1")]
        pub change_type: i32,
        #[yaserde(rename = "Locale", prefix = "nsi1")]
        pub locale: String,
        #[yaserde(rename = "RequestedLocales", prefix = "nsi1")]
        pub requested_locales: RequestedLocalesType,
        #[yaserde(rename = "TemplateRevision", prefix = "nsi1")]
        pub template_revision: i32,
        #[yaserde(rename = "Templates", prefix = "nsi1")]
        pub templates: Templates2,
        #[yaserde(rename = "CollapseCondition", prefix = "nsi1")]
        pub collapse_condition: Option<CollapseConditionType>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "Templates", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct Templates2 {
        #[yaserde(rename = "RecentActivityTemplate", prefix = "nsi1")]
        pub recent_activity_template: Vec<RecentActivityTemplateType>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "RequestedLocalesType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct RequestedLocalesType {
        #[yaserde(rename = "string", prefix = "nsi1")]
        pub string: Vec<String>,
    }

    impl ToXml for GetContactsRecentActivityResponseMessageSoapEnvelope {
        type Error = SoapMarshallError;
        fn to_xml(&self) -> Result<String, Self::Error>  {
            to_string(self).map_err(|e| SoapMarshallError::SerializationError { message: e})
        }

    }

}