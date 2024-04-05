pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_sharingservice::{SOAP_ENCODING};
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::msnab_datatypes::DynamicItems;

    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;
        use crate::soap::abch::ab_service::ab_find_by_contact::request::AbfindByContactsMessageSoapEnvelope;

        #[test]
        fn deser_test() {
                //TODO
            let raw = r#""#;

            // let deser = from_str::<AbfindByContactsMessageSoapEnvelope>(raw).expect("things to work");

        }


    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAddDynamicItemMessage {
        #[yaserde(rename = "AddDynamicItem", default)]
        pub body: AddDynamicItemRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "AddDynamicItemRequestType")]
    pub struct AddDynamicItemRequestType {
        #[yaserde(rename = "abId", default)]
        pub ab_id: String,
        #[yaserde(rename = "dynamicItems", default)]
        pub dynamic_items: DynamicItems,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct AddDynamicItemMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAddDynamicItemMessage,
    }

    impl AddDynamicItemMessageSoapEnvelope {
        pub fn new(body: SoapAddDynamicItemMessage) -> Self {
            AddDynamicItemMessageSoapEnvelope {
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
    pub struct SoapAddDynamicItemResponseMessage {
        #[yaserde(rename = "AddDynamicItemResponseMessage", default)]
        pub body: AddDynamicItemResponseType,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "AddDynamicItemResponseType")]
    pub struct AddDynamicItemResponseType {}

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct AddDynamicItemResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAddDynamicItemResponseMessage,
    }

    impl AddDynamicItemResponseMessageSoapEnvelope {
        pub fn new(body: SoapAddDynamicItemResponseMessage) -> Self {
            AddDynamicItemResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }


}