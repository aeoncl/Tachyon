pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_datatypes::AbHandleType;
    use crate::soap::abch::request_header::RequestHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapFindFriendsInCommonMessage {
        #[yaserde(rename = "FindFriendsInCommon", default)]
        pub body: FindFriendsInCommonRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "FindFriendsInCommonRequestType")]
    pub struct FindFriendsInCommonRequestType {
        #[yaserde(rename = "targetAB", default)]
        pub target_ab: Option<AbHandleType>,
        #[yaserde(rename = "domainID", default)]
        pub domain_id: i32,
        #[yaserde(rename = "view", default)]
        pub view: String,
        #[yaserde(rename = "maxResults", default)]
        pub max_results: i32,
        #[yaserde(rename = "options", default)]
        pub options: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct FindFriendsInCommonMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapFindFriendsInCommonMessage,
    }

    impl FindFriendsInCommonMessageSoapEnvelope {
        pub fn new(body: SoapFindFriendsInCommonMessage) -> Self {
            FindFriendsInCommonMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

}

pub mod response {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_datatypes::{ArrayOfContactType, ContactType};
    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::service_header::ServiceHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapFindFriendsInCommonResponseMessage {
        #[yaserde(rename = "FindFriendsInCommonResponseMessage", default)]
        pub body: FindFriendsInCommonResponse,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "FindFriendsInCommonResponse",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    prefix = "nsi1"
    )]
    pub struct FindFriendsInCommonResponse {
        #[yaserde(rename = "FindFriendsInCommonResult", default)]
        pub find_friends_in_common_result: Option<FindFriendsInCommonResult>,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "FindFriendsInCommonResult")]
    pub struct FindFriendsInCommonResult {
        #[yaserde(rename = "MatchedList", default)]
        pub matched_list: Option<ArrayOfContactType>,
        #[yaserde(rename = "UnmatchedList", default)]
        pub unmatched_list: Option<ArrayOfContactType>,
        #[yaserde(rename = "MatchedCount", default)]
        pub matched_count: i32,
        #[yaserde(rename = "UnmatchedCount", default)]
        pub unmatched_count: i32,
        #[yaserde(rename = "TargetContact", default)]
        pub target_contact: Option<ContactType>,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct FindFriendsInCommonResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapFindFriendsInCommonResponseMessage,
    }

    impl FindFriendsInCommonResponseMessageSoapEnvelope {
        pub fn new(body: SoapFindFriendsInCommonResponseMessage) -> Self {
            FindFriendsInCommonResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }


}