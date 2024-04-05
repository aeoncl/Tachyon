pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::ab_service::ab_find_contacts_paged::response::Groups;
    use crate::soap::abch::msnab_datatypes::Guid;
    use crate::soap::abch::msnab_sharingservice::SOAP_ENCODING;
    use crate::soap::abch::request_header::RequestHeaderContainer;

    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;
        use crate::soap::abch::ab_service::ab_group_update::request::AbgroupUpdateMessageSoapEnvelope;

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
                                      <ABGroupUpdate xmlns="http://www.msn.com/webservices/AddressBook">
                                      	<abId>00000000-0000-0000-0000-000000000000</abId>
                                      	<groups>
                                      		<Group>
                                      			<groupId>a267edb0-a29a-4257-8fbe-73468c4c0845</groupId>
                                      			<groupInfo>
                                      				<name>hi</name>
                                      			</groupInfo>
                                      			<propertiesChanged>GroupName</propertiesChanged>
                                      		</Group>
                                      	</groups>
                                      </ABGroupUpdate>
                                  </soap:Body>
                              </soap:Envelope>
                              "#;

            let deser = from_str::<AbgroupUpdateMessageSoapEnvelope>(raw).expect("things to work");

            assert_eq!("00000000-0000-0000-0000-000000000000", &deser.body.ab_group_update.ab_id.body);
            assert_eq!("GroupName", &deser.body.ab_group_update.groups.group.get(0).as_ref().expect("to be here").properties_changed);

        }


    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbgroupUpdateMessage {
        #[yaserde(rename = "ABGroupUpdate", default)]
        pub ab_group_update: AbgroupUpdateRequestType
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABGroupUpdateRequestType")]
    pub struct AbgroupUpdateRequestType {
        #[yaserde(rename = "abId", default)]
        pub ab_id: Guid,
        #[yaserde(rename = "groups", default)]
        pub groups: Groups,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct AbgroupUpdateMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbgroupUpdateMessage,
    }

    impl AbgroupUpdateMessageSoapEnvelope {
        pub fn new(body: SoapAbgroupUpdateMessage) -> Self {
            AbgroupUpdateMessageSoapEnvelope {
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

    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;
        use crate::soap::abch::ab_service::ab_group_delete::response::AbgroupDeleteResponseMessageSoapEnvelope;
        use crate::soap::abch::ab_service::ab_group_update::response::AbgroupUpdateResponseMessageSoapEnvelope;

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
                              		<ABGroupUpdateResponse xmlns="http://www.msn.com/webservices/AddressBook"/>
                              	</soap:Body>
                              </soap:Envelope>
                              "#;

            let deser = from_str::<AbgroupUpdateResponseMessageSoapEnvelope>(raw).expect("things to work");

        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbgroupUpdateResponseMessage {
        #[yaserde(rename = "ABGroupUpdateResponse", default)]
        pub response: AbgroupUpdateResponse,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "ABGroupUpdateResponse",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    prefix = "nsi1"
    )]
    pub struct AbgroupUpdateResponse {}

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct AbgroupUpdateResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbgroupUpdateResponseMessage,
    }

    impl AbgroupUpdateResponseMessageSoapEnvelope {
        pub fn new(body: SoapAbgroupUpdateResponseMessage) -> Self {
            AbgroupUpdateResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }


}