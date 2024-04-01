pub mod request {

    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;
        use crate::soap::abch::sharing_service::add_member::request::AddMemberMessageSoapEnvelope;

        #[test]
        fn add_member_request() {
            let request = "<?xml version=\"1.0\" encoding=\"utf-8\"?><soap:Envelope xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:soapenc=\"http://schemas.xmlsoap.org/soap/encoding/\"><soap:Header><ABApplicationHeader xmlns=\"http://www.msn.com/webservices/AddressBook\"><ApplicationId>AAD9B99B-58E6-4F23-B975-D9EC1F9EC24A</ApplicationId><IsMigration>false</IsMigration><PartnerScenario>Timer</PartnerScenario><CacheKey>12r1:8nBBE6vX1J4uPKajtbem5XBIblimCwAhIziAeEAwYD0AMiaztryWvcZthkN9oX_pl2scBKXfKvRvuWKYdHUNuRkgiyV9rzcDpnDIDiM6vdcEB6d82wjjnL4TAFAjc5X8i-C94mNfQvujUk470P7fz9qbWfK6ANcEtygDb-oWsYVfEBrxl6geTUg9tGT7yCIsls7ECcLyqwsROuAbWCrued_VPKiUgSIvqG8gaA</CacheKey></ABApplicationHeader><ABAuthHeader xmlns=\"http://www.msn.com/webservices/AddressBook\"><ManagedGroupRequest>false</ManagedGroupRequest><TicketToken>t=tickettoken</TicketToken></ABAuthHeader></soap:Header><soap:Body><AddMember xmlns=\"http://www.msn.com/webservices/AddressBook\"><serviceHandle><Id>0</Id><Type>Messenger</Type><ForeignId></ForeignId></serviceHandle><memberships><Membership><MemberRole>Allow</MemberRole><Members><Member xsi:type=\"PassportMember\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"><Type>Passport</Type><State>Accepted</State><PassportName>passport@shlasouf.local</PassportName></Member></Members></Membership></memberships></AddMember></soap:Body></soap:Envelope>";
            let request_deserialized : AddMemberMessageSoapEnvelope = from_str(request).unwrap();

            //TODO assertions
        }

    }

    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::sharing_service::find_membership::response::Memberships;
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::msnab_datatypes::HandleType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAddMemberMessage {
        #[yaserde(rename = "AddMember", default)]
        pub body: AddMemberRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "AddMemberRequestType",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1")]
    pub struct AddMemberRequestType {
        #[yaserde(rename = "serviceHandle", default)]
        pub service_handle: HandleType,
        #[yaserde(rename = "memberships", default)]
        pub memberships: Memberships,
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
    pub struct AddMemberMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAddMemberMessage,
    }

    impl AddMemberMessageSoapEnvelope {
        pub fn new(body: SoapAddMemberMessage) -> Self {
            AddMemberMessageSoapEnvelope {
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
        use crate::soap::abch::sharing_service::add_member::response::AddMemberResponseMessageSoapEnvelope;

        #[test]
        fn test_add_member_response() {
            let response = AddMemberResponseMessageSoapEnvelope::new("cachekey");

            let response_serialized = to_string(&response).unwrap();
            println!("{}", response_serialized);

            //TODO assertion
        }


    }

    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::service_header::ServiceHeaderContainer;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAddMemberResponseMessage {
        #[yaserde(rename = "AddMemberResponse", default)]
        pub body: AddMemberResponse,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "AddMemberResponse",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1"
    default_namespace="nsi1"
    )]
    pub struct AddMemberResponse {}

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    namespace = "soapenc: http://schemas.xmlsoap.org/soap/encoding/",
    prefix = "soap"
    )]
    pub struct AddMemberResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAddMemberResponseMessage,
    }

    impl AddMemberResponseMessageSoapEnvelope {
        pub fn new(cache_key: &str) -> Self {
            let body = SoapAddMemberResponseMessage{
                body: AddMemberResponse {},
                fault: None,
            };
            Self{body, header: Some(ServiceHeaderContainer::new(cache_key))}
        }
    }
}