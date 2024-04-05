pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_datatypes::ArrayOfGuid;
    use crate::soap::abch::msnab_sharingservice::{SOAP_ENCODING};
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::service_header::ServiceHeaderContainer;


    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;
        use crate::soap::abch::ab_service::ab_find_by_contact::request::AbfindByContactsMessageSoapEnvelope;

        #[test]
        fn deser_test() {

            //TODO Endpoint seems to be not used by clients.

            //let raw = r#""#;

           // let deser = from_str::<AbfindByContactsMessageSoapEnvelope>(raw).expect("things to work");



        }


    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbfindByContactsMessage {
        #[yaserde(rename = "ABFindByContacts", default)]
        pub body: AbfindByContactsRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABFindByContacts")]
    pub struct AbfindByContactsRequestType {
        #[yaserde(rename = "abId", default)]
        pub ab_id: String,
        #[yaserde(rename = "abView", default)]
        pub ab_view: String,
        #[yaserde(rename = "contactIds", default)]
        pub contact_ids: Option<ArrayOfGuid>,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct AbfindByContactsMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbfindByContactsMessage,
    }

    impl AbfindByContactsMessageSoapEnvelope {
        pub fn new(body: SoapAbfindByContactsMessage) -> Self {
            AbfindByContactsMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }
}

pub mod response {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_datatypes::{AbType, ArrayOfContactType};
    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::msnab_sharingservice::{SOAP_ENCODING};
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::service_header::ServiceHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbfindByContactsResponseMessage {
        #[yaserde(rename = "ABFindByContactsResponse", default)]
        pub body: AbfindByContactsResponse,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "ABFindByContactsResponse",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    prefix = "nsi1"
    )]
    pub struct AbfindByContactsResponse {
        #[yaserde(rename = "ABFindByContactsResult", default)]
        pub ab_find_by_contacts_result: AbfindByContactsResponseType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABFindByContactsResponseType")]
    pub struct AbfindByContactsResponseType {
        #[yaserde(rename = "contacts", default)]
        pub contacts: Option<ArrayOfContactType>,
        #[yaserde(rename = "ab", default)]
        pub ab: AbType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct AbfindByContactsResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbfindByContactsResponseMessage,
    }

    impl AbfindByContactsResponseMessageSoapEnvelope {
        pub fn new(body: SoapAbfindByContactsResponseMessage) -> Self {
            AbfindByContactsResponseMessageSoapEnvelope {
                body,
                header: None
            }
        }
    }


}