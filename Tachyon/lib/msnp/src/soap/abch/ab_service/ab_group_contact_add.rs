pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_datatypes::{ArrayOfContactType, GroupFilterType, Guid};
    use crate::soap::abch::msnab_sharingservice::{SOAP_ENCODING};
    use crate::soap::abch::request_header::RequestHeaderContainer;

    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;
        use crate::soap::abch::ab_service::ab_group_contact_add::request::AbgroupContactAddMessageSoapEnvelope;

        #[test]
        fn deser_test() {

            let raw = r#"<?xml version="1.0" encoding="utf-8"?>
                             <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
                             	xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                             	xmlns:xsd="http://www.w3.org/2001/XMLSchema">

                             	<soap:Header>
                             		<ABApplicationHeader xmlns="http://www.msn.com/webservices/AddressBook">
                             			<ApplicationId>MSN_APPLICATION_ID</ApplicationId>
                             			<IsMigration>false</IsMigration>
                             			<PartnerScenario>Initial</PartnerScenario>
                             		</ABApplicationHeader>
                             		<ABAuthHeader xmlns="http://www.msn.com/webservices/AddressBook">
                             			<ManagedGroupRequest>false</ManagedGroupRequest>
                             			<TicketToken>t=TicketToken</TicketToken>
                             		</ABAuthHeader>
                             	</soap:Header>
                             	<soap:Body>
                             		<ABGroupContactAdd xmlns="http://www.msn.com/webservices/AddressBook">
                             			<abId>a267edb0-a29a-4257-8fbe-73468c4c0845</abId>
                             			<groupFilter>
                             				<groupIds>
                             					<guid>a267edb0-a29a-4257-8fbe-73468c4c0845</guid>
                             					<guid>a267edb0-a29a-4257-8fbe-73468c4c0845</guid>
                             					<guid>a267edb0-a29a-4257-8fbe-73468c4c0845</guid>
                             				</groupIds>
                             			</groupFilter>
                             			<contacts>
                             				<Contact>
                             					<contactId>1</contactId>
                             				</Contact>
                             				<Contact>
                             					<contactId>2</contactId>
                             				</Contact>
                             			</contacts>
                             			<groupContactAddOptions>
                             				<fGenerateMissingQuickName>true</fGenerateMissingQuickName>
                             				<EnableAllowListManagement>true</EnableAllowListManagement>
                             			</groupContactAddOptions>
                             		</ABGroupContactAdd>
                             	</soap:Body>
                             </soap:Envelope>
                           "#;

            let deser = from_str::<AbgroupContactAddMessageSoapEnvelope>(raw).expect("deser to work");

            assert_eq!(true, deser.body.ab_group_contact_add.group_contact_add_options.expect("to be here").enable_allow_list_management.expect("to be here"));
            assert_eq!("a267edb0-a29a-4257-8fbe-73468c4c0845", &deser.body.ab_group_contact_add.group_filter.group_ids.expect("to be here").guid[0].body);
            assert_eq!("1", &deser.body.ab_group_contact_add.contacts.expect("to be here").contact.remove(0).contact_id.expect("to be here"));
        }

    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Body",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    prefix = "soap",
    default_namespace="soap"
    )]
    pub struct SoapAbgroupContactAddMessage {
        #[yaserde(rename = "ABGroupContactAdd", default)]
        pub ab_group_contact_add: AbgroupContactAddRequestType
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABGroupContactAdd",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1")]
    pub struct AbgroupContactAddRequestType {
        #[yaserde(rename = "abId", default)]
        pub ab_id: Guid,
        #[yaserde(rename = "groupFilter", default)]
        pub group_filter: GroupFilterType,
        #[yaserde(rename = "contacts", default)]
        pub contacts: Option<ArrayOfContactType>,
        #[yaserde(rename = "groupContactAddOptions", default)]
        pub group_contact_add_options: Option<GroupContactAddOptions>,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "groupContactAddOptions",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct GroupContactAddOptions {
        #[yaserde(rename = "fGenerateMissingQuickName", prefix = "nsi1")]
        pub f_generate_missing_quick_name: Option<bool>,
        #[yaserde(rename = "EnableAllowListManagement", prefix = "nsi1")]
        pub enable_allow_list_management: Option<bool>,
    }



    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct AbgroupContactAddMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbgroupContactAddMessage,
    }

    impl AbgroupContactAddMessageSoapEnvelope {
        pub fn new(body: SoapAbgroupContactAddMessage) -> Self {
            AbgroupContactAddMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

}

pub mod response {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_datatypes::Guid;
    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::msnab_sharingservice::{SOAP_ENCODING};
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::service_header::ServiceHeaderContainer;

    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;
        use crate::soap::abch::ab_service::ab_group_contact_add::response::AbgroupContactAddResponseMessageSoapEnvelope;

        #[test]
        fn deser_test() {

            let raw = r#"<?xml version="1.0" encoding="utf-8"?>
                                <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
                                    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                                    xmlns:xsd="http://www.w3.org/2001/XMLSchema">
                                    <soap:Header>
                                        <ServiceHeader xmlns="http://www.msn.com/webservices/AddressBook">
                                            <Version>15.01.1408.0000</Version>
                                            <CacheKey>johnny-cache</CacheKey>
                                            <CacheKeyChanged>true</CacheKeyChanged>
                                            <PreferredHostName>127.0.0.1</PreferredHostName>
                                            <SessionId>sesshhhh</SessionId>
                                        </ServiceHeader>
                                    </soap:Header>
                                    <soap:Body>
                                        <ABGroupContactAddResponse xmlns="http://www.msn.com/webservices/AddressBook">
                                            <ABGroupContactAddResult>
                                                <guid>a267edb0-a29a-4257-8fbe-73468c4c0845</guid>
                                            </ABGroupContactAddResult>
                                        </ABGroupContactAddResponse>
                                    </soap:Body>
                                </soap:Envelope>"#;

            let deser = from_str::<AbgroupContactAddResponseMessageSoapEnvelope>(raw).expect("things to work");

        }

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbgroupContactAddResponseMessage {
        #[yaserde(rename = "ABGroupContactAddResponse", default)]
        pub body: AbgroupContactAddResponse,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "ABGroupContactAddResponse",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    prefix = "nsi1"
    )]
    pub struct AbgroupContactAddResponse {
        #[yaserde(rename = "ABGroupContactAddResult", default)]
        pub ab_group_contact_add_result: Option<AbgroupContactAddResultType>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABGroupContactAddResultType")]
    pub struct AbgroupContactAddResultType {
        #[yaserde(rename = "guid", default)]
        pub guid: Guid,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    prefix = "soap"
    )]
    pub struct AbgroupContactAddResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbgroupContactAddResponseMessage,
    }

    impl AbgroupContactAddResponseMessageSoapEnvelope {
        pub fn new(body: SoapAbgroupContactAddResponseMessage) -> Self {
            AbgroupContactAddResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }


}