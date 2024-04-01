pub mod request {

    mod tests {
        use yaserde::de::from_str;
        use yaserde::ser::to_string;
        use crate::soap::abch::ab_service::ab_contact_add::request::AbcontactAddMessageSoapEnvelope;

        #[test]
        fn test_ab_add_contacts(){
            let request = "<?xml version=\"1.0\" encoding=\"utf-8\"?><soap:Envelope xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:soapenc=\"http://schemas.xmlsoap.org/soap/encoding/\"><soap:Header><ABApplicationHeader xmlns=\"http://www.msn.com/webservices/AddressBook\"><ApplicationId>AAD9B99B-58E6-4F23-B975-D9EC1F9EC24A</ApplicationId><IsMigration>false</IsMigration><PartnerScenario>Timer</PartnerScenario><CacheKey>12r1:8nBBE6vX1J4uPKajtbem5XBIblimCwAhIziAeEAwYD0AMiaztryWvcZthkN9oX_pl2scBKXfKvRvuWKYdHUNuRkgiyV9rzcDpnDIDiM6vdcEB6d82wjjnL4TAFAjc5X8i-C94mNfQvujUk470P7fz9qbWfK6ANcEtygDb-oWsYVfEBrxl6geTUg9tGT7yCIsls7ECcLyqwsROuAbWCrued_VPKiUgSIvqG8gaA</CacheKey></ABApplicationHeader><ABAuthHeader xmlns=\"http://www.msn.com/webservices/AddressBook\"><ManagedGroupRequest>false</ManagedGroupRequest><TicketToken>t=0bfusc4t3dT0k3n</TicketToken></ABAuthHeader></soap:Header><soap:Body><ABContactAdd xmlns=\"http://www.msn.com/webservices/AddressBook\"><abId>00000000-0000-0000-0000-000000000000</abId><contacts><Contact xmlns=\"http://www.msn.com/webservices/AddressBook\"><contactInfo><contactType>LivePending</contactType><passportName>test@shlasouf.local</passportName><isMessengerUser>true</isMessengerUser><MessengerMemberInfo><PendingAnnotations><Annotation><Name>MSN.IM.InviteMessage</Name><Value>HI</Value></Annotation></PendingAnnotations><DisplayName>Aeonshl</DisplayName></MessengerMemberInfo></contactInfo></Contact></contacts><options><EnableAllowListManagement>true</EnableAllowListManagement></options></ABContactAdd></soap:Body></soap:Envelope>";
            let request_deserialized : AbcontactAddMessageSoapEnvelope = from_str(request).unwrap();

            assert_eq!(&request_deserialized.header.as_ref().unwrap().ab_auth_header.ticket_token.as_str(), &"t=0bfusc4t3dT0k3n");
            assert_eq!(&request_deserialized.body.body.ab_id.body, &String::from("00000000-0000-0000-0000-000000000000"));
            assert_eq!(request_deserialized.body.body.contacts.as_ref().unwrap().contact.len(), 1usize);
            assert_eq!(request_deserialized.body.body.contacts.as_ref().unwrap().contact[0].contact_info.as_ref().unwrap().passport_name.as_ref().unwrap(), &"test@shlasouf.local".to_string());


            let request_reserialized = to_string(&request_deserialized).unwrap();
        }

    }

    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::ab_service::ab_contact_update::request::Options;
    use crate::soap::abch::msnab_datatypes::{ArrayOfContactType, Guid};
    use crate::soap::abch::msnab_sharingservice::{SOAP_ENCODING};
    use crate::soap::abch::request_header::RequestHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct SoapAbcontactAddMessage {
        #[yaserde(rename = "ABContactAdd", prefix="nsi1")]
        pub body: AbcontactAddRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABContactAddRequestType"
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct AbcontactAddRequestType {
        #[yaserde(rename = "abId", prefix = "nsi1")]
        pub ab_id: Guid,
        #[yaserde(rename = "contacts", prefix = "nsi1")]
        pub contacts: Option<ArrayOfContactType>,
        #[yaserde(rename = "options", prefix = "nsi1")]
        pub options: Option<Options>,
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
    pub struct AbcontactAddMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbcontactAddMessage,
    }

    impl AbcontactAddMessageSoapEnvelope {
        pub fn new(body: SoapAbcontactAddMessage) -> Self {
            AbcontactAddMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }



}

pub mod response {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::shared::models::uuid::Uuid;
    use crate::soap::abch::msnab_datatypes::Guid;
    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::service_header::ServiceHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbcontactAddResponseMessage {
        #[yaserde(rename = "ABContactAddResponse", default)]
        pub body: AbcontactAddResponse,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "ABContactAddResponse",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    default_namespace = "nsi1"
    )]
    pub struct AbcontactAddResponse {
        #[yaserde(rename = "ABContactAddResult", default)]
        pub ab_contact_add_result: Option<AbcontactAddResultType>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABContactAddResultType")]
    pub struct AbcontactAddResultType {
        #[yaserde(rename = "guid")]
        pub guid: Guid,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance"
    namespace = "xsd: http://www.w3.org/2001/XMLSchema"
    prefix = "soap"
    )]
    pub struct AbcontactAddResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbcontactAddResponseMessage,
    }

    impl AbcontactAddResponseMessageSoapEnvelope {
        pub fn get_response(contact_uuid: &Uuid, cache_key: &str) -> Self {

            let body_content = AbcontactAddResponse {
                ab_contact_add_result: Some(AbcontactAddResultType{ guid: Guid{ body: contact_uuid.to_string() } }),
            };

            let body =  SoapAbcontactAddResponseMessage{
                body: body_content,
                fault: None,
            };

            Self { body, header: Some(ServiceHeaderContainer::new(cache_key)) }
        }
    }
}