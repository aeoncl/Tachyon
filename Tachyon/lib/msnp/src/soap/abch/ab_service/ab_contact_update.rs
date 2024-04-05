pub mod request {


    #[cfg(test)]

    mod tests {
        use yaserde::de::from_str;
        use crate::soap::abch::ab_service::ab_contact_update::request::AbcontactUpdateMessageSoapEnvelope;

        #[test]
        fn test_contact_update_request() {
            let request = r#"<?xml version="1.0" encoding="utf-8"?>
                                    <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
                                        xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                                        xmlns:xsd="http://www.w3.org/2001/XMLSchema"
                                        xmlns:soapenc="http://schemas.xmlsoap.org/soap/encoding/">
                                        <soap:Header>
                                            <ABApplicationHeader xmlns="http://www.msn.com/webservices/AddressBook">
                                                <ApplicationId>AAD9B99B-58E6-4F23-B975-D9EC1F9EC24A</ApplicationId>
                                                <IsMigration>false</IsMigration>
                                                <PartnerScenario>Timer</PartnerScenario>
                                                <CacheKey>cachekey</CacheKey>
                                            </ABApplicationHeader>
                                            <ABAuthHeader xmlns="http://www.msn.com/webservices/AddressBook">
                                                <ManagedGroupRequest>false</ManagedGroupRequest>
                                                <TicketToken>t=tickettoken</TicketToken>
                                            </ABAuthHeader>
                                        </soap:Header>
                                        <soap:Body>
                                            <ABContactUpdate xmlns="http://www.msn.com/webservices/AddressBook">
                                                <abId>00000000-0000-0000-0000-000000000000</abId>
                                                <contacts>
                                                    <Contact xmlns="http://www.msn.com/webservices/AddressBook">
                                                        <contactId>11D65AFA-21C2-55E0-90D6-5DB43D5FFE6E</contactId>
                                                        <contactInfo>
                                                            <quickName>♏👽Pelopelo👽♏ (IG)</quickName>
                                                        </contactInfo>
                                                        <propertiesChanged>ContactQuickName</propertiesChanged>
                                                    </Contact>
                                                </contacts>
                                            </ABContactUpdate>
                                        </soap:Body>
                                    </soap:Envelope>"#;

            let request_deserialized : AbcontactUpdateMessageSoapEnvelope = from_str(request).expect("to be here");

            let contact = request_deserialized.body.body.contacts.expect("contact to be here").contact.remove(0);

            assert_eq!("♏👽Pelopelo👽♏ (IG)", &contact.contact_info.expect("contact info to be here").quick_name.expect("quickname to be here"));
            assert_eq!("t=tickettoken", &request_deserialized.header.expect("headers to be here").ab_auth_header.ticket_token)
        }

    }

    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_datatypes::{ArrayOfContactType, Guid};
    use crate::soap::abch::request_header::RequestHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbcontactUpdateMessage {
        #[yaserde(rename = "ABContactUpdate", default)]
        pub body: AbcontactUpdateRequestType
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABContactUpdateRequestType",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1")]
    pub struct AbcontactUpdateRequestType {
        #[yaserde(rename = "abId", default)]
        pub ab_id: Guid,
        #[yaserde(rename = "contacts", default)]
        pub contacts: Option<ArrayOfContactType>,
        #[yaserde(rename = "options", default)]
        pub options: Option<Options>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "options")]
    pub struct Options {
        #[yaserde(rename = "IncludeEmulatedMemberships", default)]
        pub include_emulated_memberships: bool,
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
    pub struct AbcontactUpdateMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbcontactUpdateMessage,
    }

    impl AbcontactUpdateMessageSoapEnvelope {
        pub fn new(body: SoapAbcontactUpdateMessage) -> Self {
            AbcontactUpdateMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }


}

#[cfg(test)]
pub mod response {

    mod tests {
        use yaserde::ser::to_string;
        use crate::soap::abch::ab_service::ab_contact_update::response::AbcontactUpdateResponseMessageSoapEnvelope;


        #[test]
        fn test_contact_update_response() {
            let response = AbcontactUpdateResponseMessageSoapEnvelope::get_response("cachekey");

            let response_serialized = to_string(&response).unwrap();
            println!("{}", response_serialized);

            //TODO assertions

        }

    }

    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::service_header::ServiceHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbcontactUpdateResponseMessage {
        #[yaserde(rename = "ABContactUpdateResponse", default)]
        pub body: AbcontactUpdateResponse,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "ABContactUpdateResponse",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1"
    default_namespace="nsi1"
    )]
    pub struct AbcontactUpdateResponse {}

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    namespace = "soapenc: http://schemas.xmlsoap.org/soap/encoding/",
    prefix = "soap"
    )]
    pub struct AbcontactUpdateResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbcontactUpdateResponseMessage,
    }

    impl AbcontactUpdateResponseMessageSoapEnvelope {
        pub fn get_response(cache_key: &str) -> Self {

            let body =  SoapAbcontactUpdateResponseMessage{
                body: AbcontactUpdateResponse {},
                fault: None,
            };

            Self {body, header: Some(ServiceHeaderContainer::new(cache_key))}
        }
    }




}