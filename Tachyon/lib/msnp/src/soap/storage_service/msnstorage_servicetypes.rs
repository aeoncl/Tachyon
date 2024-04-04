//! THIS IS A GENERATED FILE!
//! Take care when hand editing. Changes will be lost during subsequent runs of the code generator.
//!
//! version: 0.1.6
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
	namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
	prefix = "soapenv",
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
	rename = "StorageApplicationHeader",
	namespace = "tns: http://www.msn.com/webservices/storage/2008",
	namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
	prefix = "tns",
)]
pub struct StorageApplicationHeader {
	#[yaserde(rename = "ApplicationID", default)]
	pub application_id: String, 
	#[yaserde(rename = "Scenario", default)]
	pub scenario: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "StorageUserHeader",
	namespace = "tns: http://www.msn.com/webservices/storage/2008",
	namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
	prefix = "tns",
)]
pub struct StorageUserHeader {
	#[yaserde(rename = "Puid", default)]
	pub puid: i32, 
	#[yaserde(rename = "Cid", default)]
	pub cid: Option<i32>, 
	#[yaserde(rename = "TicketToken", default)]
	pub ticket_token: Token, 
	#[yaserde(rename = "IsAdmin", default)]
	pub is_admin: Option<bool>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "AffinityCacheHeader",
	namespace = "tns: http://www.msn.com/webservices/storage/2008",
	namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
	prefix = "tns",
)]
pub struct AffinityCacheHeader {
	#[yaserde(rename = "CacheKey", default)]
	pub cache_key: Option<Token>, 
}
pub type GetProfile = GetProfileRequestType;

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "GetProfileRequestType",
)]
pub struct GetProfileRequestType {
	#[yaserde(rename = "profileHandle", default)]
	pub profile_handle: Handle, 
	#[yaserde(rename = "profileAttributes", default)]
	pub profile_attributes: ProfileAttributes, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "GetProfileResponse",
	namespace = "tns: http://www.msn.com/webservices/storage/2008",
	namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
	prefix = "tns",
)]
pub struct GetProfileResponse {
	#[yaserde(rename = "GetProfileResult", default)]
	pub get_profile_result: GetProfileResultType, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "Attachments",
)]
pub struct Attachments {
	#[yaserde(rename = "Document", default)]
	pub document: Vec<DocumentBaseType>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ExpressionProfile",
)]
pub struct ExpressionProfile {
	#[yaserde(rename = "ResourceID", default)]
	pub resource_id: String, 
	#[yaserde(rename = "DateModified", default)]
	pub date_modified: String, 
	#[yaserde(rename = "Version", default)]
	pub version: Option<i32>, 
	#[yaserde(rename = "Flags", default)]
	pub flags: i32, 
	#[yaserde(rename = "Photo", default)]
	pub photo: DocumentBaseType, 
	#[yaserde(rename = "Attachments", default)]
	pub attachments: Option<Attachments>, 
	#[yaserde(rename = "PersonalStatus", default)]
	pub personal_status: String, 
	#[yaserde(rename = "PersonalStatusLastModified", default)]
	pub personal_status_last_modified: String, 
	#[yaserde(rename = "DisplayName", default)]
	pub display_name: String, 
	#[yaserde(rename = "DisplayNameLastModified", default)]
	pub display_name_last_modified: String, 
	#[yaserde(rename = "StaticUserTilePublicURL", default)]
	pub static_user_tile_public_url: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "GetProfileResultType",
)]
pub struct GetProfileResultType {
	#[yaserde(rename = "ResourceID", default)]
	pub resource_id: String, 
	#[yaserde(rename = "DateModified", default)]
	pub date_modified: String, 
	#[yaserde(rename = "ExpressionProfile", default)]
	pub expression_profile: ExpressionProfile, 
}
pub type UpdateProfile = UpdateProfileRequestType;

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "profile",
)]
pub struct Profile {
	#[yaserde(rename = "ResourceID", default)]
	pub resource_id: String, 
	#[yaserde(rename = "ExpressionProfile", default)]
	pub expression_profile: ExpressionProfile, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "profileAttributesToDelete",
)]
pub struct ProfileAttributesToDelete {
	#[yaserde(rename = "ExpressionProfileAttributes", default)]
	pub expression_profile_attributes: ExpressionProfileAttributesType, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "UpdateProfileRequestType",
)]
pub struct UpdateProfileRequestType {
	#[yaserde(rename = "profile", default)]
	pub profile: Profile, 
	#[yaserde(rename = "profileAttributesToDelete", default)]
	pub profile_attributes_to_delete: Option<ProfileAttributesToDelete>, 
}
pub type UpdateProfileResponse = AnyType;

pub type AnyType = Option<String>;

