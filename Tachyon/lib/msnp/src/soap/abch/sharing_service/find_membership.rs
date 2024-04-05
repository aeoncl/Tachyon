
pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};

    use crate::soap::abch::msnab_datatypes::ServiceFilter;
    use crate::soap::abch::msnab_sharingservice::SOAP_ENCODING;
    use crate::soap::abch::request_header::RequestHeaderContainer;

    #[cfg(test)]

    mod tests {
        use yaserde::de::from_str;

        use crate::soap::abch::sharing_service::find_membership::request::FindMembershipMessageSoapEnvelope;

        #[test]
        fn test_find_membership_request() {

            let request_body = "<?xml version=\"1.0\" encoding=\"utf-8\"?><soap:Envelope xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:soapenc=\"http://schemas.xmlsoap.org/soap/encoding/\"><soap:Header><ABApplicationHeader xmlns=\"http://www.msn.com/webservices/AddressBook\"><ApplicationId>AAD9B99B-58E6-4F23-B975-D9EC1F9EC24A</ApplicationId><IsMigration>false</IsMigration><PartnerScenario>Initial</PartnerScenario><CacheKey>12r1:8nBBE6vX1J4uPKajtbem5XBIblimCwAhIziAeEAwYD0AMiaztryWvcZthkN9oX_pl2scBKXfKvRvuWKYdHUNuRkgiyV9rzcDpnDIDiM6vdcEB6d82wjjnL4TAFAjc5X8i-C94mNfQvujUk470P7fz9qbWfK6ANcEtygDb-oWsYVfEBrxl6geTUg9tGT7yCIsls7ECcLyqwsROuAbWCrued_VPKiUgSIvqG8gaA</CacheKey></ABApplicationHeader><ABAuthHeader xmlns=\"http://www.msn.com/webservices/AddressBook\"><ManagedGroupRequest>false</ManagedGroupRequest><TicketToken>t=0bfus4t3d_t0k3n</TicketToken></ABAuthHeader></soap:Header><soap:Body><FindMembership xmlns=\"http://www.msn.com/webservices/AddressBook\"><serviceFilter><Types><ServiceType>Messenger</ServiceType><ServiceType>SocialNetwork</ServiceType><ServiceType>Space</ServiceType><ServiceType>Profile</ServiceType></Types></serviceFilter><View>Full</View><deltasOnly>true</deltasOnly><lastChange>2022-04-20T13:03:28Z</lastChange></FindMembership></soap:Body></soap:Envelope>";
            let r: FindMembershipMessageSoapEnvelope = from_str(&request_body).unwrap();

            let header = &r.header.unwrap();
            assert_eq!(r.body.body.deltas_only, true);
            assert_eq!(header.ab_auth_header.ticket_token, String::from("t=0bfus4t3d_t0k3n"));
            assert_eq!(header.application_header.partner_scenario, String::from("Initial"));

        }

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapFindMembershipMessage {
        #[yaserde(rename = "FindMembership", prefix="soap")]
        pub body: FindMembershipRequestType,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "FindMembership",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct FindMembershipRequestType {
        #[yaserde(rename = "serviceFilter", prefix="nsi1")]
        pub service_filter: Option<ServiceFilter>,
        #[yaserde(rename = "view", prefix="nsi1")]
        pub view: String,
        #[yaserde(rename = "expandMembership", prefix="nsi1")]
        pub expand_membership: bool,
        #[yaserde(rename = "deltasOnly", prefix="nsi1")]
        pub deltas_only: bool,
        #[yaserde(rename = "lastChange", prefix="nsi1")]
        pub last_change: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct FindMembershipMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<RequestHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapFindMembershipMessage,
    }

    impl FindMembershipMessageSoapEnvelope {
        pub fn new(body: SoapFindMembershipMessage) -> Self {
            FindMembershipMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

}

pub mod response {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::msnab_datatypes::{BaseMember, HandleType, RoleId};

    use crate::soap::abch::msnab_faults::SoapFault;
    use crate::soap::abch::service_header::ServiceHeaderContainer;

    pub mod factory {
        use chrono::Local;

        use crate::shared::models::uuid::Uuid;
        use crate::soap::abch::sharing_service::find_membership::response::{ArrayOfServiceType, CircleAttributesType, FindMembershipResponse, FindMembershipResponseMessage, FindMembershipResponseMessageSoapEnvelope, Handle, InfoType, Members, Membership, MembershipResult, Memberships, OwnerNamespaceInfoType, OwnerNamespaceType, ServiceType, SoapFindMembershipResponseMessage};
        use crate::soap::abch::msnab_datatypes::{BaseMember, HandleType, RoleId, ServiceName};
        use crate::soap::abch::service_header::ServiceHeaderContainer;

        pub struct FindMembershipResponseFactory;

        impl FindMembershipResponseFactory {

            pub fn get_empty_response(uuid: &Uuid, msn_addr: &str, cache_key: &str, membership_is_complete: bool) -> FindMembershipResponseMessageSoapEnvelope {

                let circle_attributes = CircleAttributesType{ is_presence_enabled: false, is_event: None, domain: String::from("WindowsLive") };
                let handle = Handle { id: Uuid::nil().to_string(), is_passport_name_hidden: false, cid: String::from("0") };
                let owner_namespace_info = OwnerNamespaceInfoType{ handle, creator_puid: String::from("0"), creator_cid: uuid.to_decimal_cid(), creator_passport_name: msn_addr.to_string(), circle_attributes, messenger_application_service_created: Some(false) };

                let now = Local::now();

                let owner_namespace = OwnerNamespaceType{ info: owner_namespace_info, changes: String::new(), create_date: String::from("2014-10-31T00:00:00Z"), last_change: now.format("%Y-%m-%dT%H:%M:%SZ").to_string() };

                let mut services = Vec::new();
                services.push(FindMembershipResponseFactory::get_messenger_service(Vec::new(), Vec::new(), Vec::new(), Vec::new(), membership_is_complete));

                let array_of_services = ArrayOfServiceType{ service: services };

                let result = MembershipResult{ services: Some(array_of_services), owner_namespace: Some(owner_namespace) };
                let fmr = FindMembershipResponse{ find_membership_result: result };
                let body = FindMembershipResponseMessage { find_membership_response: fmr };
                let message = SoapFindMembershipResponseMessage {body, fault: None };

                let mut response = FindMembershipResponseMessageSoapEnvelope::new(message);

                response.header = Some(ServiceHeaderContainer::new(cache_key));
                return response;
            }

            pub fn get_response(uuid: Uuid, msn_addr: &str, cache_key: &str, messenger_service: ServiceType) -> FindMembershipResponseMessageSoapEnvelope {

                let circle_attributes = CircleAttributesType{ is_presence_enabled: false, is_event: None, domain: String::from("WindowsLive") };
                let handle = Handle { id: Uuid::nil().to_string(), is_passport_name_hidden: false, cid: String::from("0") };
                let owner_namespace_info = OwnerNamespaceInfoType{ handle, creator_puid: String::from("0"), creator_cid: uuid.to_decimal_cid(), creator_passport_name: msn_addr.to_string(),  circle_attributes, messenger_application_service_created: Some(false) };

                let now = Local::now();

                let owner_namespace = OwnerNamespaceType{ info: owner_namespace_info, changes: String::new(), create_date: String::from("2014-10-31T00:00:00Z"), last_change: now.format("%Y-%m-%dT%H:%M:%SZ").to_string() };

                let mut services = Vec::new();
                services.push(messenger_service);

                let array_of_services = ArrayOfServiceType{ service: services };

                let result = MembershipResult{ services: Some(array_of_services), owner_namespace: Some(owner_namespace) };
                let fmr = FindMembershipResponse{ find_membership_result: result };
                let body = FindMembershipResponseMessage { find_membership_response: fmr };
                let message = SoapFindMembershipResponseMessage {body:body, fault: None };

                let mut response = FindMembershipResponseMessageSoapEnvelope::new(message);

                response.header = Some(ServiceHeaderContainer::new(cache_key));
                return response;
            }

            pub fn get_messenger_service(allow_list: Vec<BaseMember>, block_list: Vec<BaseMember>, reverse_list: Vec<BaseMember>, pending_list: Vec<BaseMember>, membership_is_complete: bool) -> ServiceType {

                let mut memberships = Vec::new();

                memberships.push(FindMembershipResponseFactory::get_membership(RoleId::Allow, allow_list, membership_is_complete));
                memberships.push(FindMembershipResponseFactory::get_membership(RoleId::Block, block_list, membership_is_complete));
                memberships.push(FindMembershipResponseFactory::get_membership(RoleId::Pending, pending_list, membership_is_complete));
                memberships.push(FindMembershipResponseFactory::get_membership(RoleId::Reverse, reverse_list, membership_is_complete));

                let handle = HandleType{ id: 1, rs_type: ServiceName{ body: "Messenger".to_string() }, foreign_id: Some(String::new()) };
                let info_type = InfoType{ handle: handle, display_name: None, inverse_required: false, authorization_criteria: Some(String::from("Everyone")), rss_url: None, is_bot: false };
                let array_of_membership = Memberships{ membership: memberships };
                let now = Local::now();
                return ServiceType{ memberships: Some(array_of_membership), info: info_type, changes: String::new(), last_change: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(), deleted: false };
            }

            pub fn get_membership(role_id: RoleId, members: Vec<BaseMember>, membership_is_complete: bool) -> Membership {
                let members_array = Members{ member: members };
                return Membership{ member_role: role_id, members: members_array, membership_is_complete: Some(membership_is_complete) };
            }

        }
    }

    #[cfg(test)]
    mod tests {
        use yaserde::de::from_str;
        use yaserde::ser::to_string;

        use crate::shared::models::uuid::Uuid;
        use crate::soap::abch::sharing_service::find_membership::response::{ArrayOfServiceType, CircleAttributesType, FindMembershipResponse, FindMembershipResponseMessage, FindMembershipResponseMessageSoapEnvelope, Handle, InfoType, Members, Membership, MembershipResult, Memberships, OwnerNamespaceInfoType, OwnerNamespaceType, ServiceType, SoapFindMembershipResponseMessage};
        use crate::soap::abch::sharing_service::find_membership::response::factory::FindMembershipResponseFactory;
        use crate::soap::abch::msnab_datatypes::{BaseMember, HandleType, MemberState, MemberType, RoleId, ServiceName};

        #[test]
        fn test_get_empty_find_membership_response() {

            let response = FindMembershipResponseFactory::get_empty_response(&Uuid::from_seed("TEST"), "test@matrix.org", "c4che_key", true);
            let serialized = to_string(&response).unwrap();
        }

        #[test]
        fn test_find_membership() {

            let response = "<?xml version=\"1.0\" encoding=\"utf-8\"?><soap:Envelope xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\"><soap:Header><ServiceHeader xmlns=\"http://www.msn.com/webservices/AddressBook\"><Version>15.01.1408.0000</Version><CacheKey>12r1:VmV4AZeGXqJvemPx973A_p7r6w72NNhyQpzukKLaIV6hBndxsNXKb7zZI0sRgKhI6ClG7T5mnA9ufYtOKNRFtII4vAqvlXGZU7Z2-fcwIZm1309Rr8Ufp1kNEf7-O9qX7nIU18Gfg6b7FBFwAdw-ziCtldVhyoW1ubS5s6TJNasYNDa2SlEyZEo0KZmjmkUD8DvSkU9IaGfk4y_vXRCl39CU5fNaht8MLz4teQ</CacheKey><CacheKeyChanged>true</CacheKeyChanged><PreferredHostName>localhost</PreferredHostName><SessionId>9f5a2db2-fb72-4309-8ac4-4602768ed83b</SessionId></ServiceHeader></soap:Header><soap:Body><FindMembershipResponse xmlns=\"http://www.msn.com/webservices/AddressBook\"><FindMembershipResult><Services><Service><Memberships><Membership><MemberRole>Allow</MemberRole><Members><Member xsi:type=\"PassportMember\"><MembershipId>Allow/bc57b0bc-1edb-4566-8d4d-793edc21ad48</MembershipId><Type>Passport</Type><State>Accepted</State><Deleted>false</Deleted><LastChanged>2020-12-05T10:16:20Z</LastChanged><JoinedDate>2020-12-05T10:05:53Z</JoinedDate><ExpirationDate>0001-01-01T00:00:00</ExpirationDate><Changes /><PassportName>test@test.fr</PassportName><IsPassportNameHidden>false</IsPassportNameHidden><PassportId>0</PassportId><CID>250350177642791142269461059834176712008</CID><PassportChanges /><LookedupByCID>false</LookedupByCID></Member></Members><MembershipIsComplete>true</MembershipIsComplete></Membership><Membership><MemberRole>Block</MemberRole><Members></Members><MembershipIsComplete>true</MembershipIsComplete></Membership><Membership><MemberRole>Reverse</MemberRole><Members><Member xsi:type=\"PassportMember\"><MembershipId>Reverse/bc57b0bc-1edb-4566-8d4d-793edc21ad48</MembershipId><Type>Passport</Type><State>Accepted</State><Deleted>false</Deleted><LastChanged>2020-12-05T10:16:20Z</LastChanged><JoinedDate>2020-12-05T10:05:53Z</JoinedDate><ExpirationDate>0001-01-01T00:00:00</ExpirationDate><Changes /><PassportName>test@test.fr</PassportName><IsPassportNameHidden>false</IsPassportNameHidden><PassportId>0</PassportId><CID>250350177642791142269461059834176712008</CID><PassportChanges /><LookedupByCID>false</LookedupByCID></Member></Members><MembershipIsComplete>true</MembershipIsComplete></Membership><Membership><MemberRole>Pending</MemberRole><Members></Members><MembershipIsComplete>true</MembershipIsComplete></Membership></Memberships><Info><Handle><Id>1</Id><Type>Messenger</Type><ForeignId /></Handle><InverseRequired>false</InverseRequired><AuthorizationCriteria>Everyone</AuthorizationCriteria><IsBot>false</IsBot></Info><Changes /><LastChange>2020-12-05T10:16:20Z</LastChange><Deleted>false</Deleted></Service></Services><OwnerNamespace><Info><Handle><Id>00000000-0000-0000-0000-000000000000</Id><IsPassportNameHidden>false</IsPassportNameHidden><CID>0</CID></Handle><CreatorPuid>0</CreatorPuid><CreatorCID>-8633146156665081703</CreatorCID><CreatorPassportName>aeon@test.fr</CreatorPassportName><CircleAttributes><IsPresenceEnabled>false</IsPresenceEnabled><Domain>WindowsLive</Domain></CircleAttributes><MessengerApplicationServiceCreated>false</MessengerApplicationServiceCreated></Info><Changes /><CreateDate>2020-12-05T10:16:20Z</CreateDate><LastChange>2020-12-05T10:16:20Z</LastChange></OwnerNamespace></FindMembershipResult></FindMembershipResponse></soap:Body></soap:Envelope>";

            let r: FindMembershipResponseMessageSoapEnvelope = from_str(&response).unwrap();
            let test = r.body.body.find_membership_response.find_membership_result.services;
            let test2 = 0;
        }

        #[test]
        fn test_find_membership_2() {

            let circle_attributes = CircleAttributesType{ is_presence_enabled: false, is_event: None, domain: String::from("WindowsLive") };
            let handle = Handle { id: String::from("00000000-0000-0000-0000-000000000000"), is_passport_name_hidden: false, cid: String::from("0") };
            let owner_namespace_info = OwnerNamespaceInfoType{ handle, creator_puid: String::from("0"), creator_cid: 863314, creator_passport_name: String::from("aeon@test.fr"), circle_attributes: circle_attributes, messenger_application_service_created: Some(false) };
            let owner_namespace = OwnerNamespaceType{ info: owner_namespace_info, changes: "Hi".to_string(), create_date: "date".to_string(), last_change: "date".to_string() };

            let mut members = Vec::new();
            let member = BaseMember{ membership_id: Some(String::from("faefaef")), rs_type: MemberType::Passport, location: None, display_name: Some("displayName".to_string()), state: MemberState::Accepted, new_role: None, annotations: None, deleted: Some(false), last_changed: Some("date".to_string()), joined_date: Some("date".to_string()), expiration_date: Some("date".to_string()), changes: None, xsi_type: String::from("faefa"), passport_name: None, circle_id: None, is_passport_name_hidden: None, passport_id: None, cid: None, passport_changes: None, lookedup_by_cid: None };

            members.push(member);

            let members_array = Members{ member: members };

            let role_id = RoleId::Allow;

            let mut memberships = Vec::new();

            let membership = Membership{ member_role: role_id, members: members_array, membership_is_complete: Some(true) };

            memberships.push(membership);
            let array_of_membership = Memberships{ membership: memberships };

            let handle = HandleType{ id: 123, rs_type: ServiceName{ body: "feadea".to_string() }, foreign_id: None };
            let info_type = InfoType{ handle: handle, display_name: Some("display_name".to_string()), inverse_required: false, authorization_criteria: None, rss_url: None, is_bot: false };

            let mut services = Vec::new();
            let dummy_service = ServiceType{ memberships: Some(array_of_membership), info: info_type, changes: "changes".to_string(), last_change: "last change".to_string(), deleted: false };
            services.push(dummy_service);

            let array_of_services = ArrayOfServiceType{ service: services };

            let result = MembershipResult{ services: Some(array_of_services), owner_namespace: Some(owner_namespace) };
            let fmr = FindMembershipResponse{ find_membership_result: result };
            let body = FindMembershipResponseMessage { find_membership_response: fmr };
            let message = SoapFindMembershipResponseMessage {body:body, fault: None };
            let test = FindMembershipResponseMessageSoapEnvelope::new(message);
            let test_string = to_string(&test).unwrap();
            println!("{}",test_string);
            let test1 = 0;

            let r: FindMembershipResponseMessageSoapEnvelope = from_str(&test_string).unwrap();
            let test = r.body.body.find_membership_response.find_membership_result.services;
            let test2 = 0;
        }


    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapFindMembershipResponseMessage {
        #[yaserde(rename = "FindMembershipResponse", default)]
        pub body: FindMembershipResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "FindMembershipResponse")]
    pub struct FindMembershipResponseMessage {
        #[yaserde(flatten, default)]
        pub find_membership_response: FindMembershipResponse,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "FindMembershipResponse",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct FindMembershipResponse {
        #[yaserde(rename = "FindMembershipResult", default)]
        pub find_membership_result: MembershipResult,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "MembershipResult", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1")]
    pub struct MembershipResult {
        #[yaserde(rename = "Services", default, prefix = "nsi1", namespace = "nsi1: http://www.msn.com/webservices/AddressBook", default_namespace="nsi1")]
        pub services: Option<ArrayOfServiceType>,
        #[yaserde(rename = "OwnerNamespace", default, prefix = "nsi1", namespace = "nsi1: http://www.msn.com/webservices/AddressBook", default_namespace="nsi1")]
        pub owner_namespace: Option<OwnerNamespaceType>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "OwnerNamespaceType",namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct OwnerNamespaceType {
        #[yaserde(rename = "Info", prefix = "nsi1")]
        pub info: OwnerNamespaceInfoType,
        #[yaserde(rename = "Changes", prefix = "nsi1")]
        pub changes: String,
        #[yaserde(rename = "CreateDate", prefix = "nsi1")]
        pub create_date: String,
        #[yaserde(rename = "LastChange", prefix = "nsi1")]
        pub last_change: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "OwnerNamespaceInfoType",namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct OwnerNamespaceInfoType {
        #[yaserde(rename = "Handle", prefix = "nsi1")]
        pub handle: Handle,
        #[yaserde(rename = "CreatorPuid", prefix = "nsi1")]
        pub creator_puid: String,
        #[yaserde(rename = "CreatorCID", prefix = "nsi1")]
        pub creator_cid: i64,
        #[yaserde(rename = "CreatorPassportName", prefix = "nsi1")]
        pub creator_passport_name: String,
        #[yaserde(rename = "CircleAttributes", prefix = "nsi1")]
        pub circle_attributes: CircleAttributesType,
        #[yaserde(rename = "MessengerApplicationServiceCreated", prefix = "nsi1")]
        pub messenger_application_service_created: Option<bool>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "CircleAttributesType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct CircleAttributesType {
        #[yaserde(rename = "IsPresenceEnabled", prefix = "nsi1")]
        pub is_presence_enabled: bool,
        #[yaserde(rename = "IsEvent", prefix = "nsi1")]
        pub is_event: Option<bool>,
        #[yaserde(rename = "Domain", prefix = "nsi1")]
        pub domain: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "Handle", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct Handle {
        #[yaserde(rename = "Id", prefix = "nsi1")]
        pub id: String,
        #[yaserde(rename = "IsPassportNameHidden", prefix = "nsi1")]
        pub is_passport_name_hidden: bool,
        #[yaserde(rename = "CID", prefix = "nsi1")]
        pub cid: String,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "ArrayOfServiceType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct ArrayOfServiceType {
        #[yaserde(rename = "Service", prefix = "nsi1")]
        pub service: Vec<ServiceType>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soap"
    )]
    pub struct FindMembershipResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapFindMembershipResponseMessage,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "ServiceType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct ServiceType {
        #[yaserde(rename = "Memberships", prefix = "nsi1")]
        pub memberships: Option<Memberships>,
        #[yaserde(rename = "Info", prefix = "nsi1")]
        pub info: InfoType,
        #[yaserde(rename = "Changes", prefix = "nsi1")]
        pub changes: String,
        #[yaserde(rename = "LastChange", prefix = "nsi1")]
        pub last_change: String,
        #[yaserde(rename = "Deleted", prefix = "nsi1")]
        pub deleted: bool,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "InfoType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct InfoType {
        #[yaserde(rename = "Handle", prefix = "nsi1")]
        pub handle: HandleType,
        #[yaserde(rename = "DisplayName", prefix = "nsi1")]
        pub display_name: Option<String>,
        #[yaserde(rename = "InverseRequired", prefix = "nsi1")]
        pub inverse_required: bool,
        #[yaserde(rename = "AuthorizationCriteria", prefix = "nsi1")]
        pub authorization_criteria: Option<String>,
        #[yaserde(rename = "RSSUrl", prefix = "nsi1")]
        pub rss_url: Option<String>,
        #[yaserde(rename = "IsBot", prefix = "nsi1")]
        pub is_bot: bool,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "Memberships", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct Memberships {
        #[yaserde(rename = "Membership", prefix = "nsi1")]
        pub membership: Vec<Membership>,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "Members", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct Members {
        #[yaserde(rename = "Member", prefix = "nsi1")]
        pub member: Vec<BaseMember>,
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "Membership", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct Membership {
        #[yaserde(rename = "MemberRole", prefix = "nsi1")]
        pub member_role: RoleId,
        #[yaserde(rename = "Members", prefix = "nsi1")]
        pub members: Members,
        #[yaserde(rename = "MembershipIsComplete", prefix = "nsi1")]
        pub membership_is_complete: Option<bool>,
    }

    impl FindMembershipResponseMessageSoapEnvelope {
        pub fn new(body: SoapFindMembershipResponseMessage) -> Self {
            FindMembershipResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }


}