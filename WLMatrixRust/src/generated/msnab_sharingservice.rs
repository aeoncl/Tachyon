//! THIS IS A GENERATED FILE!
//! Take care when hand editing. Changes will be lost during subsequent runs of the code generator.
//!
//! version: 0.1.5
//!

#![allow(dead_code)]
#![allow(unused_imports)]
use crate::generated::msnab_datatypes::types::*;
use log::{debug, warn};
use std::io::{Read, Write};
use yaserde_derive::{YaDeserialize, YaSerialize};
pub const SOAP_ENCODING: &str = "http://www.w3.org/2003/05/soap-encoding";
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
pub struct Header {}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
    rename = "Fault",
    namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
    prefix = "soap"
)]
pub struct SoapFault {
    #[yaserde(rename = "faultcode", default)]
    pub fault_code: Option<String>,
    #[yaserde(rename = "faultstring", default)]
    pub fault_string: Option<String>,
}
impl std::error::Error for SoapFault {}

impl std::fmt::Display for SoapFault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.fault_code, &self.fault_string) {
            (None, None) => Ok(()),
            (None, Some(fault_string)) => f.write_str(fault_string),
            (Some(fault_code), None) => f.write_str(fault_code),
            (Some(fault_code), Some(fault_string)) => {
                f.write_str(fault_code)?;
                f.write_str(": ")?;
                f.write_str(fault_string)
            }
        }
    }
}
pub type SoapResponse = Result<(reqwest::StatusCode, String), reqwest::Error>;

pub mod messages {
    use super::*;
    use super::types::{AbapplicationHeader, AbauthHeader};
    use async_trait::async_trait;
    use yaserde::de::from_str;
    use yaserde::ser::to_string;
    use yaserde::{YaDeserialize, YaSerialize};
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABHeader")]
    pub struct Abheader {
        #[yaserde(flatten, default)]
        pub application_header: types::AbapplicationHeader,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "FindMembershipMessage")]
    pub struct FindMembershipMessage {
        #[yaserde(flatten, default)]
        pub find_membership_request: types::FindMembership,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "FindMembershipByRoleMessage")]
    pub struct FindMembershipByRoleMessage {
        #[yaserde(flatten, default)]
        pub find_membership_by_role_request: types::FindMembershipByRole,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABFindAllMessage")]
    pub struct AbfindAllMessage {
        #[yaserde(flatten, default)]
        pub ab_find_all_request: types::AbfindAll,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABContactAddMessage")]
    pub struct AbcontactAddMessage {
        #[yaserde(flatten, default)]
        pub ab_contact_add_request: types::AbcontactAdd,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABContactDeleteMessage")]
    pub struct AbcontactDeleteMessage {
        #[yaserde(flatten, default)]
        pub ab_contact_delete_request: types::AbcontactDelete,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "DeleteContactMessage")]
    pub struct DeleteContactMessage {
        #[yaserde(flatten, default)]
        pub delete_contact_request: types::DeleteContact,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABGroupContactAddMessage")]
    pub struct AbgroupContactAddMessage {
        #[yaserde(flatten, default)]
        pub ab_group_contact_add_request: types::AbgroupContactAdd,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABGroupAddMessage")]
    pub struct AbgroupAddMessage {
        #[yaserde(flatten, default)]
        pub ab_group_add_request: types::AbgroupAdd,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABGroupUpdateMessage")]
    pub struct AbgroupUpdateMessage {
        #[yaserde(flatten, default)]
        pub ab_group_update_request: types::AbgroupUpdate,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABGroupDeleteMessage")]
    pub struct AbgroupDeleteMessage {
        #[yaserde(flatten, default)]
        pub ab_group_delete_request: types::AbgroupDelete,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABGroupContactDeleteMessage")]
    pub struct AbgroupContactDeleteMessage {
        #[yaserde(flatten, default)]
        pub ab_group_contact_delete_request: types::AbgroupContactDelete,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABContactUpdateMessage")]
    pub struct AbcontactUpdateMessage {
        #[yaserde(flatten, default)]
        pub ab_contact_update_request: types::AbcontactUpdate,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "AddMemberMessage")]
    pub struct AddMemberMessage {
        #[yaserde(flatten, default)]
        pub add_member_request: types::AddMember,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "DeleteMemberMessage")]
    pub struct DeleteMemberMessage {
        #[yaserde(flatten, default)]
        pub delete_member_request: types::DeleteMember,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "AddServiceMessage")]
    pub struct AddServiceMessage {
        #[yaserde(flatten, default)]
        pub add_service_request: types::AddService,
    }

    
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "Header")]
    pub struct ServiceHeaderContainer {

        #[yaserde(rename="ServiceHeader")]
        pub service_header: messages::ServiceHeader
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "Header")]
    pub struct RequestHeaderContainer{

        #[yaserde(rename="ABApplicationHeader")]
        pub application_header: AbapplicationHeader,

        #[yaserde(rename="ABAuthHeader")]
        pub ab_auth_header: AbauthHeader,


    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ServiceHeader")]
    pub struct ServiceHeader {
        #[yaserde(flatten, default)]
        pub service_header: types::ServiceHeader,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "FindMembershipResponse")]
    pub struct FindMembershipResponseMessage {
        #[yaserde(flatten, default)]
        pub find_membership_response: types::FindMembershipResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "FindMembershipByRoleResponseMessage")]
    pub struct FindMembershipByRoleResponseMessage {
        #[yaserde(flatten, default)]
        pub find_membership_by_role_response: types::FindMembershipByRoleResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABFindAllResponseMessage")]
    pub struct AbfindAllResponseMessage {
        #[yaserde(flatten, default)]
        pub ab_find_all_response: types::AbfindAllResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABContactAddResponseMessage")]
    pub struct AbcontactAddResponseMessage {
        #[yaserde(flatten, default)]
        pub ab_contact_add_response: types::AbcontactAddResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABContactDeleteResponseMessage")]
    pub struct AbcontactDeleteResponseMessage {
        #[yaserde(default)]
        pub ab_contact_delete_response: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "DeleteContactResponseMessage")]
    pub struct DeleteContactResponseMessage {
        #[yaserde(default)]
        pub delete_contact_response: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABGroupContactAddResponseMessage")]
    pub struct AbgroupContactAddResponseMessage {
        #[yaserde(flatten, default)]
        pub ab_group_contact_add_response: types::AbgroupContactAddResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABGroupAddResponse")]
    pub struct AbgroupAddResponseMessage {
        #[yaserde(flatten, default)]
        pub ab_group_add_response: types::AbgroupAddResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABGroupUpdateResponseMessage")]
    pub struct AbgroupUpdateResponseMessage {
        #[yaserde(flatten, default)]
        pub ab_group_update_response: types::AbgroupUpdateResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABGroupDeleteResponseMessage")]
    pub struct AbgroupDeleteResponseMessage {
        #[yaserde(flatten, default)]
        pub ab_group_delete_response: types::AbgroupDeleteResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABGroupContactDeleteResponseMessage")]
    pub struct AbgroupContactDeleteResponseMessage {
        #[yaserde(flatten, default)]
        pub ab_group_contact_delete_response: types::AbgroupContactDeleteResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABContactUpdateResponseMessage")]
    pub struct AbcontactUpdateResponseMessage {
        #[yaserde(flatten, default)]
        pub ab_contact_update_response: types::AbcontactUpdateResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "AddMemberResponseMessage")]
    pub struct AddMemberResponseMessage {
        #[yaserde(flatten, default)]
        pub add_member_response: types::AddMemberResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "DeleteMemberResponseMessage")]
    pub struct DeleteMemberResponseMessage {
        #[yaserde(flatten, default)]
        pub delete_member_response: types::DeleteMemberResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "AddServiceResponseMessage")]
    pub struct AddServiceResponseMessage {
        #[yaserde(flatten, default)]
        pub add_service_response: types::AddServiceResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "InvalidPassportUserMessage")]
    pub struct InvalidPassportUserMessage {
        #[yaserde(flatten, default)]
        pub fault: InvalidPassportUser,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABAddMessage")]
    pub struct AbaddMessage {
        #[yaserde(flatten, default)]
        pub ab_add_request: types::Abadd,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABAddResponseMessage")]
    pub struct AbaddResponseMessage {
        #[yaserde(flatten, default)]
        pub ab_add_response: types::AbaddResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "UpdateDynamicItemMessage")]
    pub struct UpdateDynamicItemMessage {
        #[yaserde(flatten, default)]
        pub update_dynamic_item: types::UpdateDynamicItem,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "UpdateDynamicItemResponseMessage")]
    pub struct UpdateDynamicItemResponseMessage {
        #[yaserde(default)]
        pub update_dynamic_item_response: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABFindContactsPagedMessage")]
    pub struct AbfindContactsPagedMessage {
        #[yaserde(flatten, default)]
        pub ab_find_contacts_paged_request: types::AbfindContactsPaged,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABFindContactsPagedResponse")]
    pub struct AbfindContactsPagedResponseMessage {
        #[yaserde(flatten, default)]
        pub ab_find_contacts_paged_response: types::AbfindContactsPagedResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "FindFriendsInCommonMessage")]
    pub struct FindFriendsInCommonMessage {
        #[yaserde(flatten, default)]
        pub find_friends_in_common_request: types::FindFriendsInCommon,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "FindFriendsInCommonResponseMessage")]
    pub struct FindFriendsInCommonResponseMessage {
        #[yaserde(flatten, default)]
        pub find_friends_in_common_response: types::FindFriendsInCommonResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "GetContactsRecentActivityMessage")]
    pub struct GetContactsRecentActivityMessage {
        #[yaserde(flatten, default)]
        pub get_contacts_recent_activity_request: types::GetContactsRecentActivity,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "GetContactsRecentActivityResponseMessage")]
    pub struct GetContactsRecentActivityResponseMessage {
        #[yaserde(flatten, default)]
        pub get_contacts_recent_activity_response: types::GetContactsRecentActivityResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "WNHeader")]
    pub struct Wnheader {
        #[yaserde(flatten, default)]
        pub wn_application_header: types::WnapplicationHeader,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "CreateCircleMessage")]
    pub struct CreateCircleMessage {
        #[yaserde(flatten, default)]
        pub create_circle_request: types::CreateCircle,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "CreateCircleResponseMessage")]
    pub struct CreateCircleResponseMessage {
        #[yaserde(flatten, default)]
        pub create_circle_response: types::CreateCircleResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "CreateContactMessage")]
    pub struct CreateContactMessage {
        #[yaserde(flatten, default)]
        pub create_contact_request: types::CreateContact,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "CreateContactResponseMessage")]
    pub struct CreateContactResponseMessage {
        #[yaserde(flatten, default)]
        pub create_contact_response: types::CreateContactResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ManageWLConnectionMessage")]
    pub struct ManageWLConnectionMessage {
        #[yaserde(flatten, default)]
        pub manage_wl_connection: types::ManageWLConnection,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ManageWLConnectionResponseMessage")]
    pub struct ManageWLConnectionResponseMessage {
        #[yaserde(flatten, default)]
        pub manage_wl_connection_response: types::ManageWLConnectionResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "BreakConnectionMessage")]
    pub struct BreakConnectionMessage {
        #[yaserde(flatten, default)]
        pub break_connection: types::BreakConnection,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "BreakConnectionResponseMessage")]
    pub struct BreakConnectionResponseMessage {
        #[yaserde(flatten, default)]
        pub break_connection_response: types::BreakConnectionResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "AddDynamicItemMessage")]
    pub struct AddDynamicItemMessage {
        #[yaserde(flatten, default)]
        pub add_dynamic_item: types::AddDynamicItem,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "AddDynamicItemResponseMessage")]
    pub struct AddDynamicItemResponseMessage {
        #[yaserde(flatten, default)]
        pub add_dynamic_item_response: types::AddDynamicItemResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABFindByContactsMessage")]
    pub struct AbfindByContactsMessage {
        #[yaserde(flatten, default)]
        pub ab_find_by_contacts: types::AbfindByContacts,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABFindByContactsResponseMessage")]
    pub struct AbfindByContactsResponseMessage {
        #[yaserde(flatten, default)]
        pub ab_find_by_contacts_response: types::AbfindByContactsResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "GetBatchRecentActivityMessage")]
    pub struct GetBatchRecentActivityMessage {
        #[yaserde(flatten, default)]
        pub get_batch_recent_activity: types::GetBatchRecentActivity,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "GetBatchRecentActivityResponseMessage")]
    pub struct GetBatchRecentActivityResponseMessage {
        #[yaserde(flatten, default)]
        pub get_batch_recent_activity_response: types::GetBatchRecentActivityResponse,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "WNServiceHeader")]
    pub struct WnserviceHeader {
        #[yaserde(flatten, default)]
        pub wn_service_header: types::WnserviceHeader,
    }
}

