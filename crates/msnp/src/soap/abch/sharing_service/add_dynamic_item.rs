pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};

    use crate::soap::abch::msnab_datatypes::DynamicItems;
    use crate::soap::abch::request_header::RequestHeaderContainer;

    #[cfg(test)]
    mod tests {
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
    use yaserde::ser::to_string;
    use yaserde_derive::{YaDeserialize, YaSerialize};

    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::service_header::ServiceHeaderContainer;
    use crate::soap::abch::sharing_service::find_membership::response::FindMembershipResponseMessageSoapEnvelope;
    use crate::soap::error::SoapMarshallError;
    use crate::soap::traits::xml::ToXml;

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

    impl ToXml for AddDynamicItemResponseMessageSoapEnvelope {
        type Error = SoapMarshallError;
        fn to_xml(&self) -> Result<String, Self::Error>  {
            to_string(self).map_err(|e| SoapMarshallError::SerializationError { message: e})
        }

    }


}