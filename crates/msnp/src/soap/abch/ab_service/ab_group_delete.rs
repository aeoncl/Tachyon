pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_datatypes::{GroupFilterType, Guid};
    use crate::soap::abch::request_header::RequestHeaderContainer;

    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;
        use crate::soap::abch::ab_service::ab_group_delete::request::AbgroupDeleteMessageSoapEnvelope;

        #[test]
        fn deser_test() {

            let raw = r#"<?xml version="1.0" encoding="utf-8"?>
            <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
                 <soap:Header>
                	<ABApplicationHeader xmlns="http://www.msn.com/webservices/AddressBook">
                		<ApplicationId>3794391A-4816-4BAC-B34B-6EC7FB5046C6</ApplicationId>
                		<IsMigration>false</IsMigration>
                		<PartnerScenario>Timer</PartnerScenario>
                	</ABApplicationHeader>
                	<ABAuthHeader xmlns="http://www.msn.com/webservices/AddressBook">
                		<ManagedGroupRequest>false</ManagedGroupRequest>
                		<TicketToken>t=TicketToken</TicketToken>
                	</ABAuthHeader>
                </soap:Header>
            	<soap:Body>
                <ABGroupDelete xmlns="http://www.msn.com/webservices/AddressBook">
                	<abId>00000000-0000-0000-0000-000000000000</abId>
                	<groupFilter>
                		<groupIds>
                			<guid>a267edb0-a29a-4257-8fbe-73468c4c0845</guid>
                			<guid>a267edb0-a29a-4257-8fbe-73468c4c0845</guid>
                		</groupIds>
                	</groupFilter>
                </ABGroupDelete>
            	</soap:Body>
            </soap:Envelope>
            "#;

            let deser = from_str::<AbgroupDeleteMessageSoapEnvelope>(raw).expect("things to work");

            assert_eq!("a267edb0-a29a-4257-8fbe-73468c4c0845", &deser.body.ab_group_delete.group_filter.group_ids.expect("to be here").guid.remove(0).body);


        }


    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbgroupDeleteMessage {
        #[yaserde(rename = "ABGroupDelete", default)]
        pub ab_group_delete: AbgroupDeleteRequestType
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABGroupDeleteRequestType")]
    pub struct AbgroupDeleteRequestType {
        #[yaserde(rename = "abId", default)]
        pub ab_id: Guid,
        #[yaserde(rename = "groupFilter", default)]
        pub group_filter: GroupFilterType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct AbgroupDeleteMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbgroupDeleteMessage,
    }

    impl AbgroupDeleteMessageSoapEnvelope {
        pub fn new(body: SoapAbgroupDeleteMessage) -> Self {
            AbgroupDeleteMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }


}

pub mod response {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::service_header::ServiceHeaderContainer;

    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;
        use crate::soap::abch::ab_service::ab_find_by_contact::request::AbfindByContactsMessageSoapEnvelope;
        use crate::soap::abch::ab_service::ab_group_delete::response::AbgroupDeleteResponseMessageSoapEnvelope;

        #[test]
        fn deser_test() {


            let raw = r#"<?xml version="1.0" encoding="utf-8"?>
            <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xmlns:xsd="http://www.w3.org/2001/XMLSchema">
            	<soap:Header>
            		<ServiceHeader xmlns="http://www.msn.com/webservices/AddressBook">
            			<Version>15.01.1408.0000</Version>
            			<CacheKey>cachekey</CacheKey>
            			<CacheKeyChanged>true</CacheKeyChanged>
            			<PreferredHostName>127.0.0.1</PreferredHostName>
            			<SessionId>sessionid</SessionId>
            		</ServiceHeader>
            	</soap:Header>
            	<soap:Body>
            		<ABGroupDeleteResponse xmlns="http://www.msn.com/webservices/AddressBook"/>
            	</soap:Body>
            </soap:Envelope>
            "#;

             let deser = from_str::<AbgroupDeleteResponseMessageSoapEnvelope>(raw).expect("things to work");



        }


    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbgroupDeleteResponseMessage {
        #[yaserde(rename = "ABGroupDeleteResponse", default)]
        pub body: AbgroupDeleteResponse
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "ABGroupDeleteResponse",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    prefix = "nsi1"
    )]
    pub struct AbgroupDeleteResponse {}

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct AbgroupDeleteResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbgroupDeleteResponseMessage,
    }

    impl AbgroupDeleteResponseMessageSoapEnvelope {
        pub fn new(body: SoapAbgroupDeleteResponseMessage) -> Self {
            AbgroupDeleteResponseMessageSoapEnvelope {
                body,
                header: None
            }
        }
    }



}