pub mod types {
    use super::*;
    use async_trait::async_trait;
    use yaserde::de::from_str;
    use yaserde::ser::to_string;
    use yaserde::{YaDeserialize, YaSerialize};
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ABApplicationHeader",
        namespace = "soap: http://www.msn.com/webservices/AddressBook",
        prefix = "soap"
        default_namespace="soap"
    )]
    pub struct AbapplicationHeader {
        #[yaserde(rename = "ApplicationId", prefix="soap")]
        pub application_id: String,
        #[yaserde(rename = "IsMigration", prefix="soap")]
        pub is_migration: bool,
        #[yaserde(rename = "PartnerScenario", prefix="soap")]
        pub partner_scenario: String,
        #[yaserde(rename = "CacheKey", prefix="soap")]
        pub cache_key: Option<String>,
        #[yaserde(rename = "BrandId", prefix="soap")]
        pub brand_id: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ABAuthHeader",
        namespace = "soap: http://www.msn.com/webservices/AddressBook",
        prefix = "soap"
        default_namespace="soap"
    )]
    pub struct AbauthHeader {
        #[yaserde(rename = "ManagedGroupRequest", prefix="soap")]
        pub managed_group_request: bool,
        #[yaserde(rename = "TicketToken", prefix="soap")]
        pub ticket_token: String,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ServiceHeader",
        namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
        prefix = "nsi1",
        default_namespace="nsi1"
    )]
    pub struct ServiceHeader {
        #[yaserde(rename = "Version", prefix="nsi1")]
        pub version: String,
        #[yaserde(rename = "CacheKey", prefix="nsi1")]
        pub cache_key: Option<String>,
        #[yaserde(rename = "CacheKeyChanged", prefix="nsi1")]
        pub cache_key_changed: Option<bool>,
        #[yaserde(rename = "PreferredHostName", prefix="nsi1")]
        pub preferred_host_name: Option<String>,
        #[yaserde(rename = "SessionId", prefix="nsi1")]
        pub session_id: Option<String>,
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
    pub type FindMembership = FindMembershipRequestType;

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
    pub type FindMembershipByRole = FindMembershipByRoleRequestType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "options")]
    pub struct Options {
        #[yaserde(rename = "IncludeEmulatedMemberships", default)]
        pub include_emulated_memberships: bool,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "FindMembershipByRoleRequestType")]
    pub struct FindMembershipByRoleRequestType {
        #[yaserde(rename = "serviceFilter", default)]
        pub service_filter: Option<ServiceFilter>,
        #[yaserde(rename = "includedRoles", default)]
        pub included_roles: Option<ArrayOfRoleId>,
        #[yaserde(rename = "view", default)]
        pub view: String,
        #[yaserde(rename = "expandMembership", default)]
        pub expand_membership: bool,
        #[yaserde(rename = "options", default)]
        pub options: Option<Options>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "FindMembershipByRoleResponse",
        namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
        namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
        prefix = "nsi1"
    )]
    pub struct FindMembershipByRoleResponse {
        #[yaserde(rename = "FindMembershipByRoleResult", default)]
        pub find_membership_by_role_result: MembershipResult,
    }
    pub type AbfindAll = AbfindAllRequestType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABFindAllRequestType")]
    pub struct AbfindAllRequestType {
        #[yaserde(rename = "abId", default)]
        pub ab_id: Guid,
        #[yaserde(rename = "abView", default)]
        pub ab_view: Option<String>,
        #[yaserde(rename = "deltasOnly", default)]
        pub deltas_only: Option<bool>,
        #[yaserde(rename = "lastChange", default)]
        pub last_change: Option<String>,
        #[yaserde(rename = "dynamicItemView", default)]
        pub dynamic_item_view: Option<String>,
        #[yaserde(rename = "dynamicItemLastChange", default)]
        pub dynamic_item_last_change: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "groups",    
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1"
    default_namespace="nsi1")]
    pub struct Groups {
        #[yaserde(rename = "Group", prefix="nsi1")]
        pub group: Vec<GroupType>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "DynamicItems",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1")]
    pub struct DynamicItems {
        #[yaserde(rename = "DynamicItem", prefix="nsi1")]
        pub dynamic_item: Vec<BaseDynamicItemType>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "CircleResult",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1")]
    pub struct CircleResult {
        #[yaserde(rename = "CircleTicket", prefix="nsi1")]
        pub circle_ticket: String,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ab")]
    pub struct Ab {
        #[yaserde(rename = "abId", default)]
        pub ab_id: String,
        #[yaserde(rename = "abInfo", default)]
        pub ab_info: AbInfoType,
        #[yaserde(rename = "lastChange", default)]
        pub last_change: String,
        #[yaserde(rename = "DynamicItemLastChanged", default)]
        pub dynamic_item_last_changed: String,
        #[yaserde(rename = "RecentActivityItemLastChanged", default)]
        pub recent_activity_item_last_changed: Option<String>,
        #[yaserde(rename = "createDate", default)]
        pub create_date: String,
        #[yaserde(rename = "propertiesChanged", default)]
        pub properties_changed: String,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABFindAllResultType")]
    pub struct AbfindAllResultType {
        #[yaserde(rename = "groups", default)]
        pub groups: Option<Groups>,
        #[yaserde(rename = "contacts", default)]
        pub contacts: Option<ArrayOfContactType>,
        #[yaserde(rename = "DynamicItems", default)]
        pub dynamic_items: Option<DynamicItems>,
        #[yaserde(rename = "CircleResult", default)]
        pub circle_result: CircleResult,
        #[yaserde(rename = "ab", default)]
        pub ab: Ab,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ABFindAllResponse",
        namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
        namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
        prefix = "nsi1"
    )]
    pub struct AbfindAllResponse {
        #[yaserde(rename = "ABFindAllResult", default)]
        pub ab_find_all_result: AbfindAllResultType,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABFindByContactsRequestType")]
    pub struct AbfindByContactsRequestType {
        #[yaserde(rename = "abId", default)]
        pub ab_id: String,
        #[yaserde(rename = "abView", default)]
        pub ab_view: String,
        #[yaserde(rename = "contactIds", default)]
        pub contact_ids: Option<ArrayOfGuid>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABFindByContactsResponseType")]
    pub struct AbfindByContactsResponseType {
        #[yaserde(rename = "contacts", default)]
        pub contacts: Option<ArrayOfContactType>,
        #[yaserde(rename = "ab", default)]
        pub ab: AbType,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ABFindByContactsResponse",
        namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
        namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
        prefix = "nsi1"
    )]
    pub struct AbfindByContactsResponse {
        #[yaserde(rename = "ABFindByContactsResult", default)]
        pub ab_find_by_contacts_result: AbfindByContactsResponseType,
    }
    pub type AbfindByContacts = AbfindByContactsRequestType;

    pub type AbcontactAdd = AbcontactAddRequestType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABContactAddRequestType")]
    pub struct AbcontactAddRequestType {
        #[yaserde(rename = "abId", default)]
        pub ab_id: Guid,
        #[yaserde(rename = "contacts", default)]
        pub contacts: Option<ArrayOfContactType>,
        #[yaserde(rename = "options", default)]
        pub options: Option<Options>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABContactAddResultType")]
    pub struct AbcontactAddResultType {
        #[yaserde(rename = "guid", default)]
        pub guid: Guid,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ABContactAddResponse",
        namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
        namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
        prefix = "nsi1"
    )]
    pub struct AbcontactAddResponse {
        #[yaserde(rename = "ABContactAddResult", default)]
        pub ab_contact_add_result: Option<AbcontactAddResultType>,
    }
    pub type AbcontactDelete = AbcontactDeleteRequestType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABContactDeleteRequestType")]
    pub struct AbcontactDeleteRequestType {
        #[yaserde(rename = "abId", default)]
        pub ab_id: Guid,
        #[yaserde(rename = "contacts", default)]
        pub contacts: Option<ArrayOfContactType>,
    }

    pub type DeleteContact = DeleteContactRequestType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "DeleteContactRequestType")]
    pub struct DeleteContactRequestType {
        #[yaserde(rename = "contactId", default)]
        pub contact_id: Guid,
    }

    pub type AbgroupContactAdd = AbgroupContactAddRequestType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "groupContactAddOptions")]
    pub struct GroupContactAddOptions {
        #[yaserde(rename = "fGenerateMissingQuickName", default)]
        pub f_generate_missing_quick_name: Option<bool>,
        #[yaserde(rename = "EnableAllowListManagement", default)]
        pub enable_allow_list_management: Option<bool>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABGroupContactAddRequestType")]
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
    #[yaserde(rename = "ABGroupContactAddResultType")]
    pub struct AbgroupContactAddResultType {
        #[yaserde(rename = "guid", default)]
        pub guid: Guid,
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
    pub type AbgroupAdd = AbgroupAddRequestType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "groupAddOptions",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1")]
    pub struct GroupAddOptions {
        #[yaserde(rename = "fRenameOnMsgrConflict", prefix="nsi1")]
        pub f_rename_on_msgr_conflict: Option<bool>,
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
    #[yaserde(rename = "ABGroupAddResultType",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
    prefix = "nsi1",
    default_namespace="nsi1")]
    pub struct AbgroupAddResultType {
        #[yaserde(rename = "guid", prefix="nsi1")]
        pub guid: String,
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
    pub type AbgroupUpdate = AbgroupUpdateRequestType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABGroupUpdateRequestType")]
    pub struct AbgroupUpdateRequestType {
        #[yaserde(rename = "abId", default)]
        pub ab_id: Guid,
        #[yaserde(rename = "groups", default)]
        pub groups: Groups,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ABGroupUpdateResponse",
        namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
        namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
        prefix = "nsi1"
    )]
    pub struct AbgroupUpdateResponse {}
    pub type AbgroupDelete = AbgroupDeleteRequestType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABGroupDeleteRequestType")]
    pub struct AbgroupDeleteRequestType {
        #[yaserde(rename = "abId", default)]
        pub ab_id: Guid,
        #[yaserde(rename = "groupFilter", default)]
        pub group_filter: GroupFilterType,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ABGroupDeleteResponse",
        namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
        namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
        prefix = "nsi1"
    )]
    pub struct AbgroupDeleteResponse {}
    pub type AbcontactUpdate = AbcontactUpdateRequestType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABContactUpdateRequestType")]
    pub struct AbcontactUpdateRequestType {
        #[yaserde(rename = "abId", default)]
        pub ab_id: Guid,
        #[yaserde(rename = "contacts", default)]
        pub contacts: Option<ArrayOfContactType>,
        #[yaserde(rename = "options", default)]
        pub options: Option<Options>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ABContactUpdateResponse",
        namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
        namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
        prefix = "nsi1"
    )]
    pub struct AbcontactUpdateResponse {}
    pub type AbgroupContactDelete = AbgroupContactDeleteRequestType;

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
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ABGroupContactDeleteResponse",
        namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
        namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
        prefix = "nsi1"
    )]
    pub struct AbgroupContactDeleteResponse {}
    pub type AddMember = AddMemberRequestType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "memberships")]
    pub struct Memberships {
        #[yaserde(rename = "Membership", default)]
        pub membership: Vec<Membership>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "AddMemberRequestType")]
    pub struct AddMemberRequestType {
        #[yaserde(rename = "serviceHandle", default)]
        pub service_handle: HandleType,
        #[yaserde(rename = "memberships", default)]
        pub memberships: Memberships,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "AddMemberResponse",
        namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
        namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
        prefix = "nsi1"
    )]
    pub struct AddMemberResponse {}
    pub type DeleteMember = DeleteMemberRequestType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "DeleteMemberRequestType")]
    pub struct DeleteMemberRequestType {
        #[yaserde(rename = "serviceHandle", default)]
        pub service_handle: HandleType,
        #[yaserde(rename = "memberships", default)]
        pub memberships: Memberships,
        #[yaserde(rename = "nsHandle", default)]
        pub ns_handle: Option<ContentHandleType>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "DeleteMemberResponse",
        namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
        namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
        prefix = "nsi1"
    )]
    pub struct DeleteMemberResponse {}
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABAddResponseType")]
    pub struct AbaddResponseType {
        #[yaserde(rename = "ABAddResult", default)]
        pub ab_add_result: String,
    }
    pub type AbaddResponse = AbaddResponseType;

    pub type Abadd = AbaddRequestType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABAddRequestType")]
    pub struct AbaddRequestType {
        #[yaserde(rename = "abInfo", default)]
        pub ab_info: AbInfoType,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "UpdateDynamicItemRequestType",
    namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1")]
    pub struct UpdateDynamicItemRequestType {
        #[yaserde(rename = "abId", prefix="nsi1")]
        pub ab_id: String,
        #[yaserde(rename = "dynamicItems", prefix="nsi1")]
        pub dynamic_items: DynamicItems,
    }
    pub type UpdateDynamicItem = UpdateDynamicItemRequestType;

    pub type AbfindContactsPaged = AbfindContactsPagedRequestType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABFindContactsPagedRequestType",
        namespace = "soap: http://www.msn.com/webservices/AddressBook",
        prefix = "soap",
        default_namespace="soap"
    )]
    pub struct AbfindContactsPagedRequestType {
        #[yaserde(rename = "filterOptions", prefix="soap")]
        pub filter_options: FilterOptionsType,
        #[yaserde(rename = "abView", prefix="soap")]
        pub ab_view: String,
        #[yaserde(rename = "extendedContent", prefix="soap")]
        pub extended_content: String,
        #[yaserde(rename = "abHandle", prefix="soap")]
        pub ab_handle: Option<AbHandleType>,
        #[yaserde(rename = "pageContext", prefix="soap")]
        pub page_context: Option<PageContextType>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ABFindContactsPagedResult", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1")]
    pub struct AbfindContactsPagedResultType {
        #[yaserde(rename = "Groups", prefix="nsi1")]
        pub groups: Option<Groups>,
        #[yaserde(rename = "Contacts", prefix="nsi1")]
        pub contacts: Option<ArrayOfContactType>,
        #[yaserde(rename = "CircleResult", prefix="nsi1")]
        pub circle_result: CircleResultType,
        #[yaserde(rename = "Ab", prefix="nsi1")]
        pub ab: Ab,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ABFindContactsPagedResponse",
        namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
        prefix = "nsi1"
        default_namespace="nsi1"
    )]
    pub struct AbfindContactsPagedResponse {
        #[yaserde(rename = "ABFindContactsPagedResult", prefix="nsi1")]
        pub ab_find_contacts_paged_result: AbfindContactsPagedResultType,
    }
    pub type FindFriendsInCommon = FindFriendsInCommonRequestType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "FindFriendsInCommonRequestType")]
    pub struct FindFriendsInCommonRequestType {
        #[yaserde(rename = "targetAB", default)]
        pub target_ab: Option<AbHandleType>,
        #[yaserde(rename = "domainID", default)]
        pub domain_id: i32,
        #[yaserde(rename = "view", default)]
        pub view: String,
        #[yaserde(rename = "maxResults", default)]
        pub max_results: i32,
        #[yaserde(rename = "options", default)]
        pub options: String,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "FindFriendsInCommonResponse",
        namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
        namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
        prefix = "nsi1"
    )]
    pub struct FindFriendsInCommonResponse {
        #[yaserde(rename = "FindFriendsInCommonResult", default)]
        pub find_friends_in_common_result: Option<FindFriendsInCommonResult>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "FindFriendsInCommonResult")]
    pub struct FindFriendsInCommonResult {
        #[yaserde(rename = "MatchedList", default)]
        pub matched_list: Option<ArrayOfContactType>,
        #[yaserde(rename = "UnmatchedList", default)]
        pub unmatched_list: Option<ArrayOfContactType>,
        #[yaserde(rename = "MatchedCount", default)]
        pub matched_count: i32,
        #[yaserde(rename = "UnmatchedCount", default)]
        pub unmatched_count: i32,
        #[yaserde(rename = "TargetContact", default)]
        pub target_contact: Option<ContactType>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "WNApplicationHeaderType")]
    pub struct WnapplicationHeaderType {
        #[yaserde(rename = "ApplicationId", default)]
        pub application_id: Guid,
        #[yaserde(rename = "RenderingApplicationId", default)]
        pub rendering_application_id: Option<Guid>,
    }
    pub type WnapplicationHeader = WnapplicationHeaderType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "WNAuthHeaderType")]
    pub struct WnauthHeaderType {
        #[yaserde(rename = "TicketToken", default)]
        pub ticket_token: String,
    }
    pub type WnauthHeader = WnauthHeaderType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "WNServiceHeaderType")]
    pub struct WnserviceHeaderType {
        #[yaserde(rename = "Version", default)]
        pub version: String,
        #[yaserde(rename = "CacheKey", default)]
        pub cache_key: Option<String>,
        #[yaserde(rename = "CacheKeyChanged", default)]
        pub cache_key_changed: Option<bool>,
        #[yaserde(rename = "PreferredHostName", default)]
        pub preferred_host_name: Option<String>,
        #[yaserde(rename = "InExperimentalSample", default)]
        pub in_experimental_sample: Option<bool>,
    }
    pub type WnserviceHeader = WnserviceHeaderType;

    pub type GetBatchRecentActivity = GetBatchRecentActivityRequestType;

    pub type GetContactsRecentActivity = GetContactsRecentActivityRequestType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "templateTypes")]
    pub struct TemplateTypes {
        #[yaserde(rename = "string", default)]
        pub string: Vec<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "GetBatchRecentActivityRequestType")]
    pub struct GetBatchRecentActivityRequestType {
        #[yaserde(rename = "entityHandle", default)]
        pub entity_handle: EntityHandle,
        #[yaserde(rename = "locales", default)]
        pub locales: Locale,
        #[yaserde(rename = "count", default)]
        pub count: i32,
        #[yaserde(rename = "templateTypes", default)]
        pub template_types: TemplateTypes,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "GetContactsRecentActivityRequestType")]
    pub struct GetContactsRecentActivityRequestType {
        #[yaserde(rename = "entityHandle", default)]
        pub entity_handle: EntityHandle,
        #[yaserde(rename = "locales", default)]
        pub locales: Locale,
        #[yaserde(rename = "count", default)]
        pub count: i32,
    }
    pub type GetBatchRecentActivityResponse = GetBatchRecentActivityResultType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "GetContactsRecentActivityResponse",
        namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
        namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
        prefix = "nsi1"
    )]
    pub struct GetContactsRecentActivityResponse {
        #[yaserde(rename = "GetContactsRecentActivityResult", default)]
        pub get_contacts_recent_activity_result: GetContactsRecentActivityResultType,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "Activities")]
    pub struct Activities {
        #[yaserde(rename = "ActivityDetails", default)]
        pub activity_details: Vec<ActivityDetailsType>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "Templates")]
    pub struct Templates {
        #[yaserde(rename = "RecentActivityTemplateContainer", default)]
        pub recent_activity_template_container: Vec<RecentActivityTemplateContainerType>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "GetBatchRecentActivityResultType")]
    pub struct GetBatchRecentActivityResultType {
        #[yaserde(rename = "Activities", default)]
        pub activities: Activities,
        #[yaserde(rename = "Templates", default)]
        pub templates: Templates,
        #[yaserde(rename = "FeedUrl", default)]
        pub feed_url: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "GetContactsRecentActivityResultType")]
    pub struct GetContactsRecentActivityResultType {
        #[yaserde(rename = "Activities", default)]
        pub activities: Activities,
        #[yaserde(rename = "Templates", default)]
        pub templates: Templates,
        #[yaserde(rename = "FeedUrl", default)]
        pub feed_url: String,
    }
    pub type ManageWLConnection = ManageWLConnectionRequestType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "ManageWLConnectionRequestType")]
    pub struct ManageWLConnectionRequestType {
        #[yaserde(rename = "abHandle", default)]
        pub ab_handle: AbHandleType,
        #[yaserde(rename = "contactId", default)]
        pub contact_id: String,
        #[yaserde(rename = "connection", default)]
        pub connection: bool,
        #[yaserde(rename = "presence", default)]
        pub presence: bool,
        #[yaserde(rename = "action", default)]
        pub action: i32,
        #[yaserde(rename = "relationshipType", default)]
        pub relationship_type: i32,
        #[yaserde(rename = "relationshipRole", default)]
        pub relationship_role: i32,
        #[yaserde(rename = "annotations", default)]
        pub annotations: Option<ArrayOfAnnotation>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "ManageWLConnectionResponse",
        namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
        namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
        prefix = "nsi1"
    )]
    pub struct ManageWLConnectionResponse {
        #[yaserde(rename = "ManageWLConnectionResult", default)]
        pub manage_wl_connection_result: ContactType,
    }
    pub type CreateContact = CreateContactType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "CreateContactType")]
    pub struct CreateContactType {
        #[yaserde(rename = "abHandle", default)]
        pub ab_handle: AbHandleType,
        #[yaserde(rename = "contactHandle", default)]
        pub contact_handle: ContactHandleType,
        #[yaserde(rename = "contactInfo", default)]
        pub contact_info: Option<ContactInfoType>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "CreateContactResponse",
        namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
        namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
        prefix = "nsi1"
    )]
    pub struct CreateContactResponse {
        #[yaserde(rename = "CreateContactResult", default)]
        pub create_contact_result: ContactType,
    }
    pub type CreateCircle = CreateCircleRequestType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "CreateCircleRequestType")]
    pub struct CreateCircleRequestType {
        #[yaserde(rename = "properties", default)]
        pub properties: ContentInfoType,
        #[yaserde(rename = "callerInfo", default)]
        pub caller_info: CallerInfoType,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "CreateCircleResponseType")]
    pub struct CreateCircleResponseType {
        #[yaserde(rename = "Id", default)]
        pub id: Guid,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
        rename = "CreateCircleResponse",
        namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
        namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
        prefix = "nsi1"
    )]
    pub struct CreateCircleResponse {
        #[yaserde(rename = "CreateCircleResult", default)]
        pub create_circle_result: CreateCircleResponseType,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "BreakConnectionRequestType")]
    pub struct BreakConnectionRequestType {
        #[yaserde(rename = "abHandle", default)]
        pub ab_handle: AbHandleType,
        #[yaserde(rename = "contactId", default)]
        pub contact_id: String,
        #[yaserde(rename = "deleteContact", default)]
        pub delete_contact: bool,
        #[yaserde(rename = "blockContact", default)]
        pub block_contact: bool,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "BreakConnectionResponseType")]
    pub struct BreakConnectionResponseType {}
    pub type BreakConnection = BreakConnectionRequestType;

    pub type BreakConnectionResponse = BreakConnectionResponseType;

    pub type AddDynamicItem = AddDynamicItemRequestType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "AddDynamicItemRequestType")]
    pub struct AddDynamicItemRequestType {
        #[yaserde(rename = "abId", default)]
        pub ab_id: String,
        #[yaserde(rename = "dynamicItems", default)]
        pub dynamic_items: DynamicItems,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "AddDynamicItemResponseType")]
    pub struct AddDynamicItemResponseType {}
    pub type AddDynamicItemResponse = AddDynamicItemResponseType;

    pub type AddService = AddServiceRequestType;

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "AddServiceRequestType")]
    pub struct AddServiceRequestType {
        #[yaserde(rename = "serviceInfo", default)]
        pub service_info: InfoType,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(rename = "AddServiceResponseType")]
    pub struct AddServiceResponseType {
        #[yaserde(rename = "AddServiceResult", default)]
        pub add_service_result: i64,
    }
    pub type AddServiceResponse = AddServiceResponseType;
}

