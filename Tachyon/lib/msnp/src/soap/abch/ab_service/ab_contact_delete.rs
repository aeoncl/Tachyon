pub mod request {

    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;
        use yaserde::ser::to_string;
        use crate::soap::abch::ab_service::ab_contact_delete::request::AbcontactDeleteMessageSoapEnvelope;

        #[test]
        fn test_ab_delete_contacts(){
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
                                            <TicketToken>t=0bfusc4t3dT0k3n</TicketToken>
                                        </ABAuthHeader>
                                    </soap:Header>
                                    <soap:Body>
                                        <ABContactDelete xmlns="http://www.msn.com/webservices/AddressBook">
                                            <abId>00000000-0000-0000-0000-000000000000</abId>
                                            <contacts>
                                                <Contact>
                                                    <contactId>12E0CAB8-3381-5DDF-ADF9-B55281751A6F</contactId>
                                                </Contact>
                                            </contacts>
                                        </ABContactDelete>
                                    </soap:Body>
                                </soap:Envelope>"#;

            let request_deserialized : AbcontactDeleteMessageSoapEnvelope = from_str(request).expect("to be here");

            assert_eq!(&request_deserialized.header.as_ref().expect("to be here").ab_auth_header.ticket_token.as_str(), &"t=0bfusc4t3dT0k3n");
            assert_eq!(&request_deserialized.body.ab_contact_delete.ab_id.body, &String::from("00000000-0000-0000-0000-000000000000"));
            assert_eq!(request_deserialized.body.ab_contact_delete.contacts.as_ref().expect("to be here").contact.len(), 1usize);

            let request_reserialized = to_string(&request_deserialized).expect("to have worked");
        }

    }

    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_datatypes::{ArrayOfContactType, Guid};
    use crate::soap::abch::msnab_sharingservice::{SOAP_ENCODING};
    use crate::soap::abch::request_header::RequestHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "SoapAbcontactDeleteMessage",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct SoapAbcontactDeleteMessage {
        #[yaserde(rename = "ABContactDelete", prefix = "nsi1")]
        pub ab_contact_delete: AbcontactDeleteRequestType
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABContactDeleteRequestType",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    prefix = "nsi1",
    default_namespace="nsi1")]
    pub struct AbcontactDeleteRequestType {
        #[yaserde(rename = "abId", prefix = "nsi1")]
        pub ab_id: Guid,
        #[yaserde(rename = "contacts", prefix = "nsi1")]
        pub contacts: Option<ArrayOfContactType>,
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
    pub struct AbcontactDeleteMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbcontactDeleteMessage,
    }

    impl AbcontactDeleteMessageSoapEnvelope {
        pub fn new(body: SoapAbcontactDeleteMessage) -> Self {
            AbcontactDeleteMessageSoapEnvelope {
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
        use crate::soap::abch::ab_service::ab_contact_delete::response::AbcontactDeleteResponseMessageSoapEnvelope;

        #[test]
        fn test_ab_contact_delete_response() {
            let response = AbcontactDeleteResponseMessageSoapEnvelope::get_response("cachekey");
            let response_serialized = to_string(&response).unwrap();
            println!("{}", response_serialized);
        }


    }

    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::service_header::ServiceHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbcontactDeleteResponseMessage {
        #[yaserde(rename = "ABContactDeleteResponse", default)]
        pub body: AbcontactDeleteResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABContactDeleteResponse",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct AbcontactDeleteResponseMessage {
        #[yaserde(default)]
        pub ab_contact_delete_response: Option<String>,
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
    pub struct AbcontactDeleteResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbcontactDeleteResponseMessage,
    }

    impl AbcontactDeleteResponseMessageSoapEnvelope {
        pub fn get_response(cache_key: &str) -> Self {

            let body_content = AbcontactDeleteResponseMessage {
                ab_contact_delete_response: None,
            };

            let body =  SoapAbcontactDeleteResponseMessage{
                body: body_content,
                fault: None,
            };

            Self { body, header:Some(ServiceHeaderContainer::new(cache_key))}
        }
    }




}