pub type FindDocuments = FindDocumentsRequestType;

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "documentAttributes",
)]
pub struct DocumentAttributes {
	#[yaserde(rename = "ResourceID", default)]
	pub resource_id: bool, 
	#[yaserde(rename = "Name", default)]
	pub name: bool, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "documentFilter",
)]
pub struct DocumentFilter {
	#[yaserde(rename = "FilterAttributes", default)]
	pub filter_attributes: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "documentSort",
)]
pub struct DocumentSort {
	#[yaserde(rename = "SortBy", default)]
	pub sort_by: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "findContext",
)]
pub struct FindContext {
	#[yaserde(rename = "FindMethod", default)]
	pub find_method: String, 
	#[yaserde(rename = "ChunkSize", default)]
	pub chunk_size: i32, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "FindDocumentsRequestType",
)]
pub struct FindDocumentsRequestType {
	#[yaserde(rename = "objectHandle", default)]
	pub object_handle: Handle, 
	#[yaserde(rename = "documentAttributes", default)]
	pub document_attributes: DocumentAttributes, 
	#[yaserde(rename = "documentFilter", default)]
	pub document_filter: DocumentFilter, 
	#[yaserde(rename = "documentSort", default)]
	pub document_sort: DocumentSort, 
	#[yaserde(rename = "findContext", default)]
	pub find_context: FindContext, 
}
pub type FindDocumentsResponse = FindDocumentsResultType;

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "Document",
)]
pub struct Document {
	#[yaserde(rename = "ResourceID", default)]
	pub resource_id: String, 
	#[yaserde(rename = "Name", default)]
	pub name: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "FindDocumentsResultType",
)]
pub struct FindDocumentsResultType {
	#[yaserde(rename = "Document", default)]
	pub document: Document, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "CreateProfileRequestType",
)]
pub struct CreateProfileRequestType {
	#[yaserde(rename = "profile", default)]
	pub profile: Profile, 
}
pub type CreateProfile = CreateProfileRequestType;

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "CreateProfileResponse",
	namespace = "tns: http://www.msn.com/webservices/storage/2008",
	namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
	prefix = "tns",
)]
pub struct CreateProfileResponse {
	#[yaserde(rename = "CreateProfileResult", default)]
	pub create_profile_result: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ShareItemRequestType",
)]
pub struct ShareItemRequestType {
	#[yaserde(rename = "resourceID", default)]
	pub resource_id: String, 
	#[yaserde(rename = "displayName", default)]
	pub display_name: String, 
}
pub type ShareItem = ShareItemRequestType;

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ShareItemResponseType",
)]
pub struct ShareItemResponseType {
	#[yaserde(rename = "ShareItemResult", default)]
	pub share_item_result: String, 
}
pub type ShareItemResponse = ShareItemResponseType;

pub type UpdateDocument = UpdateDocumentRequestType;

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "UpdateDocumentRequestType",
)]
pub struct UpdateDocumentRequestType {
	#[yaserde(rename = "document", default)]
	pub document: DocumentBaseType, 
}
pub type UpdateDocumentResponse = AnyType;

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "CreateDocumentRequestType",
)]
pub struct CreateDocumentRequestType {
	#[yaserde(rename = "parentHandle", default)]
	pub parent_handle: Handle, 
	#[yaserde(rename = "document", default)]
	pub document: DocumentBaseType, 
	#[yaserde(rename = "relationshipName", default)]
	pub relationship_name: String, 
}
pub type CreateDocument = CreateDocumentRequestType;

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "CreateDocumentResponseType",
)]
pub struct CreateDocumentResponseType {
	#[yaserde(rename = "CreateDocumentResult", default)]
	pub create_document_result: String, 
}
pub type CreateDocumentResponse = CreateDocumentResponseType;

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "relationships",
)]
pub struct Relationships {
	#[yaserde(rename = "Relationship", default)]
	pub relationship: Vec<Relationship>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "CreateRelationshipsRequestType",
)]
pub struct CreateRelationshipsRequestType {
	#[yaserde(rename = "relationships", default)]
	pub relationships: Relationships, 
}
pub type CreateRelationships = CreateRelationshipsRequestType;

pub type CreateRelationshipsResponse = AnyType;

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "targetHandles",
)]
pub struct TargetHandles {
	#[yaserde(rename = "ObjectHandle", default)]
	pub object_handle: Vec<Handle>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "DeleteRelationshipsRequestType",
)]
pub struct DeleteRelationshipsRequestType {
	#[yaserde(rename = "sourceHandle", default)]
	pub source_handle: Handle, 
	#[yaserde(rename = "targetHandles", default)]
	pub target_handles: TargetHandles, 
}
pub type DeleteRelationships = DeleteRelationshipsRequestType;

pub type DeleteRelationshipsResponse = AnyType;

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