pub mod ports {
    use super::*;
    use async_trait::async_trait;
    use yaserde::de::from_str;
    use yaserde::ser::to_string;
    use yaserde::{YaDeserialize, YaSerialize};
    pub type FindMembershipMessage = messages::FindMembershipMessage;

    pub type FindMembershipResponseMessage = messages::FindMembershipResponseMessage;

    pub type FindMembershipByRoleMessage = messages::FindMembershipByRoleMessage;

    pub type FindMembershipByRoleResponseMessage = messages::FindMembershipByRoleResponseMessage;

    pub type AddMemberMessage = messages::AddMemberMessage;

    pub type AddMemberResponseMessage = messages::AddMemberResponseMessage;

    pub type DeleteMemberMessage = messages::DeleteMemberMessage;

    pub type DeleteMemberResponseMessage = messages::DeleteMemberResponseMessage;

    pub type CreateCircleMessage = messages::CreateCircleMessage;

    pub type CreateCircleResponseMessage = messages::CreateCircleResponseMessage;

    pub type AddServiceMessage = messages::AddServiceMessage;

    pub type AddServiceResponseMessage = messages::AddServiceResponseMessage;

    #[async_trait]
    pub trait SharingServicePortType {
        async fn find_membership(
            &self,
            find_membership_message: FindMembershipMessage,
        ) -> Result<FindMembershipResponseMessage, Option<SoapFault>>;
        async fn find_membership_by_role(
            &self,
            find_membership_by_role_message: FindMembershipByRoleMessage,
        ) -> Result<FindMembershipByRoleResponseMessage, Option<SoapFault>>;
        async fn add_member(
            &self,
            add_member_message: AddMemberMessage,
        ) -> Result<AddMemberResponseMessage, Option<SoapFault>>;
        async fn delete_member(
            &self,
            delete_member_message: DeleteMemberMessage,
        ) -> Result<DeleteMemberResponseMessage, Option<SoapFault>>;
        async fn create_circle(
            &self,
            create_circle_message: CreateCircleMessage,
        ) -> Result<CreateCircleResponseMessage, Option<SoapFault>>;
        async fn add_service(
            &self,
            add_service_message: AddServiceMessage,
        ) -> Result<AddServiceResponseMessage, Option<SoapFault>>;
    }
    pub type AbfindAllMessage = messages::AbfindAllMessage;

    pub type AbfindAllResponseMessage = messages::AbfindAllResponseMessage;

    pub type AbcontactAddMessage = messages::AbcontactAddMessage;

    pub type AbcontactAddResponseMessage = messages::AbcontactAddResponseMessage;

    pub type AbcontactDeleteMessage = messages::AbcontactDeleteMessage;

    pub type AbcontactDeleteResponseMessage = messages::AbcontactDeleteResponseMessage;

    pub type DeleteContactMessage = messages::DeleteContactMessage;

    pub type DeleteContactResponseMessage = messages::DeleteContactResponseMessage;

    pub type AbgroupContactAddMessage = messages::AbgroupContactAddMessage;

    pub type AbgroupContactAddResponseMessage = messages::AbgroupContactAddResponseMessage;

    pub type AbgroupAddMessage = messages::AbgroupAddMessage;

    pub type AbgroupAddResponseMessage = messages::AbgroupAddResponseMessage;

    pub type AbgroupUpdateMessage = messages::AbgroupUpdateMessage;

    pub type AbgroupUpdateResponseMessage = messages::AbgroupUpdateResponseMessage;

    pub type AbgroupDeleteMessage = messages::AbgroupDeleteMessage;

    pub type AbgroupDeleteResponseMessage = messages::AbgroupDeleteResponseMessage;

    pub type AbgroupContactDeleteMessage = messages::AbgroupContactDeleteMessage;

    pub type AbgroupContactDeleteResponseMessage = messages::AbgroupContactDeleteResponseMessage;

    pub type AbcontactUpdateMessage = messages::AbcontactUpdateMessage;

    pub type AbcontactUpdateResponseMessage = messages::AbcontactUpdateResponseMessage;

    pub type AbaddMessage = messages::AbaddMessage;

    pub type AbaddResponseMessage = messages::AbaddResponseMessage;

    pub type UpdateDynamicItemMessage = messages::UpdateDynamicItemMessage;

    pub type UpdateDynamicItemResponseMessage = messages::UpdateDynamicItemResponseMessage;

    pub type AbfindContactsPagedMessage = messages::AbfindContactsPagedMessage;

    pub type AbfindContactsPagedResponseMessage = types::AbfindContactsPagedResponse;

    pub type FindFriendsInCommonMessage = messages::FindFriendsInCommonMessage;

    pub type FindFriendsInCommonResponseMessage = messages::FindFriendsInCommonResponseMessage;

    pub type CreateContactMessage = messages::CreateContactMessage;

    pub type CreateContactResponseMessage = messages::CreateContactResponseMessage;

    pub type ManageWLConnectionMessage = messages::ManageWLConnectionMessage;

    pub type ManageWLConnectionResponseMessage = messages::ManageWLConnectionResponseMessage;

    pub type BreakConnectionMessage = messages::BreakConnectionMessage;

    pub type BreakConnectionResponseMessage = messages::BreakConnectionResponseMessage;

    pub type AddDynamicItemMessage = messages::AddDynamicItemMessage;

    pub type AddDynamicItemResponseMessage = messages::AddDynamicItemResponseMessage;

    pub type AbfindByContactsMessage = messages::AbfindByContactsMessage;

    pub type AbfindByContactsResponseMessage = messages::AbfindByContactsResponseMessage;

    #[async_trait]
    pub trait AbservicePortType {
        async fn ab_find_all(
            &self,
            abfind_all_message: AbfindAllMessage,
        ) -> Result<AbfindAllResponseMessage, Option<SoapFault>>;
        async fn ab_contact_add(
            &self,
            abcontact_add_message: AbcontactAddMessage,
        ) -> Result<AbcontactAddResponseMessage, Option<SoapFault>>;
        async fn ab_contact_delete(
            &self,
            abcontact_delete_message: AbcontactDeleteMessage,
        ) -> Result<AbcontactDeleteResponseMessage, Option<SoapFault>>;
        async fn delete_contact(
            &self,
            delete_contact_message: DeleteContactMessage,
        ) -> Result<DeleteContactResponseMessage, Option<SoapFault>>;
        async fn ab_group_contact_add(
            &self,
            abgroup_contact_add_message: AbgroupContactAddMessage,
        ) -> Result<AbgroupContactAddResponseMessage, Option<SoapFault>>;
        async fn ab_group_add(
            &self,
            abgroup_add_message: AbgroupAddMessage,
        ) -> Result<AbgroupAddResponseMessage, Option<SoapFault>>;
        async fn ab_group_update(
            &self,
            abgroup_update_message: AbgroupUpdateMessage,
        ) -> Result<AbgroupUpdateResponseMessage, Option<SoapFault>>;
        async fn ab_group_delete(
            &self,
            abgroup_delete_message: AbgroupDeleteMessage,
        ) -> Result<AbgroupDeleteResponseMessage, Option<SoapFault>>;
        async fn ab_group_contact_delete(
            &self,
            abgroup_contact_delete_message: AbgroupContactDeleteMessage,
        ) -> Result<AbgroupContactDeleteResponseMessage, Option<SoapFault>>;
        async fn ab_contact_update(
            &self,
            abcontact_update_message: AbcontactUpdateMessage,
        ) -> Result<AbcontactUpdateResponseMessage, Option<SoapFault>>;
        async fn ab_add(
            &self,
            abadd_message: AbaddMessage,
        ) -> Result<AbaddResponseMessage, Option<SoapFault>>;
        async fn update_dynamic_item(
            &self,
            update_dynamic_item_message: UpdateDynamicItemMessage,
        ) -> Result<UpdateDynamicItemResponseMessage, Option<SoapFault>>;
        async fn ab_find_contacts_paged(
            &self,
            abfind_contacts_paged_message: AbfindContactsPagedMessage,
        ) -> Result<AbfindContactsPagedResponseMessage, Option<SoapFault>>;
        async fn find_friends_in_common(
            &self,
            find_friends_in_common_message: FindFriendsInCommonMessage,
        ) -> Result<FindFriendsInCommonResponseMessage, Option<SoapFault>>;
        async fn create_contact(
            &self,
            create_contact_message: CreateContactMessage,
        ) -> Result<CreateContactResponseMessage, Option<SoapFault>>;
        async fn manage_wl_connection(
            &self,
            manage_wl_connection_message: ManageWLConnectionMessage,
        ) -> Result<ManageWLConnectionResponseMessage, Option<SoapFault>>;
        async fn break_connection(
            &self,
            break_connection_message: BreakConnectionMessage,
        ) -> Result<BreakConnectionResponseMessage, Option<SoapFault>>;
        async fn add_dynamic_item(
            &self,
            add_dynamic_item_message: AddDynamicItemMessage,
        ) -> Result<AddDynamicItemResponseMessage, Option<SoapFault>>;
        async fn ab_find_by_contacts(
            &self,
            abfind_by_contacts_message: AbfindByContactsMessage,
        ) -> Result<AbfindByContactsResponseMessage, Option<SoapFault>>;
    }
    pub type GetContactsRecentActivityMessage = messages::GetContactsRecentActivityMessage;

