pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_datatypes::Guid;
    use crate::soap::abch::msnab_sharingservice::{SOAP_ENCODING};
    use crate::soap::abch::request_header::RequestHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbfindAllMessage {
        #[yaserde(rename = "ABFindAll", default)]
        pub body: AbfindAllRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABFindAllRequestType")]
    pub struct AbfindAllRequestType {
        #[yaserde(rename = "abId", default)]
        pub ab_id: Guid,
        #[yaserde(rename = "abView", default)]
        pub ab_view: Option<String>,
        #[yaserde(rename = "deltasOnly", default)]
        pub deltas_only: Option<bool>,
        #[yaserde(rename = "lastChange", default)]
        pub last_change: Option<String>,
        #[yaserde(rename = "dynamicItemView", default)]
        pub dynamic_item_view: Option<String>,
        #[yaserde(rename = "dynamicItemLastChange", default)]
        pub dynamic_item_last_change: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    prefix = "soap"
    )]
    pub struct AbfindAllMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbfindAllMessage,
    }

    impl AbfindAllMessageSoapEnvelope {
        pub fn new(body: SoapAbfindAllMessage) -> Self {
            AbfindAllMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

}

pub mod response {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::ab_service::ab_find_contacts_paged::response::{Ab, Groups};
    use crate::soap::abch::msnab_datatypes::{ArrayOfContactType, DynamicItems} ;
    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::msnab_sharingservice::{SOAP_ENCODING};
    use crate::soap::abch::request_header::RequestHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbfindAllResponseMessage {
        #[yaserde(rename = "AbfindAllResponseMessage", default)]
        pub body: AbfindAllResponse,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "ABFindAllResponse",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    prefix = "nsi1"
    )]
    pub struct AbfindAllResponse {
        #[yaserde(rename = "ABFindAllResult", default)]
        pub ab_find_all_result: AbfindAllResultType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABFindAllResultType")]
    pub struct AbfindAllResultType {
        #[yaserde(rename = "groups", default)]
        pub groups: Option<Groups>,
        #[yaserde(rename = "contacts", default)]
        pub contacts: Option<ArrayOfContactType>,
        #[yaserde(rename = "DynamicItems", default)]
        pub dynamic_items: Option<DynamicItems>,
        #[yaserde(rename = "CircleResult", default)]
        pub circle_result: CircleResult,
        #[yaserde(rename = "ab", default)]
        pub ab: Ab,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "CircleResult",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1")]
    pub struct CircleResult {
        #[yaserde(rename = "CircleTicket", prefix="nsi1")]
        pub circle_ticket: String,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    prefix = "soap"
    )]
    pub struct AbfindAllResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbfindAllResponseMessage,
    }

    impl AbfindAllResponseMessageSoapEnvelope {
        pub fn new(body: SoapAbfindAllResponseMessage) -> Self {
            AbfindAllResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

}