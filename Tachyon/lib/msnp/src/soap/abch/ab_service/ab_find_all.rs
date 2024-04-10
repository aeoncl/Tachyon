pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_datatypes::Guid;
    use crate::soap::abch::request_header::RequestHeaderContainer;


    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;
        use crate::soap::abch::ab_service::ab_find_all::request::AbfindAllMessageSoapEnvelope;

        #[test]
        fn deser_test() {

            let raw = r#"<?xml version="1.0" encoding="utf-8"?>
            <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:soapenc="http://schemas.xmlsoap.org/soap/encoding/">
 	            <soap:Header>
 	        	<ABApplicationHeader xmlns="http://www.msn.com/webservices/AddressBook">
 	        		<ApplicationId>FD897282-977D-48D0-A478-4EE3FB2B754E</ApplicationId>
 	        		<IsMigration>false</IsMigration>
 	        		<PartnerScenario>Initial</PartnerScenario>
 	        	</ABApplicationHeader>
 	        	<ABAuthHeader xmlns="http://www.msn.com/webservices/AddressBook">
 	        	    <TicketToken>t=TicketToken</TicketToken>
 	        		<ManagedGroupRequest>false</ManagedGroupRequest>
 	        	</ABAuthHeader>
 	            </soap:Header>
 	            <soap:Body>
 	        	    <ABFindAll xmlns="http://www.msn.com/webservices/AddressBook">
 	        		    <abId>00000000-0000-0000-0000-000000000000</abId>
 	        		    <abView>Full</abView>
 	        		    <deltasOnly>true</deltasOnly>
 	        		    <dynamicItemView>Gleam</dynamicItemView>
 	        		    <lastChange>0001-01-01T00:00:00.0000000-08:00</lastChange>
 	        	    </ABFindAll>
 	            </soap:Body>
            </soap:Envelope>"#;

            let deser = from_str::<AbfindAllMessageSoapEnvelope>(raw).expect("things to work");

            assert_eq!("Gleam", &deser.body.body.dynamic_item_view.expect("to be here"));
        }


    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbfindAllMessage {
        #[yaserde(rename = "ABFindAll", default)]
        pub body: AbfindAllRequestType
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABFindAllRequestType",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1"
    )]
    pub struct AbfindAllRequestType {
        #[yaserde(rename = "abId", prefix = "nsi1")]
        pub ab_id: Guid,
        #[yaserde(rename = "abView", prefix = "nsi1")]
        pub ab_view: Option<String>,
        #[yaserde(rename = "deltasOnly", prefix = "nsi1")]
        pub deltas_only: Option<bool>,
        #[yaserde(rename = "lastChange", prefix = "nsi1")]
        pub last_change: Option<String>,
        #[yaserde(rename = "dynamicItemView", prefix = "nsi1")]
        pub dynamic_item_view: Option<String>,
        #[yaserde(rename = "dynamicItemLastChange", prefix = "nsi1")]
        pub dynamic_item_last_change: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct AbfindAllMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbfindAllMessage,
    }

    impl AbfindAllMessageSoapEnvelope {
        pub fn new(body: SoapAbfindAllMessage) -> Self {
            AbfindAllMessageSoapEnvelope {
                body,
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
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::service_header::ServiceHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbfindAllResponseMessage {
        #[yaserde(rename = "ABFindAllResponse", default)]
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
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct AbfindAllResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbfindAllResponseMessage,
    }

    impl AbfindAllResponseMessageSoapEnvelope {
        pub fn new(body: SoapAbfindAllResponseMessage) -> Self {
            AbfindAllResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

}