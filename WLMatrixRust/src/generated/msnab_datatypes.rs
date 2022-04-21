//! THIS IS A GENERATED FILE!
//! Take care when hand editing. Changes will be lost during subsequent runs of the code generator.
//!
//! version: 0.1.5
//!

            #![allow(dead_code)]           
            #![allow(unused_imports)]
            use yaserde_derive::{YaSerialize, YaDeserialize};
            use std::io::{Read, Write};
            use log::{warn, debug};
            
            pub const SOAP_ENCODING: &str = "http://www.w3.org/2003/05/soap-encoding";
            #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
pub struct Header {
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "Fault",
	namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
	prefix = "soap",
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
use yaserde::{YaSerialize, YaDeserialize};
            use yaserde::de::from_str;
            use async_trait::async_trait;
            use yaserde::ser::to_string;
            use super::*;
            }

pub mod types {
use yaserde::{YaSerialize, YaDeserialize};
            use yaserde::de::from_str;
            use async_trait::async_trait;
            use yaserde::ser::to_string;
            use super::*;
            #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ArrayOfAnnotation", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ArrayOfAnnotation {
	#[yaserde(rename = "Annotation", prefix = "nsi1")]
	pub annotation: Vec<Annotation>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ArrayOfGuid", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ArrayOfGuid {
	#[yaserde(rename = "guid", prefix = "nsi1")]
	pub guid: Vec<Guid>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ArrayOfContactType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)] 
