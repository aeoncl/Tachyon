pub mod request {

    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;
        use crate::soap::abch::ab_service::ab_group_add::request::AbgroupAddMessageSoapEnvelope;

        #[test]
        fn test_ab_group_add_request() {

            let request = r#"<?xml version="1.0" encoding="utf-8"?>
                                <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
                                    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                                    xmlns:xsd="http://www.w3.org/2001/XMLSchema"
                                    xmlns:soapenc="http://schemas.xmlsoap.org/soap/encoding/">
                                    <soap:Header>
                                        <ABApplicationHeader xmlns="http://www.msn.com/webservices/AddressBook">
                                            <ApplicationId>AAD9B99B-58E6-4F23-B975-D9EC1F9EC24A</ApplicationId>
                                            <IsMigration>false</IsMigration>
                                            <PartnerScenario>Initial</PartnerScenario>
                                            <CacheKey>cachekey</CacheKey>
                                        </ABApplicationHeader>
                                        <ABAuthHeader xmlns="http://www.msn.com/webservices/AddressBook">
                                            <ManagedGroupRequest>false</ManagedGroupRequest>
                                            <TicketToken>t=0bfusc4t3dT0k3n</TicketToken>
                                        </ABAuthHeader>
                                    </soap:Header>
                                    <soap:Body>
                                        <ABGroupAdd xmlns="http://www.msn.com/webservices/AddressBook">
                                            <abId>00000000-0000-0000-0000-000000000000</abId>
                                            <groupAddOptions>
                                                <fRenameOnMsgrConflict>false</fRenameOnMsgrConflict>
                                            </groupAddOptions>
                                            <groupInfo>
                                                <GroupInfo>
                                                    <name>Favorites</name>
                                                    <groupType>C8529CE2-6EAD-434d-881F-341E17DB3FF8</groupType>
                                                    <fMessenger>false</fMessenger>
                                                    <IsFavorite>true</IsFavorite>
                                                    <annotations>
                                                        <Annotation>
                                                            <Name>MSN.IM.Display</Name>
                                                            <Value>1</Value>
                                                        </Annotation>
                                                    </annotations>
                                                </GroupInfo>
                                            </groupInfo>
                                        </ABGroupAdd>
                                    </soap:Body>
                                </soap:Envelope>"#;

            let request_deserialized : AbgroupAddMessageSoapEnvelope = from_str(request).unwrap();

            let header = &request_deserialized.header.unwrap();
            assert_eq!(request_deserialized.body.body.ab_id, String::from("00000000-0000-0000-0000-000000000000"));
            assert_eq!(request_deserialized.body.body.group_add_options.f_rename_on_msgr_conflict, Some(false));
            assert_eq!(request_deserialized.body.body.group_info.group_info.group_type, Some(String::from("C8529CE2-6EAD-434d-881F-341E17DB3FF8")));

            assert_eq!(header.ab_auth_header.ticket_token, String::from("t=0bfusc4t3dT0k3n"));
            assert_eq!(header.application_header.partner_scenario, String::from("Initial"));


        }

    }

    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_datatypes::GroupInfoType;
    use crate::soap::abch::request_header::RequestHeaderContainer;


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct SoapAbgroupAddMessage {
        #[yaserde(rename = "ABGroupAdd", prefix="nsi1")]
        pub body: AbgroupAddRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABGroupAddRequest",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1")]
    pub struct AbgroupAddRequestType {
        #[yaserde(rename = "abId", prefix="nsi1")]
        pub ab_id: String,
        #[yaserde(rename = "groupAddOptions",  prefix="nsi1")]
        pub group_add_options: GroupAddOptions,
        #[yaserde(rename = "groupInfo",  prefix="nsi1")]
        pub group_info: GroupInfo,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "groupInfo",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1")]
    pub struct GroupInfo {
        #[yaserde(rename = "GroupInfo", prefix="nsi1")]
        pub group_info: GroupInfoType,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "groupAddOptions",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1")]
    pub struct GroupAddOptions {
        #[yaserde(rename = "fRenameOnMsgrConflict", prefix="nsi1")]
        pub f_rename_on_msgr_conflict: Option<bool>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    namespace = "soapenc: http://schemas.xmlsoap.org/soap/encoding/"
    prefix = "soap"
    )]
    pub struct AbgroupAddMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbgroupAddMessage,
    }

    impl AbgroupAddMessageSoapEnvelope {
        pub fn new(body: SoapAbgroupAddMessage) -> Self {
            AbgroupAddMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

}

pub mod response {

    #[cfg(test)]
    mod tests {
        use yaserde::ser::to_string;
        use crate::soap::abch::ab_service::ab_group_add::response::{AbgroupAddResponse, AbgroupAddResponseMessageSoapEnvelope, AbgroupAddResultType, SoapAbgroupAddResponseMessage};

        #[test]
        fn test_ab_group_add_response() {

            let result = AbgroupAddResultType { guid: String::from("MY_GUID") };
            let group_add_response = AbgroupAddResponse{ ab_group_add_result: Some(result) };

            let body = SoapAbgroupAddResponseMessage{ body: group_add_response, fault: None };
            let response = AbgroupAddResponseMessageSoapEnvelope{ header: None, body  };

            let response_serialized = to_string(&response).unwrap();
            assert!(response_serialized.contains("MY_GUID"));
            assert!(response_serialized.contains("ABGroupAddResponse"));
            assert!(response_serialized.contains("ABGroupAddResult"));
            assert!(response_serialized.contains("guid"));
        }

    }

    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::service_header::ServiceHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbgroupAddResponseMessage {
        #[yaserde(rename = "ABGroupAddResponse", default)]
        pub body: AbgroupAddResponse,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "ABGroupAddResponse",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct AbgroupAddResponse {
        #[yaserde(rename = "ABGroupAddResult", prefix="nsi1")]
        pub ab_group_add_result: Option<AbgroupAddResultType>,
    }

    
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABGroupAddResultType",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1")]
    pub struct AbgroupAddResultType {
        #[yaserde(rename = "guid", prefix="nsi1")]
        pub guid: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct AbgroupAddResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbgroupAddResponseMessage,
    }

    impl AbgroupAddResponseMessageSoapEnvelope {
        pub fn new_favorite_group_added_response(guid: &str, cache_key: &str) -> Self {
            let result = AbgroupAddResultType { guid: guid.to_string() };
            let group_add_response = AbgroupAddResponse{ ab_group_add_result: Some(result) };

            let body = SoapAbgroupAddResponseMessage{ body: group_add_response, fault: None };
            Self{ header: Some(ServiceHeaderContainer::new(cache_key)), body  }
        }
    }
}