    pub type GetContactsRecentActivityResponseMessage =
        messages::GetContactsRecentActivityResponseMessage;

    pub type GetBatchRecentActivityMessage = messages::GetBatchRecentActivityMessage;

    pub type GetBatchRecentActivityResponseMessage =
        messages::GetBatchRecentActivityResponseMessage;

    #[async_trait]
    pub trait WhatsUpServicePortType {
        async fn get_contacts_recent_activity(
            &self,
            get_contacts_recent_activity_message: GetContactsRecentActivityMessage,
        ) -> Result<GetContactsRecentActivityResponseMessage, Option<SoapFault>>;
        async fn get_batch_recent_activity(
            &self,
            get_batch_recent_activity_message: GetBatchRecentActivityMessage,
        ) -> Result<GetBatchRecentActivityResponseMessage, Option<SoapFault>>;
    }
}

pub mod bindings {
    use super::*;
    use super::messages::{ServiceHeaderContainer, RequestHeaderContainer};
    use async_trait::async_trait;
    use yaserde::de::from_str;
    use yaserde::ser::to_string;
    use yaserde::{YaDeserialize, YaSerialize};

    impl SharingServiceBinding {
        async fn send_soap_request<T: YaSerialize>(
            &self,
            request: &T,
            action: &str,
        ) -> SoapResponse {
            let body = to_string(request).expect("failed to generate xml");
            debug!("SOAP Request: {}", body);
            let mut req = self
                .client
                .post(&self.url)
                .body(body)
                .header("Content-Type", "text/xml")
                .header("Soapaction", action);
            if let Some(credentials) = &self.credentials {
                req = req.basic_auth(
                    credentials.0.to_string(),
                    Option::Some(credentials.1.to_string()),
                );
            }
            let res = req.send().await?;
            let status = res.status();
            debug!("SOAP Status: {}", status);
            let txt = res.text().await.unwrap_or_default();
            debug!("SOAP Response: {}", txt);
            Ok((status, txt))
        }
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapFindMembershipMessage {
        #[yaserde(rename = "FindMembership", prefix="soap")]
        pub body: ports::FindMembershipMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct FindMembershipMessageSoapEnvelope {
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
        pub body: SoapFindMembershipMessage,
    }

    impl FindMembershipMessageSoapEnvelope {
        pub fn new(body: SoapFindMembershipMessage) -> Self {
            FindMembershipMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapFindMembershipResponseMessage {
        #[yaserde(rename = "FindMembershipResponse", default)]
        pub body: ports::FindMembershipResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
        namespace = "xsd: http://www.w3.org/2001/XMLSchema",
        namespace = "soapenc: http://schemas.xmlsoap.org/soap/encoding/",
        prefix = "soap"
    )]
    pub struct FindMembershipResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapFindMembershipResponseMessage,
    }

    impl FindMembershipResponseMessageSoapEnvelope {
        pub fn new(body: SoapFindMembershipResponseMessage) -> Self {
            FindMembershipResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapFindMembershipByRoleMessage {
        #[yaserde(rename = "FindMembershipByRole", default)]
        pub body: ports::FindMembershipByRoleMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct FindMembershipByRoleMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapFindMembershipByRoleMessage,
    }

    impl FindMembershipByRoleMessageSoapEnvelope {
        pub fn new(body: SoapFindMembershipByRoleMessage) -> Self {
            FindMembershipByRoleMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapFindMembershipByRoleResponseMessage {
        #[yaserde(rename = "FindMembershipByRoleResponseMessage", default)]
        pub body: ports::FindMembershipByRoleResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct FindMembershipByRoleResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapFindMembershipByRoleResponseMessage,
    }

    impl FindMembershipByRoleResponseMessageSoapEnvelope {
        pub fn new(body: SoapFindMembershipByRoleResponseMessage) -> Self {
            FindMembershipByRoleResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAddMemberMessage {
        #[yaserde(rename = "AddMember", default)]
        pub body: ports::AddMemberMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AddMemberMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAddMemberMessage,
    }

    impl AddMemberMessageSoapEnvelope {
        pub fn new(body: SoapAddMemberMessage) -> Self {
            AddMemberMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAddMemberResponseMessage {
        #[yaserde(rename = "AddMemberResponseMessage", default)]
        pub body: ports::AddMemberResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AddMemberResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAddMemberResponseMessage,
    }

    impl AddMemberResponseMessageSoapEnvelope {
        pub fn new(body: SoapAddMemberResponseMessage) -> Self {
            AddMemberResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapDeleteMemberMessage {
        #[yaserde(rename = "DeleteMember", default)]
        pub body: ports::DeleteMemberMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct DeleteMemberMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapDeleteMemberMessage,
    }

    impl DeleteMemberMessageSoapEnvelope {
        pub fn new(body: SoapDeleteMemberMessage) -> Self {
            DeleteMemberMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapDeleteMemberResponseMessage {
        #[yaserde(rename = "DeleteMemberResponseMessage", default)]
        pub body: ports::DeleteMemberResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct DeleteMemberResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapDeleteMemberResponseMessage,
    }

    impl DeleteMemberResponseMessageSoapEnvelope {
        pub fn new(body: SoapDeleteMemberResponseMessage) -> Self {
            DeleteMemberResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapCreateCircleMessage {
        #[yaserde(rename = "CreateCircle", default)]
        pub body: ports::CreateCircleMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct CreateCircleMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapCreateCircleMessage,
    }

    impl CreateCircleMessageSoapEnvelope {
        pub fn new(body: SoapCreateCircleMessage) -> Self {
            CreateCircleMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapCreateCircleResponseMessage {
        #[yaserde(rename = "CreateCircleResponseMessage", default)]
        pub body: ports::CreateCircleResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct CreateCircleResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapCreateCircleResponseMessage,
    }

    impl CreateCircleResponseMessageSoapEnvelope {
        pub fn new(body: SoapCreateCircleResponseMessage) -> Self {
            CreateCircleResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAddServiceMessage {
        #[yaserde(rename = "AddService", default)]
        pub body: ports::AddServiceMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AddServiceMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAddServiceMessage,
    }

    impl AddServiceMessageSoapEnvelope {
        pub fn new(body: SoapAddServiceMessage) -> Self {
            AddServiceMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAddServiceResponseMessage {
        #[yaserde(rename = "AddServiceResponseMessage", default)]
        pub body: ports::AddServiceResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AddServiceResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAddServiceResponseMessage,
    }

    impl AddServiceResponseMessageSoapEnvelope {
        pub fn new(body: SoapAddServiceResponseMessage) -> Self {
            AddServiceResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    impl Default for SharingServiceBinding {
        fn default() -> Self {
            SharingServiceBinding {
                client: reqwest::Client::new(),
                url: "http://www.msn.com/webservices/AddressBook".to_string(),
                credentials: Option::None,
            }
        }
    }
    impl SharingServiceBinding {
        pub fn new(url: &str, credentials: Option<(String, String)>) -> Self {
            SharingServiceBinding {
                client: reqwest::Client::new(),
                url: url.to_string(),
                credentials,
            }
        }
    }
    pub struct SharingServiceBinding {
        client: reqwest::Client,
        url: String,
        credentials: Option<(String, String)>,
    }
    #[async_trait]
    impl ports::SharingServicePortType for SharingServiceBinding {
        async fn find_membership(
            &self,
            find_membership_message: ports::FindMembershipMessage,
        ) -> Result<ports::FindMembershipResponseMessage, Option<SoapFault>> {
            let __request = FindMembershipMessageSoapEnvelope::new(SoapFindMembershipMessage {
                body: find_membership_message,
                xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
            });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/FindMembership",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: FindMembershipResponseMessageSoapEnvelope =
                from_str(&response).map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn find_membership_by_role(
            &self,
            find_membership_by_role_message: ports::FindMembershipByRoleMessage,
        ) -> Result<ports::FindMembershipByRoleResponseMessage, Option<SoapFault>> {
            let __request =
                FindMembershipByRoleMessageSoapEnvelope::new(SoapFindMembershipByRoleMessage {
                    body: find_membership_by_role_message,
                    xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/FindMembershipByRole",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: FindMembershipByRoleResponseMessageSoapEnvelope =
                from_str(&response).map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn add_member(
            &self,
            add_member_message: ports::AddMemberMessage,
        ) -> Result<ports::AddMemberResponseMessage, Option<SoapFault>> {
            let __request = AddMemberMessageSoapEnvelope::new(SoapAddMemberMessage {
                body: add_member_message,
                xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
            });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/AddMember",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: AddMemberResponseMessageSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn delete_member(
            &self,
            delete_member_message: ports::DeleteMemberMessage,
        ) -> Result<ports::DeleteMemberResponseMessage, Option<SoapFault>> {
            let __request = DeleteMemberMessageSoapEnvelope::new(SoapDeleteMemberMessage {
                body: delete_member_message,
                xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
            });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/DeleteMember",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: DeleteMemberResponseMessageSoapEnvelope =
                from_str(&response).map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn create_circle(
            &self,
            create_circle_message: ports::CreateCircleMessage,
        ) -> Result<ports::CreateCircleResponseMessage, Option<SoapFault>> {
            let __request = CreateCircleMessageSoapEnvelope::new(SoapCreateCircleMessage {
                body: create_circle_message,
                xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
            });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/CreateCircle",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: CreateCircleResponseMessageSoapEnvelope =
                from_str(&response).map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn add_service(
            &self,
            add_service_message: ports::AddServiceMessage,
        ) -> Result<ports::AddServiceResponseMessage, Option<SoapFault>> {
            let __request = AddServiceMessageSoapEnvelope::new(SoapAddServiceMessage {
                body: add_service_message,
                xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
            });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/AddService",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: AddServiceResponseMessageSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
    }

    impl AbserviceBinding {
        async fn send_soap_request<T: YaSerialize>(
            &self,
            request: &T,
            action: &str,
        ) -> SoapResponse {
            let body = to_string(request).expect("failed to generate xml");
            debug!("SOAP Request: {}", body);
            let mut req = self
                .client
                .post(&self.url)
                .body(body)
                .header("Content-Type", "text/xml")
                .header("Soapaction", action);
            if let Some(credentials) = &self.credentials {
                req = req.basic_auth(
                    credentials.0.to_string(),
                    Option::Some(credentials.1.to_string()),
                );
            }
            let res = req.send().await?;
            let status = res.status();
            debug!("SOAP Status: {}", status);
            let txt = res.text().await.unwrap_or_default();
            debug!("SOAP Response: {}", txt);
            Ok((status, txt))
        }
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbfindAllMessage {
        #[yaserde(rename = "ABFindAll", default)]
        pub body: ports::AbfindAllMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AbfindAllMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbfindAllMessage,
    }

    impl AbfindAllMessageSoapEnvelope {
        pub fn new(body: SoapAbfindAllMessage) -> Self {
            AbfindAllMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbfindAllResponseMessage {
        #[yaserde(rename = "AbfindAllResponseMessage", default)]
        pub body: ports::AbfindAllResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AbfindAllResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbfindAllResponseMessage,
    }

    impl AbfindAllResponseMessageSoapEnvelope {
        pub fn new(body: SoapAbfindAllResponseMessage) -> Self {
            AbfindAllResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbcontactAddMessage {
        #[yaserde(rename = "ABContactAdd", default)]
        pub body: ports::AbcontactAddMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
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
        pub header: Option<Header>,
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

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbcontactAddResponseMessage {
        #[yaserde(rename = "AbcontactAddResponseMessage", default)]
        pub body: ports::AbcontactAddResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AbcontactAddResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbcontactAddResponseMessage,
    }

    impl AbcontactAddResponseMessageSoapEnvelope {
        pub fn new(body: SoapAbcontactAddResponseMessage) -> Self {
            AbcontactAddResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbcontactDeleteMessage {
        #[yaserde(rename = "ABContactDelete", default)]
        pub body: ports::AbcontactDeleteMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AbcontactDeleteMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbcontactDeleteMessage,
    }

    impl AbcontactDeleteMessageSoapEnvelope {
        pub fn new(body: SoapAbcontactDeleteMessage) -> Self {
            AbcontactDeleteMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbcontactDeleteResponseMessage {
        #[yaserde(rename = "AbcontactDeleteResponseMessage", default)]
        pub body: ports::AbcontactDeleteResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AbcontactDeleteResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbcontactDeleteResponseMessage,
    }

    impl AbcontactDeleteResponseMessageSoapEnvelope {
        pub fn new(body: SoapAbcontactDeleteResponseMessage) -> Self {
            AbcontactDeleteResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapDeleteContactMessage {
        #[yaserde(rename = "DeleteContact", default)]
        pub body: ports::DeleteContactMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct DeleteContactMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapDeleteContactMessage,
    }

    impl DeleteContactMessageSoapEnvelope {
        pub fn new(body: SoapDeleteContactMessage) -> Self {
            DeleteContactMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapDeleteContactResponseMessage {
        #[yaserde(rename = "DeleteContactResponseMessage", default)]
        pub body: ports::DeleteContactResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct DeleteContactResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapDeleteContactResponseMessage,
    }

    impl DeleteContactResponseMessageSoapEnvelope {
        pub fn new(body: SoapDeleteContactResponseMessage) -> Self {
            DeleteContactResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbgroupContactAddMessage {
        #[yaserde(rename = "ABGroupContactAdd", default)]
        pub body: ports::AbgroupContactAddMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AbgroupContactAddMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbgroupContactAddMessage,
    }

    impl AbgroupContactAddMessageSoapEnvelope {
        pub fn new(body: SoapAbgroupContactAddMessage) -> Self {
            AbgroupContactAddMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbgroupContactAddResponseMessage {
        #[yaserde(rename = "AbgroupContactAddResponseMessage", default)]
        pub body: ports::AbgroupContactAddResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AbgroupContactAddResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbgroupContactAddResponseMessage,
    }

    impl AbgroupContactAddResponseMessageSoapEnvelope {
        pub fn new(body: SoapAbgroupContactAddResponseMessage) -> Self {
            AbgroupContactAddResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
        prefix = "nsi1",
        default_namespace="nsi1"
    )]
    pub struct SoapAbgroupAddMessage {
        #[yaserde(rename = "ABGroupAdd", prefix="nsi1")]
        pub body: ports::AbgroupAddMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>
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

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbgroupAddResponseMessage {
        #[yaserde(rename = "ABGroupAddResponse", default)]
        pub body: ports::AbgroupAddResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
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
        pub fn new(body: SoapAbgroupAddResponseMessage) -> Self {
            AbgroupAddResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbgroupUpdateMessage {
        #[yaserde(rename = "ABGroupUpdate", default)]
        pub body: ports::AbgroupUpdateMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AbgroupUpdateMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbgroupUpdateMessage,
    }

    impl AbgroupUpdateMessageSoapEnvelope {
        pub fn new(body: SoapAbgroupUpdateMessage) -> Self {
            AbgroupUpdateMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbgroupUpdateResponseMessage {
        #[yaserde(rename = "AbgroupUpdateResponseMessage", default)]
        pub body: ports::AbgroupUpdateResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AbgroupUpdateResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbgroupUpdateResponseMessage,
    }

    impl AbgroupUpdateResponseMessageSoapEnvelope {
        pub fn new(body: SoapAbgroupUpdateResponseMessage) -> Self {
            AbgroupUpdateResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbgroupDeleteMessage {
        #[yaserde(rename = "ABGroupDelete", default)]
        pub body: ports::AbgroupDeleteMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AbgroupDeleteMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbgroupDeleteMessage,
    }

    impl AbgroupDeleteMessageSoapEnvelope {
        pub fn new(body: SoapAbgroupDeleteMessage) -> Self {
            AbgroupDeleteMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbgroupDeleteResponseMessage {
        #[yaserde(rename = "AbgroupDeleteResponseMessage", default)]
        pub body: ports::AbgroupDeleteResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AbgroupDeleteResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbgroupDeleteResponseMessage,
    }

    impl AbgroupDeleteResponseMessageSoapEnvelope {
        pub fn new(body: SoapAbgroupDeleteResponseMessage) -> Self {
            AbgroupDeleteResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbgroupContactDeleteMessage {
        #[yaserde(rename = "ABGroupContactDelete", default)]
        pub body: ports::AbgroupContactDeleteMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AbgroupContactDeleteMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbgroupContactDeleteMessage,
    }

    impl AbgroupContactDeleteMessageSoapEnvelope {
        pub fn new(body: SoapAbgroupContactDeleteMessage) -> Self {
            AbgroupContactDeleteMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbgroupContactDeleteResponseMessage {
        #[yaserde(rename = "AbgroupContactDeleteResponseMessage", default)]
        pub body: ports::AbgroupContactDeleteResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AbgroupContactDeleteResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbgroupContactDeleteResponseMessage,
    }

    impl AbgroupContactDeleteResponseMessageSoapEnvelope {
        pub fn new(body: SoapAbgroupContactDeleteResponseMessage) -> Self {
            AbgroupContactDeleteResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbcontactUpdateMessage {
        #[yaserde(rename = "ABContactUpdate", default)]
        pub body: ports::AbcontactUpdateMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AbcontactUpdateMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbcontactUpdateMessage,
    }

    impl AbcontactUpdateMessageSoapEnvelope {
        pub fn new(body: SoapAbcontactUpdateMessage) -> Self {
            AbcontactUpdateMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbcontactUpdateResponseMessage {
        #[yaserde(rename = "AbcontactUpdateResponseMessage", default)]
        pub body: ports::AbcontactUpdateResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AbcontactUpdateResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbcontactUpdateResponseMessage,
    }

    impl AbcontactUpdateResponseMessageSoapEnvelope {
        pub fn new(body: SoapAbcontactUpdateResponseMessage) -> Self {
            AbcontactUpdateResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbaddMessage {
        #[yaserde(rename = "ABAdd", default)]
        pub body: ports::AbaddMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AbaddMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbaddMessage,
    }

    impl AbaddMessageSoapEnvelope {
        pub fn new(body: SoapAbaddMessage) -> Self {
            AbaddMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbaddResponseMessage {
        #[yaserde(rename = "AbaddResponseMessage", default)]
        pub body: ports::AbaddResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AbaddResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbaddResponseMessage,
    }

    impl AbaddResponseMessageSoapEnvelope {
        pub fn new(body: SoapAbaddResponseMessage) -> Self {
            AbaddResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapUpdateDynamicItemMessage {
        #[yaserde(rename = "UpdateDynamicItem", default)]
        pub body: ports::UpdateDynamicItemMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct UpdateDynamicItemMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapUpdateDynamicItemMessage,
    }

    impl UpdateDynamicItemMessageSoapEnvelope {
        pub fn new(body: SoapUpdateDynamicItemMessage) -> Self {
            UpdateDynamicItemMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapUpdateDynamicItemResponseMessage {
        #[yaserde(rename = "UpdateDynamicItemResponse", default)]
        pub body: ports::UpdateDynamicItemResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
        namespace = "xsd: http://www.w3.org/2001/XMLSchema",
        prefix = "soap"
    )]
    pub struct UpdateDynamicItemResponseMessageSoapEnvelope {
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapUpdateDynamicItemResponseMessage,
    }

    impl UpdateDynamicItemResponseMessageSoapEnvelope {
        pub fn new(body: SoapUpdateDynamicItemResponseMessage) -> Self {
            UpdateDynamicItemResponseMessageSoapEnvelope {
                body,
                header: None
            }
        }
    }



    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap",
        default_namespace="soap"
    )]
    pub struct SoapAbfindContactsPagedMessage {
        #[yaserde(rename = "ABFindContactsPaged", default)]
        pub body: ports::AbfindContactsPagedMessage,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AbfindContactsPagedMessageSoapEnvelope {
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
        pub body: SoapAbfindContactsPagedMessage,
    }

    impl AbfindContactsPagedMessageSoapEnvelope {
        pub fn new(body: SoapAbfindContactsPagedMessage) -> Self {
            AbfindContactsPagedMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbfindContactsPagedResponseMessage {
        #[yaserde(rename = "ABFindContactsPagedResponse", default)]
        pub body: ports::AbfindContactsPagedResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
        namespace = "xsd: http://www.w3.org/2001/XMLSchema",
        prefix = "soap"
    )]
    pub struct AbfindContactsPagedResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<ServiceHeaderContainer>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbfindContactsPagedResponseMessage,
    }

    impl AbfindContactsPagedResponseMessageSoapEnvelope {
        pub fn new(body: SoapAbfindContactsPagedResponseMessage) -> Self {
            AbfindContactsPagedResponseMessageSoapEnvelope {
                body,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapFindFriendsInCommonMessage {
        #[yaserde(rename = "FindFriendsInCommon", default)]
        pub body: ports::FindFriendsInCommonMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct FindFriendsInCommonMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapFindFriendsInCommonMessage,
    }

    impl FindFriendsInCommonMessageSoapEnvelope {
        pub fn new(body: SoapFindFriendsInCommonMessage) -> Self {
            FindFriendsInCommonMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapFindFriendsInCommonResponseMessage {
        #[yaserde(rename = "FindFriendsInCommonResponseMessage", default)]
        pub body: ports::FindFriendsInCommonResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct FindFriendsInCommonResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapFindFriendsInCommonResponseMessage,
    }

    impl FindFriendsInCommonResponseMessageSoapEnvelope {
        pub fn new(body: SoapFindFriendsInCommonResponseMessage) -> Self {
            FindFriendsInCommonResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapCreateContactMessage {
        #[yaserde(rename = "CreateContact", default)]
        pub body: ports::CreateContactMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct CreateContactMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapCreateContactMessage,
    }

    impl CreateContactMessageSoapEnvelope {
        pub fn new(body: SoapCreateContactMessage) -> Self {
            CreateContactMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapCreateContactResponseMessage {
        #[yaserde(rename = "CreateContactResponseMessage", default)]
        pub body: ports::CreateContactResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct CreateContactResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapCreateContactResponseMessage,
    }

    impl CreateContactResponseMessageSoapEnvelope {
        pub fn new(body: SoapCreateContactResponseMessage) -> Self {
            CreateContactResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapManageWLConnectionMessage {
        #[yaserde(rename = "ManageWLConnection", default)]
        pub body: ports::ManageWLConnectionMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct ManageWLConnectionMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapManageWLConnectionMessage,
    }

    impl ManageWLConnectionMessageSoapEnvelope {
        pub fn new(body: SoapManageWLConnectionMessage) -> Self {
            ManageWLConnectionMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapManageWLConnectionResponseMessage {
        #[yaserde(rename = "ManageWLConnectionResponseMessage", default)]
        pub body: ports::ManageWLConnectionResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct ManageWLConnectionResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapManageWLConnectionResponseMessage,
    }

    impl ManageWLConnectionResponseMessageSoapEnvelope {
        pub fn new(body: SoapManageWLConnectionResponseMessage) -> Self {
            ManageWLConnectionResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapBreakConnectionMessage {
        #[yaserde(rename = "BreakConnection", default)]
        pub body: ports::BreakConnectionMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct BreakConnectionMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapBreakConnectionMessage,
    }

    impl BreakConnectionMessageSoapEnvelope {
        pub fn new(body: SoapBreakConnectionMessage) -> Self {
            BreakConnectionMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapBreakConnectionResponseMessage {
        #[yaserde(rename = "BreakConnectionResponseMessage", default)]
        pub body: ports::BreakConnectionResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct BreakConnectionResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapBreakConnectionResponseMessage,
    }

    impl BreakConnectionResponseMessageSoapEnvelope {
        pub fn new(body: SoapBreakConnectionResponseMessage) -> Self {
            BreakConnectionResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAddDynamicItemMessage {
        #[yaserde(rename = "AddDynamicItem", default)]
        pub body: ports::AddDynamicItemMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AddDynamicItemMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAddDynamicItemMessage,
    }

    impl AddDynamicItemMessageSoapEnvelope {
        pub fn new(body: SoapAddDynamicItemMessage) -> Self {
            AddDynamicItemMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAddDynamicItemResponseMessage {
        #[yaserde(rename = "AddDynamicItemResponseMessage", default)]
        pub body: ports::AddDynamicItemResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AddDynamicItemResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAddDynamicItemResponseMessage,
    }

    impl AddDynamicItemResponseMessageSoapEnvelope {
        pub fn new(body: SoapAddDynamicItemResponseMessage) -> Self {
            AddDynamicItemResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbfindByContactsMessage {
        #[yaserde(rename = "ABFindByContacts", default)]
        pub body: ports::AbfindByContactsMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AbfindByContactsMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbfindByContactsMessage,
    }

    impl AbfindByContactsMessageSoapEnvelope {
        pub fn new(body: SoapAbfindByContactsMessage) -> Self {
            AbfindByContactsMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapAbfindByContactsResponseMessage {
        #[yaserde(rename = "AbfindByContactsResponseMessage", default)]
        pub body: ports::AbfindByContactsResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct AbfindByContactsResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapAbfindByContactsResponseMessage,
    }

    impl AbfindByContactsResponseMessageSoapEnvelope {
        pub fn new(body: SoapAbfindByContactsResponseMessage) -> Self {
            AbfindByContactsResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    impl Default for AbserviceBinding {
        fn default() -> Self {
            AbserviceBinding {
                client: reqwest::Client::new(),
                url: "http://www.msn.com/webservices/AddressBook".to_string(),
                credentials: Option::None,
            }
        }
    }
    impl AbserviceBinding {
        pub fn new(url: &str, credentials: Option<(String, String)>) -> Self {
            AbserviceBinding {
                client: reqwest::Client::new(),
                url: url.to_string(),
                credentials,
            }
        }
    }
    pub struct AbserviceBinding {
        client: reqwest::Client,
        url: String,
        credentials: Option<(String, String)>,
    }
    #[async_trait]
    impl ports::AbservicePortType for AbserviceBinding {
        async fn ab_find_all(
            &self,
            abfind_all_message: ports::AbfindAllMessage,
        ) -> Result<ports::AbfindAllResponseMessage, Option<SoapFault>> {
            let __request = AbfindAllMessageSoapEnvelope::new(SoapAbfindAllMessage {
                body: abfind_all_message,
                xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
            });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/ABFindAll",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: AbfindAllResponseMessageSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn ab_contact_add(
            &self,
            abcontact_add_message: ports::AbcontactAddMessage,
        ) -> Result<ports::AbcontactAddResponseMessage, Option<SoapFault>> {
            let __request = AbcontactAddMessageSoapEnvelope::new(SoapAbcontactAddMessage {
                body: abcontact_add_message,
                xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
            });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/ABContactAdd",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: AbcontactAddResponseMessageSoapEnvelope =
                from_str(&response).map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn ab_contact_delete(
            &self,
            abcontact_delete_message: ports::AbcontactDeleteMessage,
        ) -> Result<ports::AbcontactDeleteResponseMessage, Option<SoapFault>> {
            let __request = AbcontactDeleteMessageSoapEnvelope::new(SoapAbcontactDeleteMessage {
                body: abcontact_delete_message,
                xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
            });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/ABContactDelete",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: AbcontactDeleteResponseMessageSoapEnvelope =
                from_str(&response).map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn delete_contact(
            &self,
            delete_contact_message: ports::DeleteContactMessage,
        ) -> Result<ports::DeleteContactResponseMessage, Option<SoapFault>> {
            let __request = DeleteContactMessageSoapEnvelope::new(SoapDeleteContactMessage {
                body: delete_contact_message,
                xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
            });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/DeleteContact",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: DeleteContactResponseMessageSoapEnvelope =
                from_str(&response).map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn ab_group_contact_add(
            &self,
            abgroup_contact_add_message: ports::AbgroupContactAddMessage,
        ) -> Result<ports::AbgroupContactAddResponseMessage, Option<SoapFault>> {
            let __request =
                AbgroupContactAddMessageSoapEnvelope::new(SoapAbgroupContactAddMessage {
                    body: abgroup_contact_add_message,
                    xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/ABGroupContactAdd",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: AbgroupContactAddResponseMessageSoapEnvelope =
                from_str(&response).map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn ab_group_add(
            &self,
            abgroup_add_message: ports::AbgroupAddMessage,
        ) -> Result<ports::AbgroupAddResponseMessage, Option<SoapFault>> {
            let __request = AbgroupAddMessageSoapEnvelope::new(SoapAbgroupAddMessage {
                body: abgroup_add_message,
                xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
            });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/ABGroupAdd",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: AbgroupAddResponseMessageSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn ab_group_update(
            &self,
            abgroup_update_message: ports::AbgroupUpdateMessage,
        ) -> Result<ports::AbgroupUpdateResponseMessage, Option<SoapFault>> {
            let __request = AbgroupUpdateMessageSoapEnvelope::new(SoapAbgroupUpdateMessage {
                body: abgroup_update_message,
                xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
            });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/ABGroupUpdate",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: AbgroupUpdateResponseMessageSoapEnvelope =
                from_str(&response).map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn ab_group_delete(
            &self,
            abgroup_delete_message: ports::AbgroupDeleteMessage,
        ) -> Result<ports::AbgroupDeleteResponseMessage, Option<SoapFault>> {
            let __request = AbgroupDeleteMessageSoapEnvelope::new(SoapAbgroupDeleteMessage {
                body: abgroup_delete_message,
                xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
            });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/ABGroupDelete",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: AbgroupDeleteResponseMessageSoapEnvelope =
                from_str(&response).map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn ab_group_contact_delete(
            &self,
            abgroup_contact_delete_message: ports::AbgroupContactDeleteMessage,
        ) -> Result<ports::AbgroupContactDeleteResponseMessage, Option<SoapFault>> {
            let __request =
                AbgroupContactDeleteMessageSoapEnvelope::new(SoapAbgroupContactDeleteMessage {
                    body: abgroup_contact_delete_message,
                    xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/ABGroupContactDelete",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: AbgroupContactDeleteResponseMessageSoapEnvelope =
                from_str(&response).map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn ab_contact_update(
            &self,
            abcontact_update_message: ports::AbcontactUpdateMessage,
        ) -> Result<ports::AbcontactUpdateResponseMessage, Option<SoapFault>> {
            let __request = AbcontactUpdateMessageSoapEnvelope::new(SoapAbcontactUpdateMessage {
                body: abcontact_update_message,
                xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
            });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/ABContactUpdate",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: AbcontactUpdateResponseMessageSoapEnvelope =
                from_str(&response).map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn ab_add(
            &self,
            abadd_message: ports::AbaddMessage,
        ) -> Result<ports::AbaddResponseMessage, Option<SoapFault>> {
            let __request = AbaddMessageSoapEnvelope::new(SoapAbaddMessage {
                body: abadd_message,
                xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
            });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/ABAdd",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: AbaddResponseMessageSoapEnvelope = from_str(&response).map_err(|err| {
                warn!("Failed to unmarshal SOAP response: {:?}", err);
                None
            })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn update_dynamic_item(
            &self,
            update_dynamic_item_message: ports::UpdateDynamicItemMessage,
        ) -> Result<ports::UpdateDynamicItemResponseMessage, Option<SoapFault>> {
            let __request =
                UpdateDynamicItemMessageSoapEnvelope::new(SoapUpdateDynamicItemMessage {
                    body: update_dynamic_item_message,
                    xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/UpdateDynamicItem",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: UpdateDynamicItemResponseMessageSoapEnvelope =
                from_str(&response).map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn ab_find_contacts_paged(
            &self,
            abfind_contacts_paged_message: ports::AbfindContactsPagedMessage,
        ) -> Result<ports::AbfindContactsPagedResponseMessage, Option<SoapFault>> {
            let __request =
                AbfindContactsPagedMessageSoapEnvelope::new(SoapAbfindContactsPagedMessage {
                    body: abfind_contacts_paged_message,
                });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/ABFindContactsPaged",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: AbfindContactsPagedResponseMessageSoapEnvelope =
                from_str(&response).map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn find_friends_in_common(
            &self,
            find_friends_in_common_message: ports::FindFriendsInCommonMessage,
        ) -> Result<ports::FindFriendsInCommonResponseMessage, Option<SoapFault>> {
            let __request =
                FindFriendsInCommonMessageSoapEnvelope::new(SoapFindFriendsInCommonMessage {
                    body: find_friends_in_common_message,
                    xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/FindFriendsInCommon",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: FindFriendsInCommonResponseMessageSoapEnvelope =
                from_str(&response).map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn create_contact(
            &self,
            create_contact_message: ports::CreateContactMessage,
        ) -> Result<ports::CreateContactResponseMessage, Option<SoapFault>> {
            let __request = CreateContactMessageSoapEnvelope::new(SoapCreateContactMessage {
                body: create_contact_message,
                xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
            });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/CreateContact",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: CreateContactResponseMessageSoapEnvelope =
                from_str(&response).map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn manage_wl_connection(
            &self,
            manage_wl_connection_message: ports::ManageWLConnectionMessage,
        ) -> Result<ports::ManageWLConnectionResponseMessage, Option<SoapFault>> {
            let __request =
                ManageWLConnectionMessageSoapEnvelope::new(SoapManageWLConnectionMessage {
                    body: manage_wl_connection_message,
                    xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/ManageWLConnection",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: ManageWLConnectionResponseMessageSoapEnvelope =
                from_str(&response).map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn break_connection(
            &self,
            break_connection_message: ports::BreakConnectionMessage,
        ) -> Result<ports::BreakConnectionResponseMessage, Option<SoapFault>> {
            let __request = BreakConnectionMessageSoapEnvelope::new(SoapBreakConnectionMessage {
                body: break_connection_message,
                xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
            });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/BreakConnection",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: BreakConnectionResponseMessageSoapEnvelope =
                from_str(&response).map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn add_dynamic_item(
            &self,
            add_dynamic_item_message: ports::AddDynamicItemMessage,
        ) -> Result<ports::AddDynamicItemResponseMessage, Option<SoapFault>> {
            let __request = AddDynamicItemMessageSoapEnvelope::new(SoapAddDynamicItemMessage {
                body: add_dynamic_item_message,
                xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
            });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/AddDynamicItem",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: AddDynamicItemResponseMessageSoapEnvelope =
                from_str(&response).map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn ab_find_by_contacts(
            &self,
            abfind_by_contacts_message: ports::AbfindByContactsMessage,
        ) -> Result<ports::AbfindByContactsResponseMessage, Option<SoapFault>> {
            let __request = AbfindByContactsMessageSoapEnvelope::new(SoapAbfindByContactsMessage {
                body: abfind_by_contacts_message,
                xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
            });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/ABFindByContacts",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: AbfindByContactsResponseMessageSoapEnvelope =
                from_str(&response).map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
    }

    impl WhatsUpServiceBinding {
        async fn send_soap_request<T: YaSerialize>(
            &self,
            request: &T,
            action: &str,
        ) -> SoapResponse {
            let body = to_string(request).expect("failed to generate xml");
            debug!("SOAP Request: {}", body);
            let mut req = self
                .client
                .post(&self.url)
                .body(body)
                .header("Content-Type", "text/xml")
                .header("Soapaction", action);
            if let Some(credentials) = &self.credentials {
                req = req.basic_auth(
                    credentials.0.to_string(),
                    Option::Some(credentials.1.to_string()),
                );
            }
            let res = req.send().await?;
            let status = res.status();
            debug!("SOAP Status: {}", status);
            let txt = res.text().await.unwrap_or_default();
            debug!("SOAP Response: {}", txt);
            Ok((status, txt))
        }
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetContactsRecentActivityMessage {
        #[yaserde(rename = "GetContactsRecentActivity", default)]
        pub body: ports::GetContactsRecentActivityMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct GetContactsRecentActivityMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapGetContactsRecentActivityMessage,
    }

    impl GetContactsRecentActivityMessageSoapEnvelope {
        pub fn new(body: SoapGetContactsRecentActivityMessage) -> Self {
            GetContactsRecentActivityMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetContactsRecentActivityResponseMessage {
        #[yaserde(rename = "GetContactsRecentActivityResponseMessage", default)]
        pub body: ports::GetContactsRecentActivityResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct GetContactsRecentActivityResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapGetContactsRecentActivityResponseMessage,
    }

    impl GetContactsRecentActivityResponseMessageSoapEnvelope {
        pub fn new(body: SoapGetContactsRecentActivityResponseMessage) -> Self {
            GetContactsRecentActivityResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetBatchRecentActivityMessage {
        #[yaserde(rename = "GetBatchRecentActivity", default)]
        pub body: ports::GetBatchRecentActivityMessage,
        #[yaserde(attribute)]
        pub xmlns: Option<String>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct GetBatchRecentActivityMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapGetBatchRecentActivityMessage,
    }

    impl GetBatchRecentActivityMessageSoapEnvelope {
        pub fn new(body: SoapGetBatchRecentActivityMessage) -> Self {
            GetBatchRecentActivityMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetBatchRecentActivityResponseMessage {
        #[yaserde(rename = "GetBatchRecentActivityResponseMessage", default)]
        pub body: ports::GetBatchRecentActivityResponseMessage,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,
    }
    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
        rename = "Envelope",
        namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
        prefix = "soap"
    )]
    pub struct GetBatchRecentActivityResponseMessageSoapEnvelope {
        #[yaserde(rename = "encodingStyle", prefix = "soap", attribute)]
        pub encoding_style: String,
        #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
        pub tnsattr: Option<String>,
        #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
        pub urnattr: Option<String>,
        #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
        pub xsiattr: Option<String>,
        #[yaserde(rename = "Header", prefix = "soap")]
        pub header: Option<Header>,
        #[yaserde(rename = "Body", prefix = "soap")]
        pub body: SoapGetBatchRecentActivityResponseMessage,
    }

    impl GetBatchRecentActivityResponseMessageSoapEnvelope {
        pub fn new(body: SoapGetBatchRecentActivityResponseMessage) -> Self {
            GetBatchRecentActivityResponseMessageSoapEnvelope {
                encoding_style: SOAP_ENCODING.to_string(),
                tnsattr: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                body,
                urnattr: None,
                xsiattr: None,
                header: None,
            }
        }
    }

    impl Default for WhatsUpServiceBinding {
        fn default() -> Self {
            WhatsUpServiceBinding {
                client: reqwest::Client::new(),
                url: "http://www.msn.com/webservices/AddressBook".to_string(),
                credentials: Option::None,
            }
        }
    }
    impl WhatsUpServiceBinding {
        pub fn new(url: &str, credentials: Option<(String, String)>) -> Self {
            WhatsUpServiceBinding {
                client: reqwest::Client::new(),
                url: url.to_string(),
                credentials,
            }
        }
    }
    pub struct WhatsUpServiceBinding {
        client: reqwest::Client,
        url: String,
        credentials: Option<(String, String)>,
    }
    #[async_trait]
    impl ports::WhatsUpServicePortType for WhatsUpServiceBinding {
        async fn get_contacts_recent_activity(
            &self,
            get_contacts_recent_activity_message: ports::GetContactsRecentActivityMessage,
        ) -> Result<ports::GetContactsRecentActivityResponseMessage, Option<SoapFault>> {
            let __request = GetContactsRecentActivityMessageSoapEnvelope::new(
                SoapGetContactsRecentActivityMessage {
                    body: get_contacts_recent_activity_message,
                    xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                },
            );

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/GetContactsRecentActivity",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetContactsRecentActivityResponseMessageSoapEnvelope = from_str(&response)
                .map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
        async fn get_batch_recent_activity(
            &self,
            get_batch_recent_activity_message: ports::GetBatchRecentActivityMessage,
        ) -> Result<ports::GetBatchRecentActivityResponseMessage, Option<SoapFault>> {
            let __request =
                GetBatchRecentActivityMessageSoapEnvelope::new(SoapGetBatchRecentActivityMessage {
                    body: get_batch_recent_activity_message,
                    xmlns: Option::Some("http://www.msn.com/webservices/AddressBook".to_string()),
                });

            let (status, response) = self
                .send_soap_request(
                    &__request,
                    "http://www.msn.com/webservices/AddressBook/GetBatchRecentActivity",
                )
                .await
                .map_err(|err| {
                    warn!("Failed to send SOAP request: {:?}", err);
                    None
                })?;

            let r: GetBatchRecentActivityResponseMessageSoapEnvelope = from_str(&response)
                .map_err(|err| {
                    warn!("Failed to unmarshal SOAP response: {:?}", err);
                    None
                })?;
            if status.is_success() {
                Ok(r.body.body)
            } else {
                Err(r.body.fault)
            }
        }
    }
}

pub mod services {
    use super::*;
    use async_trait::async_trait;
    use yaserde::de::from_str;
    use yaserde::ser::to_string;
    use yaserde::{YaDeserialize, YaSerialize};
    pub struct SharingService {}
    impl SharingService {
        pub fn new_client(
            credentials: Option<(String, String)>,
        ) -> bindings::SharingServiceBinding {
            bindings::SharingServiceBinding::new(
                "https://contacts.msn.com/abservice/SharingService.asmx",
                credentials,
            )
        }
    }
    pub struct Abservice {}
    impl Abservice {
        pub fn new_client(credentials: Option<(String, String)>) -> bindings::AbserviceBinding {
            bindings::AbserviceBinding::new(
                "https://contacts.msn.com/abservice/abservice.asmx",
                credentials,
            )
        }
    }
    pub struct WhatsUpService {}
    impl WhatsUpService {
        pub fn new_client(
            credentials: Option<(String, String)>,
        ) -> bindings::WhatsUpServiceBinding {
            bindings::WhatsUpServiceBinding::new(
                "http://sup.live.com/whatsnew/whatsnewservice.asmx",
                credentials,
            )
        }
    }
}

pub mod factories {
    use crate::{generated::msnab_datatypes::types::*, models::uuid::UUID};

    use chrono::{Local, DateTime, NaiveDateTime};
    use lazy_static::lazy_static;

    use super::{bindings::{FindMembershipResponseMessageSoapEnvelope, SoapFindMembershipResponseMessage, AbfindContactsPagedResponseMessageSoapEnvelope, SoapAbfindContactsPagedResponseMessage, AbgroupAddResponseMessageSoapEnvelope, SoapAbgroupAddResponseMessage, UpdateDynamicItemResponseMessageSoapEnvelope, SoapUpdateDynamicItemResponseMessage}, types::{MembershipResult, FindMembershipResponse, Ab, Groups, AbfindContactsPagedResultType, AbgroupAddResponse, AbgroupAddResultType}, messages::{FindMembershipResponseMessage, ServiceHeaderContainer, AbgroupAddResponseMessage, UpdateDynamicItemResponseMessage}, ports};


    pub struct FindMembershipResponseFactory;

    impl FindMembershipResponseFactory {

        pub fn get_empty_response(uuid: UUID, msn_addr: String, cache_key: String) -> FindMembershipResponseMessageSoapEnvelope {

            let circle_attributes = CircleAttributesType{ is_presence_enabled: false, is_event: None, domain: String::from("WindowsLive") };
            let handle = Handle { id: UUID::nil().to_string(), is_passport_name_hidden: false, cid: String::from("0") };
            let owner_namespace_info = OwnerNamespaceInfoType{ handle: handle, creator_puid: String::from("0"), creator_cid: uuid.to_decimal_cid(), creator_passport_name: msn_addr, circle_attributes: circle_attributes, messenger_application_service_created: Some(false) };
            
            let now = Local::now();

            let owner_namespace = OwnerNamespaceType{ info: owner_namespace_info, changes: String::new(), create_date: String::from("2014-10-31T00:00:00Z"), last_change: now.format("%Y-%m-%dT%H:%M:%SZ").to_string() };
            
            let mut services = Vec::new();
            services.push(FindMembershipResponseFactory::get_messenger_service(Vec::new(), Vec::new(), Vec::new(), Vec::new(), true));
    
            let array_of_services = ArrayOfServiceType{ service: services };
            
            let result = MembershipResult{ services: Some(array_of_services), owner_namespace: Some(owner_namespace) };
            let fmr = FindMembershipResponse{ find_membership_result: result };
            let body = FindMembershipResponseMessage { find_membership_response: fmr };
            let message = SoapFindMembershipResponseMessage {body:body, fault: None };

            let mut response = FindMembershipResponseMessageSoapEnvelope::new(message);

            response.header = Some(HeaderFactory::get_service_header(cache_key));
            return response;
        }

        pub fn get_response(uuid: UUID, msn_addr: String, cache_key: String, messenger_service: ServiceType) -> FindMembershipResponseMessageSoapEnvelope {

            let circle_attributes = CircleAttributesType{ is_presence_enabled: false, is_event: None, domain: String::from("WindowsLive") };
            let handle = Handle { id: UUID::nil().to_string(), is_passport_name_hidden: false, cid: String::from("0") };
            let owner_namespace_info = OwnerNamespaceInfoType{ handle: handle, creator_puid: String::from("0"), creator_cid: uuid.to_decimal_cid(), creator_passport_name: msn_addr, circle_attributes: circle_attributes, messenger_application_service_created: Some(false) };
            
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

            response.header = Some(HeaderFactory::get_service_header(cache_key));
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

    pub struct MemberFactory;

    impl MemberFactory {

        pub fn get_passport_member(uuid: &UUID, msn_addr: &String, state: MemberState, role_id: RoleId, deleted: bool) -> BaseMember {
            let now = Local::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
            let create_date = String::from("2014-10-31T00:00:00Z");
            let no_date = String::from("0001-01-01T00:00:00");
            let membership_id = format!("{}\\{}", role_id.to_string(), uuid.to_string());
            return BaseMember{ membership_id: Some(membership_id), xsi_type: String::from("PassportMember"), rs_type: MemberType::Passport, location: None, display_name: None, state, passport_name: Some(msn_addr.clone()), circle_id: None, is_passport_name_hidden: Some(false), passport_id: Some(0), cid: Some(uuid.to_decimal_cid()), passport_changes: Some(String::new()), lookedup_by_cid: Some(false), new_role: None, annotations: None, deleted: Some(deleted), last_changed: Some(now), joined_date: Some(create_date), expiration_date: Some(no_date), changes: Some(String::new()) };
        }


    }

    pub struct ContactFactory;

    impl ContactFactory {

        pub fn get_me_contact(uuid: &UUID, msn_addr: &String) -> ContactType {

            let now = Local::now();


            let mut annotation_array : Vec<Annotation> = Vec::new();
            annotation_array.push(AnnotationFactory::get_roam_live_properties(Some(true)));
            annotation_array.push(AnnotationFactory::get_mbea(Some(false)));
            annotation_array.push(AnnotationFactory::get_gtc(Some(true)));
            annotation_array.push(AnnotationFactory::get_blp(Some(true)));

            let array_of_annotations = ArrayOfAnnotation{ annotation: annotation_array };

            let me_contact_info = ContactInfoType{ emails: None, phones: None, locations: None, web_sites: None, annotations: Some(array_of_annotations), group_ids: None, group_ids_deleted: None, contact_type: Some(ContactTypeEnum::Me), quick_name: Some(msn_addr.clone()), first_name: None, middle_name: None, last_name: None, suffix: None, name_title: None, passport_name: Some(msn_addr.clone()), display_name: Some(msn_addr.clone()), puid: Some(0), cid: Some(uuid.to_decimal_cid()), brand_id_list: None, comment: None, is_mobile_im_enabled: Some(false), is_messenger_user: Some(true), is_favorite: Some(false), is_smtp: Some(false), has_space: Some(true), spot_watch_state: Some(String::from("NoDevice")), birthdate: Some(String::from("0001-01-01T00:00:00")), primary_email_type: Some(ContactEmailTypeType{ body: String::from("Passport") }), primary_location: Some(ContactLocationTypeType{ body: String::from("ContactLocationPersonal") }), primary_phone: Some(String::from("ContactPhonePersonal")), is_private: Some(false), anniversary: None, gender: Some(String::from("Unspecified") ), time_zone: Some(String::from("None")), trust_level: None, network_info_list: None, public_display_name: None, is_auto_update_disabled: None, is_hidden: None, is_passport_name_hidden: Some(false), is_not_mobile_visible: Some(false), is_shell_contact: None, messenger_member_info: None, properties_changed: None, client_error_data: None, link_info: None, source_handle: None, file_as: None, ur_ls: None };
            return ContactType{ contact_id: Some(uuid.to_string()), contact_info: Some(me_contact_info), properties_changed: Some(String::new()), f_deleted: Some(false), last_change: Some(now.format("%Y-%m-%dT%H:%M:%SZ").to_string()), create_date: None, last_modified_by: None, created_by: None };

        }

        pub fn get_contact(uuid: &UUID, msn_addr: &String, contact_type: ContactTypeEnum, deleted: bool) -> ContactType {
            let now = Local::now();
            let contact_info = ContactInfoType{ emails: None, phones: None, locations: None, web_sites: None, annotations: None, group_ids: None, group_ids_deleted: None, contact_type: Some(contact_type), quick_name: Some(msn_addr.clone()), first_name: None, middle_name: None, last_name: None, suffix: None, name_title: None, passport_name: Some(msn_addr.clone()), display_name: Some(msn_addr.clone()), puid: Some(0), cid: Some(uuid.to_decimal_cid()), brand_id_list: None, comment: None, is_mobile_im_enabled: Some(false), is_messenger_user: Some(true), is_favorite: Some(false), is_smtp: Some(false), has_space: Some(true), spot_watch_state: Some(String::from("NoDevice")), birthdate: Some(String::from("0001-01-01T00:00:00")), primary_email_type: Some(ContactEmailTypeType{ body: String::from("Passport") }), primary_location: Some(ContactLocationTypeType{ body: String::from("ContactLocationPersonal") }), primary_phone: Some(String::from("ContactPhonePersonal")), is_private: Some(false), anniversary: None, gender: Some(String::from("Unspecified") ), time_zone: Some(String::from("None")), trust_level: None, network_info_list: None, public_display_name: None, is_auto_update_disabled: None, is_hidden: None, is_passport_name_hidden: Some(false), is_not_mobile_visible: Some(false), is_shell_contact: None, messenger_member_info: None, properties_changed: None, client_error_data: None, link_info: None, source_handle: None, file_as: None, ur_ls: None };
            return ContactType{ contact_id: Some(uuid.to_string()), contact_info: Some(contact_info), properties_changed: Some(String::new()), f_deleted: Some(deleted), last_change: Some(now.format("%Y-%m-%dT%H:%M:%SZ").to_string()), create_date: None, last_modified_by: None, created_by: None };
        }

    }

    pub struct FindContactsPagedResponseFactory;

    impl FindContactsPagedResponseFactory {

        pub fn get_response(uuid: UUID, cache_key: String, msn_addr: String, contacts: Vec<ContactType>) -> AbfindContactsPagedResponseMessageSoapEnvelope {

            let now = Local::now();
    
            let create_date = String::from("2014-10-31T00:00:00Z");

            let ab_info_type = AbInfoType{ migrated_to: None, beta_status: None, name: None, owner_puid: 0, owner_cid: uuid.to_decimal_cid(), owner_email:Some(msn_addr.clone()), f_default: true, joined_namespace: false, is_bot: false, is_parent_managed: false, account_tier: None, account_tier_last_changed: String::from("0001-01-01T00:00:00"), profile_version: 0, subscribe_external_partner: false, notify_external_partner: false, address_book_type: String::from("Individual"), messenger_application_service_created: None, is_beta_migrated: None, last_relevance_update: None };
            let ab = Ab{ ab_id: UUID::nil().to_string(), ab_info: ab_info_type, last_change: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(), dynamic_item_last_changed: String::from("0001-01-01T00:00:00"), recent_activity_item_last_changed: None, create_date: create_date.clone(), properties_changed: String::new() };
    
            let mut contact_array = contacts;
            contact_array.push(ContactFactory::get_me_contact(&uuid, &msn_addr));
            let array_of_contact = ArrayOfContactType{ contact: contact_array };

            let mut favorite_annotation_arary : Vec<Annotation> = Vec::new();
            favorite_annotation_arary.push(AnnotationFactory::get_display(Some(true)));
            let favorite_array_of_annotations = ArrayOfAnnotation{ annotation: favorite_annotation_arary };


            let favorite_group = GroupType{ group_id: String::from("1ae28c79-c963-4fe6-8339-d72a0f7c8bd2"), group_info: GroupInfoType{ annotations: Some(favorite_array_of_annotations), group_type: Some(String::from("c8529ce2-6ead-434d-881f-341e17db3ff8")), name: Some(String::from("Favorites")), is_not_mobile_visible: Some(false), is_private: Some(false), is_favorite: Some(true), f_messenger: None }, properties_changed: String::new(), f_deleted: Some(false), last_change: Some(create_date.clone())};
            let mut group_array : Vec<GroupType> = Vec::new();
            group_array.push(favorite_group);
            let groups = Groups{ group: group_array };
    
            let circle_result = CircleResultType{ circles: None, circle_ticket: String::from("&lt;?xml version=\"1.0\" encoding=\"utf-16\"?&gt;&lt;SignedTicket xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" ver=\"1\" keyVer=\"1\"&gt;&lt;Data&gt;PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTE2Ij8+DQo8VGlja2V0IHhtbG5zOnhzaT0iaHR0cDovL3d3dy53My5vcmcvMjAwMS9YTUxTY2hlbWEtaW5zdGFuY2UiIHhtbG5zOnhzZD0iaHR0cDovL3d3dy53My5vcmcvMjAwMS9YTUxTY2hlbWEiPg0KICA8VFM+MDAwMC0wMS0wMVQwMDowMDowMDwvVFM+DQogIDxDSUQ+LTc3NzY5ODI1NzkyNzI5Mzc1NzI8L0NJRD4NCjwvVGlja2V0Pg==&lt;/Data&gt;&lt;Sig&gt;SLE8LXFmBW/2nMY9t+lG/7w4APZt3Z5U4nsu3G7KSWSdTEvTt9mt2kdssQaxxjEhy8udrLlC2dFSQXtHI/6mmbHhtaf7wx2WvRb4F1ayv5kZmrp5lJPkEXhdSwzJHlYPZM530Gsr7Md9MW4w67F7ct7i2MhsQyBLXr5nEDLlILHjTNUkbIa31IZJ5Qpwnr7Cj4XLPYOl8Phl6mHSjWdLo/CvohxRnAb/akABRyIhdd4rIvZREYsYhjSyZ/RLc6j0eLF7zkn8jjLKVGkIIFNvcGGnv/9ZtQ4zO5a/OkNB18Pvj6excNHt8zeCXiPomIikZrUOEZ4sshYRAJ7/5k/PAA==&lt;/Sig&gt;&lt;/SignedTicket&gt;") };
    
            let result = AbfindContactsPagedResultType{ groups: Some(groups), contacts: Some(array_of_contact), circle_result: circle_result, ab: ab };
            let body_body = ports::AbfindContactsPagedResponseMessage{ ab_find_contacts_paged_result: result };
            let body = SoapAbfindContactsPagedResponseMessage{ body: body_body, fault: None };

            return AbfindContactsPagedResponseMessageSoapEnvelope{header: Some(HeaderFactory::get_service_header(cache_key)), body: body };
            }


        
    }

    pub struct UpdateDynamicItemResponseFactory;
    impl UpdateDynamicItemResponseFactory {
        pub fn get_response(cache_key: String) -> UpdateDynamicItemResponseMessageSoapEnvelope {
            let body_content = UpdateDynamicItemResponseMessage{ update_dynamic_item_response: None };
            let body = SoapUpdateDynamicItemResponseMessage { body: body_content, fault: None };
            return UpdateDynamicItemResponseMessageSoapEnvelope{ header: Some(HeaderFactory::get_service_header(cache_key)), body: body };
        }
    }

    pub struct AnnotationFactory;
        
    impl AnnotationFactory {

        fn parse_boolean_value(value: &bool) -> String {
            if *value {
                return String::from("1");
            } else {
                return String::from("0");
            }
        }

        pub fn get_roam_live_properties(value : Option<bool>) -> Annotation {
            let value = value.unwrap_or(false);
            return Annotation { name: String::from("MSN.IM.RoamLiveProperties"), value: Some(AnnotationFactory::parse_boolean_value(&value)) };
        }

        pub fn get_mbea(value : Option<bool>) -> Annotation {
            let value = value.unwrap_or(false);
            return Annotation { name: String::from("MSN.IM.MBEA"), value: Some(AnnotationFactory::parse_boolean_value(&value)) };
        }

        pub fn get_gtc(value : Option<bool>) -> Annotation {
            let value = value.unwrap_or(false);
            return Annotation { name: String::from("MSN.IM.GTC"), value: Some(AnnotationFactory::parse_boolean_value(&value)) };
        }

        pub fn get_blp(value : Option<bool>) -> Annotation {
            let value = value.unwrap_or(false);
            return Annotation { name: String::from("MSN.IM.GTC"), value: Some(AnnotationFactory::parse_boolean_value(&value)) };
        }

        pub fn get_display(value: Option<bool>) -> Annotation {
            let value = value.unwrap_or(false);
            return Annotation { name: String::from("MSN.IM.Display"), value: Some(AnnotationFactory::parse_boolean_value(&value)) };
        }

        pub fn get_invite(message: String) -> Annotation {
            return Annotation { name: String::from("MSN.IM.InviteMessage"), value: Some(message) };
        }

    }

    pub struct ABGroupAddResponseFactory;
    impl ABGroupAddResponseFactory {

        pub fn get_favorite_group_added_response(guid: String, cache_key: String) -> AbgroupAddResponseMessageSoapEnvelope {
            let result = AbgroupAddResultType { guid };
            let group_add_response = AbgroupAddResponse{ ab_group_add_result: Some(result) };
    
           let body_content = AbgroupAddResponseMessage{ ab_group_add_response: group_add_response };
    
           let body = SoapAbgroupAddResponseMessage{ body: body_content, fault: None };
           return AbgroupAddResponseMessageSoapEnvelope{ header: Some(HeaderFactory::get_service_header(cache_key)), body: body  };
        }

    }

    pub struct HeaderFactory;

    impl HeaderFactory {

        pub fn get_service_header(cache_key: String) -> ServiceHeaderContainer {
            let service_header_1 = super::types::ServiceHeader{ version: String::from("15.01.1408.0000"), cache_key: Some(cache_key), cache_key_changed: Some(true), preferred_host_name: Some(String::from("localhost")), session_id: Some(String::from("17340b67-dcad-48ea-89fb-5e84fbc54cf8")) };
            let service_header = super::messages::ServiceHeader { service_header: service_header_1 };
            return ServiceHeaderContainer{ service_header };
        }
    }





    #[cfg(test)]
    mod tests {
        use super::*;
        use yaserde::de::from_str;
        use yaserde::ser::to_string;

        #[test]
        fn test_get_empty_find_membership_response() {

            let response = FindMembershipResponseFactory::get_empty_response(UUID::from_string(&String::from("TEST")), String::from("test@matrix.org"), String::from("c4che_key"));
            let serialized = to_string(&response).unwrap();
        }


    }


}

#[cfg(test)]
mod tests {
    use log::{warn, debug, info};
    use yaserde::de::from_str;
    use yaserde::ser::to_string;

    use crate::{generated::msnab_datatypes::types::{OwnerNamespaceType, OwnerNamespaceInfoType, Handle, CircleAttributesType, ArrayOfServiceType, ServiceType, Memberships, Membership, RoleId, Members, BaseMember, MemberState, InfoType, HandleType, ServiceName, ArrayOfContactType, GroupType, ContactType, CircleResultType, AbInfoType, MemberType}, models::uuid::UUID};

    use super::{bindings::{FindMembershipResponseMessageSoapEnvelope, SoapFindMembershipResponseMessage, FindMembershipMessageSoapEnvelope, AbfindContactsPagedMessageSoapEnvelope, AbfindContactsPagedResponseMessageSoapEnvelope, SoapAbfindContactsPagedResponseMessage, AbgroupAddResponseMessageSoapEnvelope, SoapAbgroupAddResponseMessage, AbgroupAddMessageSoapEnvelope, UpdateDynamicItemMessageSoapEnvelope, UpdateDynamicItemResponseMessageSoapEnvelope, SoapUpdateDynamicItemResponseMessage}, messages::{FindMembershipResponseMessage, ServiceHeaderContainer, AbgroupAddResponseMessage, UpdateDynamicItemResponseMessage}, types::{FindMembershipResponse, MembershipResult, AbfindContactsPagedResultType, Groups, Ab, AbgroupAddResponse, AbgroupAddResultType}, ports};

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
        let owner_namespace_info = OwnerNamespaceInfoType{ handle: handle, creator_puid: String::from("0"), creator_cid: 863314, creator_passport_name: String::from("aeon@test.fr"), circle_attributes: circle_attributes, messenger_application_service_created: Some(false) };
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


    #[test]
    fn test_find_membership_request() {

        let request_body = "<?xml version=\"1.0\" encoding=\"utf-8\"?><soap:Envelope xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:soapenc=\"http://schemas.xmlsoap.org/soap/encoding/\"><soap:Header><ABApplicationHeader xmlns=\"http://www.msn.com/webservices/AddressBook\"><ApplicationId>AAD9B99B-58E6-4F23-B975-D9EC1F9EC24A</ApplicationId><IsMigration>false</IsMigration><PartnerScenario>Initial</PartnerScenario><CacheKey>12r1:8nBBE6vX1J4uPKajtbem5XBIblimCwAhIziAeEAwYD0AMiaztryWvcZthkN9oX_pl2scBKXfKvRvuWKYdHUNuRkgiyV9rzcDpnDIDiM6vdcEB6d82wjjnL4TAFAjc5X8i-C94mNfQvujUk470P7fz9qbWfK6ANcEtygDb-oWsYVfEBrxl6geTUg9tGT7yCIsls7ECcLyqwsROuAbWCrued_VPKiUgSIvqG8gaA</CacheKey></ABApplicationHeader><ABAuthHeader xmlns=\"http://www.msn.com/webservices/AddressBook\"><ManagedGroupRequest>false</ManagedGroupRequest><TicketToken>t=0bfus4t3d_t0k3n</TicketToken></ABAuthHeader></soap:Header><soap:Body><FindMembership xmlns=\"http://www.msn.com/webservices/AddressBook\"><serviceFilter><Types><ServiceType>Messenger</ServiceType><ServiceType>SocialNetwork</ServiceType><ServiceType>Space</ServiceType><ServiceType>Profile</ServiceType></Types></serviceFilter><View>Full</View><deltasOnly>true</deltasOnly><lastChange>2022-04-20T13:03:28Z</lastChange></FindMembership></soap:Body></soap:Envelope>";
        let r: FindMembershipMessageSoapEnvelope = from_str(&request_body).unwrap();
        
        let header = &r.header.unwrap();
        assert_eq!(r.body.body.find_membership_request.deltas_only, true);
        assert_eq!(header.ab_auth_header.ticket_token, String::from("t=0bfus4t3d_t0k3n"));
        assert_eq!(header.application_header.partner_scenario, String::from("Initial"));

    }

    #[test]
    fn test_find_contacts_paged_request() {
        let request_body = "<?xml version=\"1.0\" encoding=\"utf-8\"?><soap:Envelope xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:soapenc=\"http://schemas.xmlsoap.org/soap/encoding/\"><soap:Header><ABApplicationHeader xmlns=\"http://www.msn.com/webservices/AddressBook\"><ApplicationId>AAD9B99B-58E6-4F23-B975-D9EC1F9EC24A</ApplicationId><IsMigration>false</IsMigration><PartnerScenario>Initial</PartnerScenario><CacheKey>12r1:8nBBE6vX1J4uPKajtbem5XBIblimCwAhIziAeEAwYD0AMiaztryWvcZthkN9oX_pl2scBKXfKvRvuWKYdHUNuRkgiyV9rzcDpnDIDiM6vdcEB6d82wjjnL4TAFAjc5X8i-C94mNfQvujUk470P7fz9qbWfK6ANcEtygDb-oWsYVfEBrxl6geTUg9tGT7yCIsls7ECcLyqwsROuAbWCrued_VPKiUgSIvqG8gaA</CacheKey></ABApplicationHeader><ABAuthHeader xmlns=\"http://www.msn.com/webservices/AddressBook\"><ManagedGroupRequest>false</ManagedGroupRequest><TicketToken>t=0bfus4t3d_t0k3n</TicketToken></ABAuthHeader></soap:Header><soap:Body><ABFindContactsPaged xmlns=\"http://www.msn.com/webservices/AddressBook\"><filterOptions><DeltasOnly>true</DeltasOnly><LastChanged>2022-04-21T19:49:28Z</LastChanged><ContactFilter><IncludeHiddenContacts>true</IncludeHiddenContacts></ContactFilter></filterOptions><abView>MessengerClient8</abView><extendedContent>AB AllGroups CircleResult</extendedContent></ABFindContactsPaged></soap:Body></soap:Envelope>";
        let r : AbfindContactsPagedMessageSoapEnvelope = from_str(&request_body).unwrap();

        let header = &r.header.unwrap();
        assert_eq!(header.ab_auth_header.ticket_token, String::from("t=0bfus4t3d_t0k3n"));
        assert_eq!(header.application_header.partner_scenario, String::from("Initial"));
    }

    #[test]
    fn test_find_contacts_paged_response() {
        let service_header_1 = super::types::ServiceHeader{ version: String::from("15.01.1408.0000"), cache_key: Some(String::from("cache_key")), cache_key_changed: Some(true), preferred_host_name: Some(String::from("localhost")), session_id: None };
        let service_header = super::messages::ServiceHeader { service_header: service_header_1 };
        let service_header_container = ServiceHeaderContainer{ service_header };

        let ab_info_type = AbInfoType{ migrated_to: None, beta_status: None, name: None, owner_puid: 0, owner_cid: 0, owner_email:None, f_default: false, joined_namespace: false, is_bot: false, is_parent_managed: false, account_tier: None, account_tier_last_changed: String::new(), profile_version: 0, subscribe_external_partner: false, notify_external_partner: false, address_book_type: String::new(), messenger_application_service_created: None, is_beta_migrated: None, last_relevance_update: None};
        let ab = Ab{ ab_id: String::from("new_ab_id"), ab_info: ab_info_type, last_change: String::new(), dynamic_item_last_changed: String::new(), recent_activity_item_last_changed: None, create_date: String::new(), properties_changed: String::new() };

        let contact_array : Vec<ContactType> = Vec::new();
        let group_array : Vec<GroupType> = Vec::new();

        let array_of_contact = ArrayOfContactType{ contact: contact_array };
        let groups = Groups{ group: group_array };

        let circle_result = CircleResultType{ circles: None, circle_ticket: String::from("&lt;?xml version=\"1.0\" encoding=\"utf-16\"?&gt;&lt;SignedTicket xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" ver=\"1\" keyVer=\"1\"&gt;&lt;Data&gt;PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTE2Ij8+DQo8VGlja2V0IHhtbG5zOnhzaT0iaHR0cDovL3d3dy53My5vcmcvMjAwMS9YTUxTY2hlbWEtaW5zdGFuY2UiIHhtbG5zOnhzZD0iaHR0cDovL3d3dy53My5vcmcvMjAwMS9YTUxTY2hlbWEiPg0KICA8VFM+MDAwMC0wMS0wMVQwMDowMDowMDwvVFM+DQogIDxDSUQ+LTc3NzY5ODI1NzkyNzI5Mzc1NzI8L0NJRD4NCjwvVGlja2V0Pg==&lt;/Data&gt;&lt;Sig&gt;SLE8LXFmBW/2nMY9t+lG/7w4APZt3Z5U4nsu3G7KSWSdTEvTt9mt2kdssQaxxjEhy8udrLlC2dFSQXtHI/6mmbHhtaf7wx2WvRb4F1ayv5kZmrp5lJPkEXhdSwzJHlYPZM530Gsr7Md9MW4w67F7ct7i2MhsQyBLXr5nEDLlILHjTNUkbIa31IZJ5Qpwnr7Cj4XLPYOl8Phl6mHSjWdLo/CvohxRnAb/akABRyIhdd4rIvZREYsYhjSyZ/RLc6j0eLF7zkn8jjLKVGkIIFNvcGGnv/9ZtQ4zO5a/OkNB18Pvj6excNHt8zeCXiPomIikZrUOEZ4sshYRAJ7/5k/PAA==&lt;/Sig&gt;&lt;/SignedTicket&gt;") };

        let result = AbfindContactsPagedResultType{ groups: Some(groups), contacts: Some(array_of_contact), circle_result: circle_result, ab: ab };
        let body_body = ports::AbfindContactsPagedResponseMessage{ ab_find_contacts_paged_result: result };
        let body = SoapAbfindContactsPagedResponseMessage{ body: body_body, fault: None };
        let r = AbfindContactsPagedResponseMessageSoapEnvelope{header: Some(service_header_container), body: body };

        let serialized = to_string(&r).unwrap();
        println!("{}", serialized);
    }

    #[test]
    fn test_ab_group_add_request() {
        let request = "<?xml version=\"1.0\" encoding=\"utf-8\"?><soap:Envelope xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:soapenc=\"http://schemas.xmlsoap.org/soap/encoding/\"><soap:Header><ABApplicationHeader xmlns=\"http://www.msn.com/webservices/AddressBook\"><ApplicationId>AAD9B99B-58E6-4F23-B975-D9EC1F9EC24A</ApplicationId><IsMigration>false</IsMigration><PartnerScenario>Initial</PartnerScenario><CacheKey>12r1:8nBBE6vX1J4uPKajtbem5XBIblimCwAhIziAeEAwYD0AMiaztryWvcZthkN9oX_pl2scBKXfKvRvuWKYdHUNuRkgiyV9rzcDpnDIDiM6vdcEB6d82wjjnL4TAFAjc5X8i-C94mNfQvujUk470P7fz9qbWfK6ANcEtygDb-oWsYVfEBrxl6geTUg9tGT7yCIsls7ECcLyqwsROuAbWCrued_VPKiUgSIvqG8gaA</CacheKey></ABApplicationHeader><ABAuthHeader xmlns=\"http://www.msn.com/webservices/AddressBook\"><ManagedGroupRequest>false</ManagedGroupRequest><TicketToken>t=0bfusc4t3dT0k3n</TicketToken></ABAuthHeader></soap:Header><soap:Body><ABGroupAdd xmlns=\"http://www.msn.com/webservices/AddressBook\"><abId>00000000-0000-0000-0000-000000000000</abId><groupAddOptions><fRenameOnMsgrConflict>false</fRenameOnMsgrConflict></groupAddOptions><groupInfo><GroupInfo><name>Favorites</name><groupType>C8529CE2-6EAD-434d-881F-341E17DB3FF8</groupType><fMessenger>false</fMessenger><IsFavorite>true</IsFavorite><annotations><Annotation><Name>MSN.IM.Display</Name><Value>1</Value></Annotation></annotations></GroupInfo></groupInfo></ABGroupAdd></soap:Body></soap:Envelope>";
        
        let request_deserialized : AbgroupAddMessageSoapEnvelope = from_str(request).unwrap();

        let header = &request_deserialized.header.unwrap();
        assert_eq!(request_deserialized.body.body.ab_group_add_request.ab_id, String::from("00000000-0000-0000-0000-000000000000"));
        assert_eq!(request_deserialized.body.body.ab_group_add_request.group_add_options.f_rename_on_msgr_conflict, Some(false));
        assert_eq!(request_deserialized.body.body.ab_group_add_request.group_info.group_info.group_type, Some(String::from("C8529CE2-6EAD-434d-881F-341E17DB3FF8")));

        assert_eq!(header.ab_auth_header.ticket_token, String::from("t=0bfusc4t3dT0k3n"));
        assert_eq!(header.application_header.partner_scenario, String::from("Initial"));
    

    }

    #[test]
    fn test_ab_group_add_response() {

        let result = AbgroupAddResultType { guid: String::from("MY_GUID") };
        let group_add_response = AbgroupAddResponse{ ab_group_add_result: Some(result) };

       let body_content = AbgroupAddResponseMessage{ ab_group_add_response: group_add_response };

       let body = SoapAbgroupAddResponseMessage{ body: body_content, fault: None };
       let response = AbgroupAddResponseMessageSoapEnvelope{ header: None, body: body  };

       let response_serialized = to_string(&response).unwrap();
       assert!(response_serialized.contains("MY_GUID"));
       assert!(response_serialized.contains("ABGroupAddResponse"));
       assert!(response_serialized.contains("ABGroupAddResult"));
       assert!(response_serialized.contains("guid"));
    }

    #[test]
    fn test_update_dynamic_item_request() {
        let request = "<?xml version=\"1.0\" encoding=\"utf-8\"?><soap:Envelope xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org2001/XMLSchema\" xmlns:soapenc=\"http://schemas.xmlsoap.org/soap/encoding/\"><soap:Header><ABApplicationHeader xmlns=\"http://www.msn.com/webservices/AddressBook\"><ApplicationId>AAD9B99B-58E6-4F23-B975-D9EC1F9EC24A</ApplicationId><IsMigration>false</IsMigration><PartnerScenario>RoamingIdentityChanged</PartnerScenario><CacheKey>12r1:8nBBE6vX1J4uPKajtbem5XBIblimCwAhIziAeEAwYD0AMiaztryWvcZthkN9oX_pl2scBKXfKvRvuWKYdHUNuRkgiyV9rzcDpnDIDiM6vdcEB6d82wjjnL4TAFAjc5X8i-C9mNfQujUk470Pfz9qbWfK6ANcEtygDb-oWsYVfEBrxl6geTUg9tGT7yCIsls7ECcLyqwsROuAbWCrued_VPKiUgSIvqG8gaA</CacheKey></ABApplicationHeader><ABAuthHeader xmlns=\"http://www.msn.com/webservices/AddressBook\"><ManagedGroupRequest>false</ManagedGroupRequest><TicketToken>t=0bfusc4t3dT0k3n</TicketToken></ABAuthHeader></soap:Header><soap:Body><UpdateDynamicItem xmlns=\"http://www.msn.com/webservices/AddressBook\"><abId>00000000-0000-0000-0000-000000000000</abId><dynamicItems><DynamicItem xsi:type=\"PassportDynamicItem\"><Type>Passport</Type><PassportName>aeoncl@matrix.org</PassportName><Notifications><NotificationData><StoreService><Info><Handle><Id>0</Id><Type>Profile</Type><ForeignId>MyProfile</ForeignId></Handle><InverseRequired>false</InverseRequired><IsBot>false</IsBot></Info><Changes /><LastChange>0001-01-01T00:00:00</LastChange><Deleted>false</Deleted></StoreService><Status>Exist Access</Status><Gleam>false</Gleam><InstanceId>0</InstanceId></NotificationData></Notifications><Changes>Notifications</Changes></DynamicItem></dynamicItems></UpdateDynamicItem></soap:Body></soap:Envelope>";

        let request_deserialized : UpdateDynamicItemMessageSoapEnvelope = from_str(request).unwrap();

    }

    #[test]
    fn test_update_dynamic_item_response() {
        let body_content = UpdateDynamicItemResponseMessage{ update_dynamic_item_response: None };
        let body = SoapUpdateDynamicItemResponseMessage { body: body_content, fault: None };
        let response =  UpdateDynamicItemResponseMessageSoapEnvelope{ header: None, body: body };
        let response_serialized = to_string(&response).unwrap();

    }


}