pub struct ArrayOfContactType {
	#[yaserde(rename = "Contact", prefix = "nsi1")]
	pub contact: Vec<ContactType>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ArrayOfcontactEmailType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ArrayOfcontactEmailType {
	#[yaserde(rename = "ContactEmail", prefix = "nsi1")]
	pub contact_email: Vec<ContactEmailType>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ArrayOfcontactPhoneType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ArrayOfcontactPhoneType {
	#[yaserde(rename = "ContactPhone", prefix = "nsi1")]
	pub contact_phone: Vec<ContactPhoneType>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ArrayOfcontactLocationType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ArrayOfcontactLocationType {
	#[yaserde(rename = "ContactLocation", prefix = "nsi1")]
	pub contact_location: Vec<ContactLocationType>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ArrayOfcontactWebSiteType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ArrayOfcontactWebSiteType {
	#[yaserde(rename = "ContactWebSite", prefix = "nsi1")]
	pub contact_web_site: Vec<ContactWebSiteType>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ArrayOfHandleType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ArrayOfHandleType {
	#[yaserde(rename = "ServiceHandle", prefix = "nsi1")]
	pub service_handle: Vec<HandleType>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ArrayOfServiceName", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ArrayOfServiceName {
	#[yaserde(rename = "ServiceType", prefix = "nsi1")]
	pub service_type: Vec<ServiceName>, 
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
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ArrayOfRoleId", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ArrayOfRoleId {
	#[yaserde(rename = "RoleId", prefix = "nsi1")]
	pub role_id: Vec<RoleId>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "abInfoType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct AbInfoType {
	#[yaserde(rename = "MigratedTo", prefix = "nsi1")]
	pub migrated_to: i16, 
	#[yaserde(rename = "BetaStatus", prefix = "nsi1")]
	pub beta_status: Option<String>, 
	#[yaserde(rename = "name", prefix = "nsi1")]
	pub name: Option<String>, 
	#[yaserde(rename = "ownerPuid", prefix = "nsi1")]
	pub owner_puid: i64, 
	#[yaserde(rename = "OwnerCID", prefix = "nsi1")]
	pub owner_cid: i64, 
	#[yaserde(rename = "ownerEmail", prefix = "nsi1")]
	pub owner_email: Option<String>, 
	#[yaserde(rename = "fDefault", prefix = "nsi1")]
	pub f_default: bool, 
	#[yaserde(rename = "joinedNamespace", prefix = "nsi1")]
	pub joined_namespace: bool, 
	#[yaserde(rename = "IsBot", prefix = "nsi1")]
	pub is_bot: bool, 
	#[yaserde(rename = "IsParentManaged", prefix = "nsi1")]
	pub is_parent_managed: bool, 
	#[yaserde(rename = "AccountTier", prefix = "nsi1")]
	pub account_tier: Option<String>, 
	#[yaserde(rename = "AccountTierLastChanged", prefix = "nsi1")]
	pub account_tier_last_changed: String, 
	#[yaserde(rename = "ProfileVersion", prefix = "nsi1")]
	pub profile_version: i32, 
	#[yaserde(rename = "SubscribeExternalPartner", prefix = "nsi1")]
	pub subscribe_external_partner: bool, 
	#[yaserde(rename = "NotifyExternalPartner", prefix = "nsi1")]
	pub notify_external_partner: bool, 
	#[yaserde(rename = "AddressBookType", prefix = "nsi1")]
	pub address_book_type: String, 
	#[yaserde(rename = "MessengerApplicationServiceCreated", prefix = "nsi1")]
	pub messenger_application_service_created: bool, 
	#[yaserde(rename = "IsBetaMigrated", prefix = "nsi1")]
	pub is_beta_migrated: bool, 
	#[yaserde(rename = "LastRelevanceUpdate", prefix = "nsi1")]
	pub last_relevance_update: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "HandleType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct HandleType {
	#[yaserde(rename = "Id", prefix = "nsi1")]
	pub id: i32, 
	#[yaserde(rename = "Type", prefix = "nsi1")]
	pub rs_type: ServiceName, 
	#[yaserde(rename = "ForeignId", prefix = "nsi1")]
	pub foreign_id: Option<String>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ServiceName", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ServiceName {
	#[yaserde(text, prefix = "nsi1")]
	pub body: String, 
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
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ServiceFilter", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ServiceFilter {
	#[yaserde(rename = "Types", prefix = "nsi1")]
	pub types: Option<ArrayOfServiceName>, 
	#[yaserde(rename = "Handles", prefix = "nsi1")]
	pub handles: Option<ArrayOfHandleType>, 
	#[yaserde(rename = "LastChange", prefix = "nsi1")]
	pub last_change: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "Location", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct Location {
	#[yaserde(rename = "Id", prefix = "nsi1")]
	pub id: Guid, 
	#[yaserde(rename = "IsPassportNameHidden", prefix = "nsi1")]
	pub is_passport_name_hidden: bool, 
	#[yaserde(rename = "CID", prefix = "nsi1")]
	pub cid: i64, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "BaseMember", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct BaseMember {

	#[yaserde(rename = "MembershipId",
	prefix = "nsi1")]
	pub membership_id: Option<i32>, 
	#[yaserde(rename = "Type", prefix = "nsi1")]
	pub rs_type: String, 
	#[yaserde(rename = "Location", prefix = "nsi1")]
	pub location: Option<Location>, 
	#[yaserde(rename = "DisplayName", prefix = "nsi1")]
	pub display_name: Option<String>, 
	#[yaserde(rename = "State", prefix = "nsi1")]
	pub state: MemberState, 
	#[yaserde(rename = "NewRole", prefix = "nsi1")]
	pub new_role: Option<RoleId>, 
	#[yaserde(rename = "Annotations", prefix = "nsi1")]
	pub annotations: Option<ArrayOfAnnotation>, 
	#[yaserde(rename = "Deleted", prefix = "nsi1")]
	pub deleted: Option<bool>, 
	#[yaserde(rename = "LastChanged", prefix = "nsi1")]
	pub last_changed: Option<String>, 
	#[yaserde(rename = "JoinedDate", prefix = "nsi1")]
	pub joined_date: Option<String>, 
	#[yaserde(rename = "ExpirationDate", prefix = "nsi1")]
	pub expiration_date: Option<String>, 
	#[yaserde(rename = "Changes", prefix = "nsi1")]
	pub changes: Option<String>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "CircleMember", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct CircleMember {
	#[yaserde(flatten, default)]
	pub base_member: BaseMember, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "CircleId", prefix = "nsi1")]
	pub circle_id: Guid, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "PassportMember", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct PassportMember {
	#[yaserde(flatten, default)]
	pub base_member: BaseMember, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "PassportName", prefix = "nsi1")]
	pub passport_name: String, 
	#[yaserde(rename = "IsPassportNameHidden", prefix = "nsi1")]
	pub is_passport_name_hidden: Option<bool>, 
	#[yaserde(rename = "PassportId", prefix = "nsi1")]
	pub passport_id: Option<i32>, 
	#[yaserde(rename = "CID", prefix = "nsi1")]
	pub cid: Option<i64>, 
	#[yaserde(rename = "PassportChanges", prefix = "nsi1")]
	pub passport_changes: Option<String>, 
	#[yaserde(rename = "LookedupByCID", prefix = "nsi1")]
	pub lookedup_by_cid: Option<bool>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "EmailMember", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct EmailMember {
	#[yaserde(flatten, prefix = "nsi1")]
	pub base_member: BaseMember, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "Email", prefix = "nsi1")]
	pub email: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "PhoneMember", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct PhoneMember {
	#[yaserde(flatten, prefix = "nsi1")]
	pub base_member: BaseMember, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "PhoneNumber", prefix = "nsi1")]
	pub phone_number: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "RoleMember", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct RoleMember {
	#[yaserde(flatten, prefix = "nsi1")]
	pub base_member: BaseMember, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "Id", prefix = "nsi1")]
	pub id: RoleId, 
	#[yaserde(rename = "DefiningService", prefix = "nsi1")]
	pub defining_service: Option<HandleType>, 
	#[yaserde(rename = "MaxRoleRecursionDepth", prefix = "nsi1")]
	pub max_role_recursion_depth: i32, 
	#[yaserde(rename = "MaxDegreesSeparation", prefix = "nsi1")]
	pub max_degrees_separation: i32, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ServiceMember", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ServiceMember {
	#[yaserde(flatten, prefix = "nsi1")]
	pub base_member: BaseMember, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "Service", prefix = "nsi1")]
	pub service: HandleType, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "DomainMember", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct DomainMember {
	#[yaserde(flatten, prefix = "nsi1")]
	pub base_member: BaseMember, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "DomainName", prefix = "nsi1")]
	pub domain_name: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "EveryoneMember", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct EveryoneMember {
	#[yaserde(flatten, prefix = "nsi1")]
	pub base_member: BaseMember, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "PartnerMember", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct PartnerMember {
	#[yaserde(flatten, prefix = "nsi1")]
	pub base_member: BaseMember, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "AppId", prefix = "nsi1")]
	pub app_id: i64, 
	#[yaserde(rename = "Scope", prefix = "nsi1")]
	pub scope: i32, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "GroupMember", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct GroupMember {
	#[yaserde(flatten, prefix = "nsi1")]
	pub base_member: BaseMember, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "Id", prefix = "nsi1")]
	pub id: Guid, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "GuidMember", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct GuidMember {
	#[yaserde(flatten, prefix = "nsi1")]
	pub base_member: BaseMember, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "Id", prefix = "nsi1")]
	pub id: Guid, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ExternalIDMember", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ExternalIDMember {
	#[yaserde(flatten, prefix = "nsi1")]
	pub base_member: BaseMember, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "SourceID", prefix = "nsi1")]
	pub source_id: String, 
	#[yaserde(rename = "ObjectID", prefix = "nsi1")]
	pub object_id: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "Guid", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct Guid {
	#[yaserde(text, prefix = "nsi1")]
	pub body: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "RoleId",namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct RoleId {
	#[yaserde(text, prefix = "nsi1")]
	pub body: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "MemberState", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct MemberState {
	#[yaserde(text, default)]
	pub body: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "Annotation", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct Annotation {
	#[yaserde(rename = "Name", prefix = "nsi1")]
	pub name: String, 
	#[yaserde(rename = "Value", prefix = "nsi1")]
	pub value: Option<String>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ContactType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ContactType {
	#[yaserde(rename = "contactId", prefix = "nsi1")]
	pub contact_id: Option<Guid>, 
	#[yaserde(rename = "contactInfo", prefix = "nsi1")]
	pub contact_info: Option<ContactInfoType>, 
	#[yaserde(rename = "propertiesChanged", prefix = "nsi1")]
	pub properties_changed: Option<String>, 
	#[yaserde(rename = "fDeleted", prefix = "nsi1")]
	pub f_deleted: Option<bool>, 
	#[yaserde(rename = "lastChange", prefix = "nsi1")]
	pub last_change: Option<String>, 
	#[yaserde(rename = "CreateDate", prefix = "nsi1")]
	pub create_date: Option<String>, 
	#[yaserde(rename = "LastModifiedBy", prefix = "nsi1")]
	pub last_modified_by: Option<i32>, 
	#[yaserde(rename = "CreatedBy", prefix = "nsi1")]
	pub created_by: Option<i32>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ContactIdType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ContactIdType {
	#[yaserde(rename = "contactId", prefix = "nsi1")]
	pub contact_id: Option<Guid>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "NetworkInfoList", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct NetworkInfoList {
	#[yaserde(rename = "NetworkInfo", prefix = "nsi1")]
	pub network_info: Vec<NetworkInfoType>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "URLs", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct Urls {
	#[yaserde(rename = "ContactURL", prefix = "nsi1")]
	pub contact_url: Vec<ContactURLType>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "contactInfoType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ContactInfoType {
	#[yaserde(rename = "emails", prefix = "nsi1")]
	pub emails: Option<ArrayOfcontactEmailType>, 
	#[yaserde(rename = "phones", prefix = "nsi1")]
	pub phones: Option<ArrayOfcontactPhoneType>, 
	#[yaserde(rename = "locations", prefix = "nsi1")]
	pub locations: Option<ArrayOfcontactLocationType>, 
	#[yaserde(rename = "webSites", prefix = "nsi1")]
	pub web_sites: Option<ArrayOfcontactWebSiteType>, 
	#[yaserde(rename = "annotations", prefix = "nsi1")]
	pub annotations: Option<ArrayOfAnnotation>, 
	#[yaserde(rename = "groupIds", prefix = "nsi1")]
	pub group_ids: Option<ArrayOfGuid>, 
	#[yaserde(rename = "groupIdsDeleted", prefix = "nsi1")]
	pub group_ids_deleted: Option<ArrayOfGuid>, 
	#[yaserde(rename = "contactType", prefix = "nsi1")]
	pub contact_type: Option<String>, 
	#[yaserde(rename = "quickName", prefix = "nsi1")]
	pub quick_name: Option<String>, 
	#[yaserde(rename = "firstName", prefix = "nsi1")]
	pub first_name: Option<String>, 
	#[yaserde(rename = "MiddleName", prefix = "nsi1")]
	pub middle_name: Option<String>, 
	#[yaserde(rename = "lastName", prefix = "nsi1")]
	pub last_name: Option<String>, 
	#[yaserde(rename = "Suffix", prefix = "nsi1")]
	pub suffix: Option<String>, 
	#[yaserde(rename = "NameTitle", prefix = "nsi1")]
	pub name_title: Option<String>, 
	#[yaserde(rename = "passportName", prefix = "nsi1")]
	pub passport_name: Option<String>, 
	#[yaserde(rename = "displayName", prefix = "nsi1")]
	pub display_name: Option<String>, 
	#[yaserde(rename = "puid", prefix = "nsi1")]
	pub puid: Option<i64>, 
	#[yaserde(rename = "CID", prefix = "nsi1")]
	pub cid: Option<i64>, 
	#[yaserde(rename = "BrandIdList", prefix = "nsi1")]
	pub brand_id_list: Option<String>, 
	#[yaserde(rename = "comment", prefix = "nsi1")]
	pub comment: Option<String>, 
	#[yaserde(rename = "isMobileIMEnabled", prefix = "nsi1")]
	pub is_mobile_im_enabled: Option<bool>, 
	#[yaserde(rename = "isMessengerUser", prefix = "nsi1")]
	pub is_messenger_user: Option<bool>, 
	#[yaserde(rename = "isFavorite", prefix = "nsi1")]
	pub is_favorite: Option<bool>, 
	#[yaserde(rename = "isSmtp", prefix = "nsi1")]
	pub is_smtp: Option<bool>, 
	#[yaserde(rename = "hasSpace", prefix = "nsi1")]
	pub has_space: Option<bool>, 
	#[yaserde(rename = "spotWatchState", prefix = "nsi1")]
	pub spot_watch_state: Option<String>, 
	#[yaserde(rename = "birthdate", prefix = "nsi1")]
	pub birthdate: Option<String>, 
	#[yaserde(rename = "primaryEmailType", prefix = "nsi1")]
	pub primary_email_type: Option<ContactEmailTypeType>, 
	#[yaserde(rename = "PrimaryLocation", prefix = "nsi1")]
	pub primary_location: Option<ContactLocationTypeType>, 
	#[yaserde(rename = "PrimaryPhone", prefix = "nsi1")]
	pub primary_phone: Option<String>, 
	#[yaserde(rename = "IsPrivate", prefix = "nsi1")]
	pub is_private: Option<bool>, 
	#[yaserde(rename = "Anniversary", prefix = "nsi1")]
	pub anniversary: Option<String>, 
	#[yaserde(rename = "Gender", prefix = "nsi1")]
	pub gender: Option<String>, 
	#[yaserde(rename = "TimeZone", prefix = "nsi1")]
	pub time_zone: Option<String>, 
	#[yaserde(rename = "TrustLevel", prefix = "nsi1")]
	pub trust_level: Option<i32>, 
	#[yaserde(rename = "NetworkInfoList", prefix = "nsi1")]
	pub network_info_list: Option<NetworkInfoList>, 
	#[yaserde(rename = "PublicDisplayName", prefix = "nsi1")]
	pub public_display_name: Option<String>, 
	#[yaserde(rename = "IsAutoUpdateDisabled", prefix = "nsi1")]
	pub is_auto_update_disabled: Option<bool>, 
	#[yaserde(rename = "IsHidden", prefix = "nsi1")]
	pub is_hidden: Option<bool>, 
	#[yaserde(rename = "IsPassportNameHidden", prefix = "nsi1")]
	pub is_passport_name_hidden: Option<bool>, 
	#[yaserde(rename = "IsNotMobileVisible", prefix = "nsi1")]
	pub is_not_mobile_visible: Option<bool>, 
	#[yaserde(rename = "IsShellContact", prefix = "nsi1")]
	pub is_shell_contact: Option<bool>, 
	#[yaserde(rename = "MessengerMemberInfo", prefix = "nsi1")]
	pub messenger_member_info: Option<MessengerMemberInfo>, 
	#[yaserde(rename = "PropertiesChanged", prefix = "nsi1")]
	pub properties_changed: Option<String>, 
	#[yaserde(rename = "clientErrorData", prefix = "nsi1")]
	pub client_error_data: Option<String>, 
	#[yaserde(rename = "LinkInfo", prefix = "nsi1")]
	pub link_info: Option<LinkInfoType>, 
	#[yaserde(rename = "SourceHandle", prefix = "nsi1")]
	pub source_handle: Option<SourceHandleType>, 
	#[yaserde(rename = "FileAs", prefix = "nsi1")]
	pub file_as: Option<String>, 
	#[yaserde(rename = "URLs", prefix = "nsi1")]
	pub ur_ls: Option<Urls>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "contactEmailType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ContactEmailType {
	#[yaserde(rename = "contactEmailType", prefix = "nsi1")]
	pub contact_email_type: ContactEmailTypeType, 
	#[yaserde(rename = "email", prefix = "nsi1")]
	pub email: String, 
	#[yaserde(rename = "isMessengerEnabled", prefix = "nsi1")]
	pub is_messenger_enabled: bool, 
	#[yaserde(rename = "Capability", prefix = "nsi1")]
	pub capability: i32, 
	#[yaserde(rename = "MessengerEnabledExternally", prefix = "nsi1")]
	pub messenger_enabled_externally: bool, 
	#[yaserde(rename = "propertiesChanged", prefix = "nsi1")]
	pub properties_changed: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ContactEmailTypeType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ContactEmailTypeType {
	#[yaserde(text, prefix = "nsi1")]
	pub body: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "contactPhoneType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ContactPhoneType {
	#[yaserde(rename = "contactPhoneType", prefix = "nsi1")]
	pub contact_phone_type: String, 
	#[yaserde(rename = "number", prefix = "nsi1")]
	pub number: String, 
	#[yaserde(rename = "isMessengerEnabled", prefix = "nsi1")]
	pub is_messenger_enabled: bool, 
	#[yaserde(rename = "propertiesChanged", prefix = "nsi1")]
	pub properties_changed: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "contactLocationType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ContactLocationType {
	#[yaserde(rename = "contactLocationType", prefix = "nsi1")]
	pub contact_location_type: String, 
	#[yaserde(rename = "name", prefix = "nsi1")]
	pub name: Option<String>, 
	#[yaserde(rename = "street", prefix = "nsi1")]
	pub street: Option<String>, 
	#[yaserde(rename = "city", prefix = "nsi1")]
	pub city: Option<String>, 
	#[yaserde(rename = "state", prefix = "nsi1")]
	pub state: Option<String>, 
	#[yaserde(rename = "country", prefix = "nsi1")]
	pub country: Option<String>, 
	#[yaserde(rename = "postalCode", prefix = "nsi1")]
	pub postal_code: Option<String>, 
	#[yaserde(rename = "Department", prefix = "nsi1")]
	pub department: Option<String>, 
	#[yaserde(rename = "Changes", prefix = "nsi1")]
	pub changes: Option<String>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ContactLocationTypeType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ContactLocationTypeType {
	#[yaserde(text, prefix = "nsi1")]
	pub body: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "contactWebSiteType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ContactWebSiteType {
	#[yaserde(rename = "contactWebSiteType", prefix = "nsi1")]
	pub contact_web_site_type: ContactWebSiteTypeType, 
	#[yaserde(rename = "webURL", prefix = "nsi1")]
	pub web_url: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ContactWebSiteTypeType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ContactWebSiteTypeType {
	#[yaserde(text, prefix = "nsi1")]
	pub body: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "GroupType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct GroupType {
	#[yaserde(rename = "groupId", prefix = "nsi1")]
	pub group_id: Guid, 
	#[yaserde(rename = "groupInfo", prefix = "nsi1")]
	pub group_info: GroupInfoType, 
	#[yaserde(rename = "propertiesChanged", prefix = "nsi1")]
	pub properties_changed: String, 
	#[yaserde(rename = "fDeleted", prefix = "nsi1")]
	pub f_deleted: Option<bool>, 
	#[yaserde(rename = "lastChange", prefix = "nsi1")]
	pub last_change: Option<String>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "groupInfoType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct GroupInfoType {
	#[yaserde(rename = "annotations", prefix = "nsi1")]
	pub annotations: Option<ArrayOfAnnotation>, 
	#[yaserde(rename = "groupType", prefix = "nsi1")]
	pub group_type: Option<Guid>, 
	#[yaserde(rename = "name", prefix = "nsi1")]
	pub name: Option<String>, 
	#[yaserde(rename = "IsNotMobileVisible", prefix = "nsi1")]
	pub is_not_mobile_visible: Option<bool>, 
	#[yaserde(rename = "IsPrivate", prefix = "nsi1")]
	pub is_private: Option<bool>, 
	#[yaserde(rename = "IsFavorite", prefix = "nsi1")]
	pub is_favorite: Option<bool>, 
	#[yaserde(rename = "fMessenger", prefix = "nsi1")]
	pub f_messenger: Option<bool>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "groupFilterType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct GroupFilterType {
	#[yaserde(rename = "groupIds", prefix = "nsi1")]
	pub group_ids: Option<ArrayOfGuid>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "additionalDetails", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct AdditionalDetails {
	#[yaserde(rename = "originalExceptionErrorMessage", prefix = "nsi1")]
	pub original_exception_error_message: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "InvalidPassportUser",
	namespace = "tns: http://www.msn.com/webservices/AddressBook",
	namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
	prefix = "tns",
)]
pub struct InvalidPassportUser {
	#[yaserde(rename = "errorcode", prefix = "nsi1")]
	pub errorcode: String, 
	#[yaserde(rename = "errorstring", prefix = "nsi1")]
	pub errorstring: String, 
	#[yaserde(rename = "additionalDetails", prefix = "nsi1")]
	pub additional_details: AdditionalDetails, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "MessengerMemberInfo", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct MessengerMemberInfo {
	#[yaserde(rename = "PendingAnnotations", prefix = "nsi1")]
	pub pending_annotations: Option<ArrayOfAnnotation>, 
	#[yaserde(rename = "DisplayName", prefix = "nsi1")]
	pub display_name: Option<String>, 
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
	rename = "NotificationDataType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct NotificationDataType {
	#[yaserde(rename = "StoreService", prefix = "nsi1")]
	pub store_service: ServiceType, 
	#[yaserde(rename = "Status", prefix = "nsi1")]
	pub status: String, 
	#[yaserde(rename = "LastChanged", prefix = "nsi1")]
	pub last_changed: String, 
	#[yaserde(rename = "Gleam", prefix = "nsi1")]
	pub gleam: bool, 
	#[yaserde(rename = "InstanceId", prefix = "nsi1")]
	pub instance_id: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "Notifications", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct Notifications {
	#[yaserde(rename = "NotificationData", prefix = "nsi1")]
	pub notification_data: Vec<NotificationDataType>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "BaseDynamicItemType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct BaseDynamicItemType {
	#[yaserde(rename = "Type", prefix = "nsi1")]
	pub rs_type: String, 
	#[yaserde(rename = "Deleted", prefix = "nsi1")]
	pub deleted: Option<bool>, 
	#[yaserde(rename = "LastChanged", prefix = "nsi1")]
	pub last_changed: Option<String>, 
	#[yaserde(rename = "Notifications", prefix = "nsi1")]
	pub notifications: Option<Notifications>, 
	#[yaserde(rename = "Changes", prefix = "nsi1")]
	pub changes: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "CircleDynamicItem", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct CircleDynamicItem {
	#[yaserde(flatten, prefix = "nsi1")]
	pub base_dynamic_item_type: BaseDynamicItemType, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "Id", prefix = "nsi1")]
	pub id: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "PassportDynamicItem", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct PassportDynamicItem {
	#[yaserde(flatten, prefix = "nsi1")]
	pub base_dynamic_item_type: BaseDynamicItemType, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "CID", prefix = "nsi1")]
	pub cid: Option<String>, 
	#[yaserde(rename = "PassportName", prefix = "nsi1")]
	pub passport_name: String, 
	#[yaserde(rename = "PassportId", prefix = "nsi1")]
	pub passport_id: Option<String>, 
	#[yaserde(rename = "SpaceStatus", prefix = "nsi1")]
	pub space_status: String, 
	#[yaserde(rename = "SpaceLastChanged", prefix = "nsi1")]
	pub space_last_changed: Option<String>, 
	#[yaserde(rename = "SpaceLastViewed", prefix = "nsi1")]
	pub space_last_viewed: Option<String>, 
	#[yaserde(rename = "SpaceGleam", prefix = "nsi1")]
	pub space_gleam: Option<bool>, 
	#[yaserde(rename = "ProfileLastChanged", prefix = "nsi1")]
	pub profile_last_changed: Option<String>, 
	#[yaserde(rename = "ProfileLastView", prefix = "nsi1")]
	pub profile_last_view: Option<String>, 
	#[yaserde(rename = "ProfileStatus", prefix = "nsi1")]
	pub profile_status: String, 
	#[yaserde(rename = "ProfileGleam", prefix = "nsi1")]
	pub profile_gleam: Option<bool>, 
	#[yaserde(rename = "ContactProfileStatus", prefix = "nsi1")]
	pub contact_profile_status: String, 
	#[yaserde(rename = "ContactProfileLastChanged", prefix = "nsi1")]
	pub contact_profile_last_changed: Option<String>, 
	#[yaserde(rename = "ContactProfileLastViewed", prefix = "nsi1")]
	pub contact_profile_last_viewed: Option<String>, 
	#[yaserde(rename = "LiveContactLastChanged", prefix = "nsi1")]
	pub live_contact_last_changed: Option<String>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "abType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct AbType {
	#[yaserde(rename = "abId", prefix = "nsi1")]
	pub ab_id: Guid, 
	#[yaserde(rename = "abInfo", prefix = "nsi1")]
	pub ab_info: AbInfoType, 
	#[yaserde(rename = "lastChange", prefix = "nsi1")]
	pub last_change: String, 
	#[yaserde(rename = "DynamicItemLastChanged", prefix = "nsi1")]
	pub dynamic_item_last_changed: String, 
	#[yaserde(rename = "RecentActivityItemLastChanged", prefix = "nsi1")]
	pub recent_activity_item_last_changed: String, 
	#[yaserde(rename = "createDate", prefix = "nsi1")]
	pub create_date: String, 
	#[yaserde(rename = "propertiesChanged", prefix = "nsi1")]
	pub properties_changed: Option<String>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "Circles", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct Circles {
	#[yaserde(rename = "CircleInverseInfo", prefix = "nsi1")]
	pub circle_inverse_info: Vec<CircleInverseInfoType>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "CircleResultType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct CircleResultType {
	#[yaserde(rename = "Circles", prefix = "nsi1")]
	pub circles: Option<Circles>, 
	#[yaserde(rename = "CircleTicket", prefix = "nsi1")]
	pub circle_ticket: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "NetworkInfoType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct NetworkInfoType {
	#[yaserde(rename = "Annotations", prefix = "nsi1")]
	pub annotations: Option<ArrayOfAnnotation>, 
	#[yaserde(rename = "DomainId", prefix = "nsi1")]
	pub domain_id: i32, 
	#[yaserde(rename = "SourceId", prefix = "nsi1")]
	pub source_id: Option<String>, 
	#[yaserde(rename = "DomainTag", prefix = "nsi1")]
	pub domain_tag: Option<String>, 
	#[yaserde(rename = "UserTileURL", prefix = "nsi1")]
	pub user_tile_url: Option<String>, 
	#[yaserde(rename = "ProfileURL", prefix = "nsi1")]
	pub profile_url: Option<String>, 
	#[yaserde(rename = "DisplayName", prefix = "nsi1")]
	pub display_name: Option<String>, 
	#[yaserde(rename = "RelationshipType", prefix = "nsi1")]
	pub relationship_type: i32, 
	#[yaserde(rename = "RelationshipState", prefix = "nsi1")]
	pub relationship_state: i32, 
	#[yaserde(rename = "RelationshipStateDate", prefix = "nsi1")]
	pub relationship_state_date: String, 
	#[yaserde(rename = "RelationshipRole", prefix = "nsi1")]
	pub relationship_role: i64, 
	#[yaserde(rename = "ExtendedData", prefix = "nsi1")]
	pub extended_data: Option<String>, 
	#[yaserde(rename = "NDRCount", prefix = "nsi1")]
	pub ndr_count: i32, 
	#[yaserde(rename = "InviterMessage", prefix = "nsi1")]
	pub inviter_message: Option<String>, 
	#[yaserde(rename = "InviterCID", prefix = "nsi1")]
	pub inviter_cid: i64, 
	#[yaserde(rename = "InviterName", prefix = "nsi1")]
	pub inviter_name: Option<String>, 
	#[yaserde(rename = "InviterEmail", prefix = "nsi1")]
	pub inviter_email: Option<String>, 
	#[yaserde(rename = "CreateDate", prefix = "nsi1")]
	pub create_date: String, 
	#[yaserde(rename = "LastChanged", prefix = "nsi1")]
	pub last_changed: String, 
	#[yaserde(rename = "PropertiesChanged", prefix = "nsi1")]
	pub properties_changed: String, 
	#[yaserde(rename = "ForwardingEmail", prefix = "nsi1")]
	pub forwarding_email: Option<String>, 
	#[yaserde(rename = "Settings", prefix = "nsi1")]
	pub settings: i32, 
	#[yaserde(rename = "AccountName", prefix = "nsi1")]
	pub account_name: Option<String>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ContactFilterType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ContactFilterType {
	#[yaserde(rename = "IncludeHiddenContacts", prefix = "nsi1")]
	pub include_hidden_contacts: bool, 
	#[yaserde(rename = "IncludeShellContacts", prefix = "nsi1")]
	pub include_shell_contacts: Option<bool>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "filterOptionsType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct FilterOptionsType {
	#[yaserde(rename = "DeltasOnly", prefix = "nsi1")]
	pub deltas_only: bool, 
	#[yaserde(rename = "LastChanged", prefix = "nsi1")]
	pub last_changed: Option<String>, 
	#[yaserde(rename = "ContactFilter", prefix = "nsi1")]
	pub contact_filter: ContactFilterType, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "entityHandle", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct EntityHandle {
	#[yaserde(rename = "Cid", prefix = "nsi1")]
	pub cid: i64, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "NotationType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct NotationType {
	#[yaserde(rename = "Name", prefix = "nsi1")]
	pub name: String, 
	#[yaserde(rename = "Value", prefix = "nsi1")]
	pub value: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "Values", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct Values {
	#[yaserde(rename = "Value", prefix = "nsi1")]
	pub value: Vec<SimpleTemplateVariableBaseType>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ListTemplateVariableItemType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ListTemplateVariableItemType {
	#[yaserde(rename = "Values", prefix = "nsi1")]
	pub values: Values, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "TemplateVariableBaseType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct TemplateVariableBaseType {
	#[yaserde(rename = "Name", prefix = "nsi1")]
	pub name: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "SimpleTemplateVariableBaseType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct SimpleTemplateVariableBaseType {
	#[yaserde(flatten, prefix = "nsi1")]
	pub template_variable_base_type: TemplateVariableBaseType, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "Value", prefix = "nsi1")]
	pub value: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "PublisherIdTemplateVariable", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct PublisherIdTemplateVariable {
	#[yaserde(flatten, prefix = "nsi1")]
	pub template_variable_base_type: TemplateVariableBaseType, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "Id", prefix = "nsi1")]
	pub id: String, 
	#[yaserde(rename = "NameHint", prefix = "nsi1")]
	pub name_hint: Option<String>, 
	#[yaserde(rename = "LastNameHint", prefix = "nsi1")]
	pub last_name_hint: Option<String>, 
	#[yaserde(rename = "IsFavorite", prefix = "nsi1")]
	pub is_favorite: Option<bool>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "TargetIdTemplateVariable", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct TargetIdTemplateVariable {
	#[yaserde(flatten, prefix = "nsi1")]
	pub publisher_id_template_variable: PublisherIdTemplateVariable, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "IdOwner", prefix = "nsi1")]
	pub id_owner: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "SafeLinkDetailsType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct SafeLinkDetailsType {
	#[yaserde(rename = "Offset", prefix = "nsi1")]
	pub offset: i32, 
	#[yaserde(rename = "Length", prefix = "nsi1")]
	pub length: i32, 
	#[yaserde(rename = "SafeUrl", prefix = "nsi1")]
	pub safe_url: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "SafeLinks", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct SafeLinks {
	#[yaserde(rename = "SafeLinkDetails", prefix = "nsi1")]
	pub safe_link_details: SafeLinkDetailsType, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "TextTemplateVariable", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct TextTemplateVariable {
	#[yaserde(flatten, prefix = "nsi1")]
	pub simple_template_variable_base_type: SimpleTemplateVariableBaseType, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "SafeLinks", prefix = "nsi1")]
	pub safe_links: Option<SafeLinks>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "Notations", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct Notations {
	#[yaserde(rename = "Notation", prefix = "nsi1")]
	pub notation: Vec<NotationType>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "HlinkTemplateVariable", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct HlinkTemplateVariable {
	#[yaserde(flatten, default)]
	pub simple_template_variable_base_type: SimpleTemplateVariableBaseType, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "Text", prefix = "nsi1")]
	pub text: String, 
	#[yaserde(rename = "Notations", prefix = "nsi1")]
	pub notations: Notations, 
	#[yaserde(rename = "ValueAsSafeLink", prefix = "nsi1")]
	pub value_as_safe_link: Option<String>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "Items", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct Items {
	#[yaserde(rename = "ListTemplateVariableItem", default)]
	pub list_template_variable_item: Vec<ListTemplateVariableItemType>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ListTemplateVariable", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ListTemplateVariable {
	#[yaserde(flatten, default)]
	pub template_variable_base_type: TemplateVariableBaseType, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "Items", prefix = "nsi1")]
	pub items: Items, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ImageTemplateVariable", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ImageTemplateVariable {
	#[yaserde(flatten, default)]
	pub simple_template_variable_base_type: SimpleTemplateVariableBaseType, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "Href", prefix = "nsi1")]
	pub href: String, 
	#[yaserde(rename = "Notations", prefix = "nsi1")]
	pub notations: Notations, 
	#[yaserde(rename = "HrefAsSafeLink", prefix = "nsi1")]
	pub href_as_safe_link: Option<String>, 
	#[yaserde(rename = "AltText", prefix = "nsi1")]
	pub alt_text: Option<String>, 
	#[yaserde(rename = "TargetMediaType", prefix = "nsi1")]
	pub target_media_type: Option<String>, 
	#[yaserde(rename = "TargetMediaSource", prefix = "nsi1")]
	pub target_media_source: Option<String>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "RelevanceInfoType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct RelevanceInfoType {
	#[yaserde(rename = "TimeWeightedScore", prefix = "nsi1")]
	pub time_weighted_score: f64, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "TemplateVariables", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct TemplateVariables {
	#[yaserde(rename = "TemplateVariable", prefix = "nsi1")]
	pub template_variable: Vec<TemplateVariableBaseType>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ActivityDetailsType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ActivityDetailsType {
	#[yaserde(rename = "OwnerCID", prefix = "nsi1")]
	pub owner_cid: String, 
	#[yaserde(rename = "ObjectId", prefix = "nsi1")]
	pub object_id: String, 
	#[yaserde(rename = "ApplicationId", prefix = "nsi1")]
	pub application_id: String, 
	#[yaserde(rename = "ChangeType", prefix = "nsi1")]
	pub change_type: String, 
	#[yaserde(rename = "PublishDate", prefix = "nsi1")]
	pub publish_date: String, 
	#[yaserde(rename = "TemplateVariables", prefix = "nsi1")]
	pub template_variables: TemplateVariables, 
	#[yaserde(rename = "ActivityID", prefix = "nsi1")]
	pub activity_id: Option<String>, 
	#[yaserde(rename = "CanPublishComments", prefix = "nsi1")]
	pub can_publish_comments: Option<bool>, 
	#[yaserde(rename = "VisibilityHint", prefix = "nsi1")]
	pub visibility_hint: Option<i32>, 
	#[yaserde(rename = "RelevanceInfo", prefix = "nsi1")]
	pub relevance_info: Option<RelevanceInfoType>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "RecentActivityTemplateType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct RecentActivityTemplateType {
	#[yaserde(rename = "Cardinality", prefix = "nsi1")]
	pub cardinality: String, 
	#[yaserde(rename = "Data", prefix = "nsi1")]
	pub data: String, 
	#[yaserde(rename = "Title", prefix = "nsi1")]
	pub title: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "RequestedLocalesType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct RequestedLocalesType {
	#[yaserde(rename = "string", prefix = "nsi1")]
	pub string: Vec<String>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "Templates", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct Templates {
	#[yaserde(rename = "RecentActivityTemplate", prefix = "nsi1")]
	pub recent_activity_template: Vec<RecentActivityTemplateType>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "RecentActivityTemplateContainerType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct RecentActivityTemplateContainerType {
	#[yaserde(rename = "ApplicationId", prefix = "nsi1")]
	pub application_id: String, 
	#[yaserde(rename = "ApplicationName", prefix = "nsi1")]
	pub application_name: String, 
	#[yaserde(rename = "ChangeType", prefix = "nsi1")]
	pub change_type: i32, 
	#[yaserde(rename = "Locale", prefix = "nsi1")]
	pub locale: String, 
	#[yaserde(rename = "RequestedLocales", prefix = "nsi1")]
	pub requested_locales: RequestedLocalesType, 
	#[yaserde(rename = "TemplateRevision", prefix = "nsi1")]
	pub template_revision: i32, 
	#[yaserde(rename = "Templates", prefix = "nsi1")]
	pub templates: Templates, 
	#[yaserde(rename = "CollapseCondition", prefix = "nsi1")]
	pub collapse_condition: Option<CollapseConditionType>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "CollapseConditionType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct CollapseConditionType {
	#[yaserde(rename = "string", prefix = "nsi1")]
	pub string: Vec<String>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "CirclePersonalMembershipType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)] 
pub struct CirclePersonalMembershipType {
	#[yaserde(rename = "Role", prefix = "nsi1")]
	pub role: RoleId, 
	#[yaserde(rename = "State", prefix = "nsi1")]
	pub state: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "abHandleType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct AbHandleType {
	#[yaserde(rename = "ABId", prefix = "nsi1")]
	pub ab_id: String, 
	#[yaserde(rename = "Puid", prefix = "nsi1")]
	pub puid: i64, 
	#[yaserde(rename = "Cid", prefix = "nsi1")]
	pub cid: i64, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "contactHandleType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ContactHandleType {
	#[yaserde(rename = "Email", prefix = "nsi1")]
	pub email: String, 
	#[yaserde(rename = "Puid", prefix = "nsi1")]
	pub puid: i64, 
	#[yaserde(rename = "Cid", prefix = "nsi1")]
	pub cid: i64, 
	#[yaserde(rename = "CircleId", prefix = "nsi1")]
	pub circle_id: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "MembershipInfoType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct MembershipInfoType {
	#[yaserde(rename = "CirclePersonalMembership", prefix = "nsi1")]
	pub circle_personal_membership: CirclePersonalMembershipType, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "PersonalInfoType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct PersonalInfoType {
	#[yaserde(rename = "MembershipInfo", prefix = "nsi1")]
	pub membership_info: MembershipInfoType, 
	#[yaserde(rename = "Name", prefix = "nsi1")]
	pub name: String, 
	#[yaserde(rename = "IsNotMobileVisible", prefix = "nsi1")]
	pub is_not_mobile_visible: bool, 
	#[yaserde(rename = "IsFavorite", prefix = "nsi1")]
	pub is_favorite: bool, 
	#[yaserde(rename = "IsFamily", prefix = "nsi1")]
	pub is_family: bool, 
	#[yaserde(rename = "Changes", prefix = "nsi1")]
	pub changes: Option<String>, 
	#[yaserde(rename = "Notes", prefix = "nsi1")]
	pub notes: Option<String>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ContentInfoType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ContentInfoType {
	#[yaserde(rename = "Domain", prefix = "nsi1")]
	pub domain: i32, 
	#[yaserde(rename = "HostedDomain", prefix = "nsi1")]
	pub hosted_domain: String, 
	#[yaserde(rename = "Type", prefix = "nsi1")]
	pub rs_type: i32, 
	#[yaserde(rename = "MembershipAccess", prefix = "nsi1")]
	pub membership_access: i32, 
	#[yaserde(rename = "IsPresenceEnabled", prefix = "nsi1")]
	pub is_presence_enabled: bool, 
	#[yaserde(rename = "RequestMembershipOption", prefix = "nsi1")]
	pub request_membership_option: i32, 
	#[yaserde(rename = "DisplayName", prefix = "nsi1")]
	pub display_name: String, 
	#[yaserde(rename = "ProfileLastUpdated", prefix = "nsi1")]
	pub profile_last_updated: Option<String>, 
	#[yaserde(rename = "Changes", prefix = "nsi1")]
	pub changes: Option<String>, 
	#[yaserde(rename = "CreateDate", prefix = "nsi1")]
	pub create_date: Option<String>, 
	#[yaserde(rename = "LastChanged", prefix = "nsi1")]
	pub last_changed: Option<String>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ContentHandleType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ContentHandleType {
	#[yaserde(rename = "Id", prefix = "nsi1")]
	pub id: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ContentType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ContentType {
	#[yaserde(rename = "Handle", prefix = "nsi1")]
	pub handle: ContentHandleType, 
	#[yaserde(rename = "Info", prefix = "nsi1")]
	pub info: ContentInfoType, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "CircleInverseInfoType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct CircleInverseInfoType {
	#[yaserde(rename = "Content", prefix = "nsi1")]
	pub content: ContentType, 
	#[yaserde(rename = "PersonalInfo", prefix = "nsi1")]
	pub personal_info: PersonalInfoType, 
	#[yaserde(rename = "Deleted", prefix = "nsi1")]
	pub deleted: bool, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "callerInfoType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct CallerInfoType {
	#[yaserde(rename = "PublicDisplayName", prefix = "nsi1")]
	pub public_display_name: String, 
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
	pub cid: i32, 
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
	pub creator_cid: String, 
	#[yaserde(rename = "CreatorPassportName", prefix = "nsi1")]
	pub creator_passport_name: String, 
	#[yaserde(rename = "CircleAttributes", prefix = "nsi1")]
	pub circle_attributes: CircleAttributesType, 
	#[yaserde(rename = "MessengerApplicationServiceCreated", prefix = "nsi1")]
	pub messenger_application_service_created: Option<bool>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "locale", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct Locale {
	#[yaserde(rename = "string", prefix = "nsi1")]
	pub string: Vec<String>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "pageContextType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct PageContextType {
	#[yaserde(rename = "PageSize", prefix = "nsi1")]
	pub page_size: i32, 
	#[yaserde(rename = "Direction", prefix = "nsi1")]
	pub direction: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "LinkInfoType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct LinkInfoType {
	#[yaserde(rename = "PersonID", prefix = "nsi1")]
	pub person_id: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "SourceHandleType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct SourceHandleType {
	#[yaserde(rename = "SourceID", prefix = "nsi1")]
	pub source_id: String, 
	#[yaserde(rename = "ObjectID", prefix = "nsi1")]
	pub object_id: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ContactURLType", namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
	prefix = "nsi1",
	default_namespace="nsi1"
)]
pub struct ContactURLType {
	#[yaserde(rename = "URLId", prefix = "nsi1")]
	pub url_id: i32, 
	#[yaserde(rename = "URLType", prefix = "nsi1")]
	pub url_type: String, 
	#[yaserde(rename = "URLName", prefix = "nsi1")]
	pub url_name: String, 
	#[yaserde(rename = "URL", prefix = "nsi1")]
	pub url: String, 
	#[yaserde(rename = "LastChange", prefix = "nsi1")]
	pub last_change: String, 
	#[yaserde(rename = "Changes", prefix = "nsi1")]
	pub changes: Option<String>, 
}


}

pub mod ports {
use yaserde::{YaSerialize, YaDeserialize};
            use yaserde::de::from_str;
            use async_trait::async_trait;
            use yaserde::ser::to_string;
            use super::*;
            }

pub mod bindings {
use yaserde::{YaSerialize, YaDeserialize};
            use yaserde::de::from_str;
            use async_trait::async_trait;
            use yaserde::ser::to_string;
            use super::*;
            }

pub mod services {
use yaserde::{YaSerialize, YaDeserialize};
            use yaserde::de::from_str;
            use async_trait::async_trait;
            use yaserde::ser::to_string;
            use super::*;
            }

