pub mod requets {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_datatypes::{ArrayOfContactType, GroupFilterType, Guid};
    use crate::soap::abch::request_header::RequestHeaderContainer;

    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;
        use crate::soap::abch::ab_service::ab_group_contact_delete::requets::AbgroupContactDeleteMessageSoapEnvelope;

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
                                    <ABGroupContactDelete xmlns="http://www.msn.com/webservices/AddressBook">
                                        <abId>00000000-0000-0000-0000-000000000000</abId>
                                        <contacts>
                                            <Contact>
                             					<contactId>1</contactId>
                             				</Contact>
                             				<Contact>
                             					<contactId>2</contactId>
                             				</Contact>
                                        </contacts>
                                        <groupFilter>
                                            <groupIds>
                             					<guid>a267edb0-a29a-4257-8fbe-73468c4c0845</guid>
                             					<guid>a267edb0-a29a-4257-8fbe-73468c4c0845</guid>
                             				</groupIds>
                                        </groupFilter>
		                            </ABGroupContactDelete>
                                </soap:Body>
                            </soap:Envelope>
                            "#;

            let deser = from_str::<AbgroupContactDeleteMessageSoapEnvelope>(raw).expect("things to work");

            assert_eq!("a267edb0-a29a-4257-8fbe-73468c4c0845", &deser.body.body.group_filter.group_ids.expect("to be here").guid.remove(0).body);
            assert_eq!("1", &deser.body.body.contacts.expect("to be here").contact.remove(0).contact_id.expect("to be here"));
        }


    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbgroupContactDeleteMessage {
        #[yaserde(rename = "ABGroupContactDelete", default)]
        pub body: AbgroupContactDeleteRequestType
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABGroupContactDeleteRequestType")]
    pub struct AbgroupContactDeleteRequestType {
        #[yaserde(rename = "abId", default)]
        pub ab_id: Guid,
        #[yaserde(rename = "contacts", default)]
        pub contacts: Option<ArrayOfContactType>,
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
    pub struct AbgroupContactDeleteMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbgroupContactDeleteMessage,
    }

    impl AbgroupContactDeleteMessageSoapEnvelope {
        pub fn new(body: SoapAbgroupContactDeleteMessage) -> Self {
            AbgroupContactDeleteMessageSoapEnvelope {
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
        use crate::soap::abch::ab_service::ab_group_contact_delete::response::AbgroupContactDeleteResponseMessageSoapEnvelope;
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
                              			<PreferredHostName>host</PreferredHostName>
                              			<SessionId>sessionid</SessionId>
                              		</ServiceHeader>
                              	</soap:Header>
                              	<soap:Body>
                              		<ABGroupContactDeleteResponse xmlns="http://www.msn.com/webservices/AddressBook"/>
                              	</soap:Body>
                              </soap:Envelope>
            "#;

            let deser = from_str::<AbgroupContactDeleteResponseMessageSoapEnvelope>(raw).expect("things to work");
        }


    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbgroupContactDeleteResponseMessage {
        #[yaserde(rename = "ABGroupContactDeleteResponse", default)]
        pub body: AbgroupContactDeleteResponse,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "ABGroupContactDeleteResponse",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    prefix = "nsi1"
    )]
    pub struct AbgroupContactDeleteResponse {}

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct AbgroupContactDeleteResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbgroupContactDeleteResponseMessage,
    }

    impl AbgroupContactDeleteResponseMessageSoapEnvelope {
        pub fn new(body: SoapAbgroupContactDeleteResponseMessage) -> Self {
            AbgroupContactDeleteResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }



}