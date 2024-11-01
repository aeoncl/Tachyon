pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_datatypes::AbInfoType;
    use crate::soap::abch::request_header::RequestHeaderContainer;

    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;
        use crate::soap::abch::ab_service::ab_add::request::AbaddMessageSoapEnvelope;


        #[test]
        fn deser_test() {

            let raw = r#"<?xml version="1.0" encoding="utf-8"?>
                            <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
                                xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                                xmlns:xsd="http://www.w3.org/2001/XMLSchema"
                                xmlns:soapenc="http://schemas.xmlsoap.org/soap/encoding/">
                                <soap:Header>
                                    <ABApplicationHeader xmlns="http://www.msn.com/webservices/AddressBook">
                                        <ApplicationId>3794391A-4816-4BAC-B34B-6EC7FB5046C6</ApplicationId>
                                        <IsMigration>false</IsMigration>
                                        <PartnerScenario>Initial</PartnerScenario>
                                    </ABApplicationHeader>
                                    <ABAuthHeader xmlns="http://www.msn.com/webservices/AddressBook">
                                        <ManagedGroupRequest>false</ManagedGroupRequest>
                                        <TicketToken>t=TicketToken</TicketToken>
                                    </ABAuthHeader>
                                </soap:Header>
                                <soap:Body>
                                    <ABAdd xmlns="http://www.msn.com/webservices/AddressBook">
                                        <abInfo>
                                            <name />
                                            <ownerPuid>0</ownerPuid>
                                            <OwnerCID>0</OwnerCID>
                                            <ownerEmail>blablabla@lukewarmmail.com</ownerEmail>
                                            <fDefault>true</fDefault>
                                            <joinedNamespace>false</joinedNamespace>
                                            <IsBot>false</IsBot>
                                            <IsParentManaged>false</IsParentManaged>
                                            <AccountTierLastChanged>0001-01-01T00:00:00</AccountTierLastChanged>
                                            <ProfileVersion>0</ProfileVersion>
                                            <SubscribeExternalPartner>false</SubscribeExternalPartner>
                                            <NotifyExternalPartner>false</NotifyExternalPartner>
                                            <AddressBookType>Individual</AddressBookType>
                                        </abInfo>
                                    </ABAdd>
                                </soap:Body>
                            </soap:Envelope>"#;

            let deser = from_str::<AbaddMessageSoapEnvelope>(raw).expect("to be deserialized correctly");

            assert_eq!("t=TicketToken", &deser.header.expect("header to be here").ab_auth_header.ticket_token);

        }


    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbaddMessage {
        #[yaserde(rename = "ABAdd", default)]
        pub body: AbaddRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABAddRequestType")]
    pub struct AbaddRequestType {
        #[yaserde(rename = "abInfo", default)]
        pub ab_info: AbInfoType,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct AbaddMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbaddMessage,
    }

    impl AbaddMessageSoapEnvelope {
        pub fn new(body: SoapAbaddMessage) -> Self {
            AbaddMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }





}

pub mod response {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::service_header::ServiceHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbaddResponseMessage {
        #[yaserde(rename = "ABAddResponse", default)]
        pub ab_add_response: AbaddResponseType,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABAddResponse")]
    pub struct AbaddResponseType {
        #[yaserde(rename = "ABAddResult", default)]
        pub ab_add_result: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct AbaddResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbaddResponseMessage,
    }

    impl AbaddResponseMessageSoapEnvelope {
        pub fn new(body: SoapAbaddResponseMessage) -> Self {
            AbaddResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }



}