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

            use self::types::{StorageUserHeader, StorageApplicationHeader, AffinityCacheHeader};
            
            pub const SOAP_ENCODING: &str = "http://www.w3.org/2003/05/soap-encoding";
            #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
pub struct Header {
}



#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(rename = "Header")]
pub struct RequestHeaderContainer{

    #[yaserde(rename="AffinityCacheHeader")]
    pub affinity_cache_header: Option<AffinityCacheHeader>,

    #[yaserde(rename="StorageApplicationHeader")]
    pub storage_application_header: Option<StorageApplicationHeader>,

    #[yaserde(rename="StorageUserHeader")]
    pub storage_user_header: Option<StorageUserHeader>,
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
            #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "SSHeader",
)]
pub struct Ssheader {
	#[yaserde(flatten, default)]
	pub storage_application_header: types::StorageApplicationHeader, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "GetProfileMessage",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	prefix = "nsi1",
    default_namespace="nsi1"
)]
pub struct GetProfileMessage {
	#[yaserde(flatten, prefix="nsi1")]
	pub get_profile_request: types::GetProfile, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "GetProfileResponse",
)]
pub struct GetProfileResponseMessage {
	#[yaserde(flatten, default)]
	pub get_profile_response: types::GetProfileResponse, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "UpdateProfile",

)]
pub struct UpdateProfileMessage {
	#[yaserde(flatten, default)]
	pub update_profile_request: types::UpdateProfile, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "FindDocumentsMessage",
)]
pub struct FindDocumentsMessage {
	#[yaserde(flatten, default)]
	pub find_documents_request: types::FindDocuments, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "FindDocumentsResponseMessage",
)]
pub struct FindDocumentsResponseMessage {
	#[yaserde(flatten, default)]
	pub find_documents_response: types::FindDocumentsResponse, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "CreateProfileMessage",
)]
pub struct CreateProfileMessage {
	#[yaserde(flatten, default)]
	pub create_profile: types::CreateProfile, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "CreateProfileResponseMessage",
)]
pub struct CreateProfileResponseMessage {
	#[yaserde(flatten, default)]
	pub create_profile_response: types::CreateProfileResponse, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ShareItemMessage",
)]
pub struct ShareItemMessage {
	#[yaserde(flatten, default)]
	pub share_item: types::ShareItem, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ShareItemResponseMessage",
)]
pub struct ShareItemResponseMessage {
	#[yaserde(flatten, default)]
	pub share_item_response: types::ShareItemResponse, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "CreateDocumentMessage",
)]
pub struct CreateDocumentMessage {
	#[yaserde(flatten, default)]
	pub create_document: types::CreateDocument, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "CreateDocumentResponseMessage",
)]
pub struct CreateDocumentResponseMessage {
	#[yaserde(flatten, default)]
	pub create_document_response: types::CreateDocumentResponse, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "UpdateDocument",
)]
pub struct UpdateDocumentMessage {
	#[yaserde(flatten, default)]
	pub update_document: types::UpdateDocument, 
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "CreateRelationshipsMessage",
)]
pub struct CreateRelationshipsMessage {
	#[yaserde(flatten, default)]
	pub create_relationships: types::CreateRelationships, 
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "DeleteRelationships",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	prefix = "nsi1",
    default_namespace="nsi1"
)]
pub struct DeleteRelationshipsMessage {
	#[yaserde(flatten, prefix="nis1")]
	pub delete_relationships: types::DeleteRelationships, 
}


}

pub mod types {
use yaserde::{YaSerialize, YaDeserialize};
            use yaserde::de::from_str;
            use async_trait::async_trait;
            use yaserde::ser::to_string;
            use crate::generated::msnstorage_datatypes::types::{Handle, DocumentBaseType, Relationship, ExpressionProfileAttributesType, ProfileAttributes};

            use super::*;
            #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "StorageApplicationHeader",
	namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
	prefix = "nsi1",
    default_namespace="nsi1"
)]
pub struct StorageApplicationHeader {
	#[yaserde(rename = "ApplicationID", prefix="nsi1")]
	pub application_id: String, 
	#[yaserde(rename = "Scenario", prefix="nsi1")]
	pub scenario: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "StorageUserHeader",
	namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
	prefix = "nsi1",
    default_namespace="nsi1"
)]
pub struct StorageUserHeader {
	#[yaserde(rename = "Puid", prefix="nsi1")]
	pub puid: i32, 
	#[yaserde(rename = "Cid", prefix="nsi1")]
	pub cid: Option<String>,
    #[yaserde(rename = "ApplicationId", prefix="nsi1")]
    pub application_id: Option<i32>, 
    #[yaserde(rename = "DeviceId", prefix="nsi1")]
    pub device_id: Option<i32>, 
    #[yaserde(rename = "IsTrustedDevice", prefix="nsi1")]
    pub is_trusted_device: Option<bool>, 
    #[yaserde(rename = "IsStrongAuth", prefix="nsi1")]
    pub is_strong_auth: Option<bool>, 
	#[yaserde(rename = "TicketToken", prefix="nsi1")]
	pub ticket_token: String, 
	#[yaserde(rename = "IsAdmin", prefix="nsi1")]
	pub is_admin: Option<bool>, 
    #[yaserde(rename = "LanguagePreference", prefix="nsi1")]
    pub language_preference: Option<i32>,
    #[yaserde(rename = "Claims", prefix="nsi1")]
    pub claims : Vec<String>
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "AffinityCacheHeader",
	namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
	prefix = "nsi1",
    default_namespace="nsi1"
)]
pub struct AffinityCacheHeader {
	#[yaserde(rename = "CacheKey", prefix="nsi1")]
	pub cache_key: Option<String>, 
}
pub type GetProfile = GetProfileRequestType;

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "GetProfileRequestType",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	prefix = "nsi1",
    default_namespace="nsi1"
)]
pub struct GetProfileRequestType {
	#[yaserde(rename = "profileHandle", prefix="nsi1")]
	pub profile_handle: Handle, 
	#[yaserde(rename = "profileAttributes", prefix="nsi1")]
	pub profile_attributes: ProfileAttributes, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "GetProfileResponse",
	namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	prefix = "nsi1",
    default_namespace = "nsi1"
)]
pub struct GetProfileResponse {
	#[yaserde(rename = "GetProfileResult", prefix = "nsi1")]
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
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	prefix = "nsi1",
    default_namespace="nsi1"
)]
pub struct ExpressionProfile {
	#[yaserde(rename = "ResourceID", prefix="nsi1")]
	pub resource_id: String, 
	#[yaserde(rename = "DateModified", prefix="nsi1")]
	pub date_modified: String, 
	#[yaserde(rename = "Version", prefix="nsi1")]
	pub version: Option<i32>, 
	#[yaserde(rename = "Flags", prefix="nsi1")]
	pub flags: Option<i32>, 
	#[yaserde(rename = "Photo", prefix="nsi1")]
	pub photo: DocumentBaseType, 
	#[yaserde(rename = "Attachments", prefix="nsi1")]
	pub attachments: Option<Attachments>, 
	#[yaserde(rename = "PersonalStatus", prefix="nsi1")]
	pub personal_status: Option<String>, 
	#[yaserde(rename = "PersonalStatusLastModified", prefix="nsi1")]
	pub personal_status_last_modified: Option<String>, 
	#[yaserde(rename = "DisplayName", prefix="nsi1")]
	pub display_name: Option<String>, 
	#[yaserde(rename = "DisplayNameLastModified", prefix="nsi1")]
	pub display_name_last_modified: Option<String>, 
	#[yaserde(rename = "StaticUserTilePublicURL", prefix="nsi1")]
	pub static_user_tile_public_url: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "GetProfileResultType",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	prefix = "nsi1",
    default_namespace="nsi1"
)]
pub struct GetProfileResultType {
	#[yaserde(rename = "ResourceID", prefix="nsi1")]
	pub resource_id: String, 
	#[yaserde(rename = "DateModified", prefix="nsi1")]
	pub date_modified: String, 
	#[yaserde(rename = "ExpressionProfile", prefix="nsi1")]
	pub expression_profile: ExpressionProfile, 
}
pub type UpdateProfile = UpdateProfileRequestType;

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "profile",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	prefix = "nsi1",
    default_namespace="nsi1"
)]
pub struct Profile {
	#[yaserde(rename = "ResourceID", prefix="nsi1")]
	pub resource_id: String, 
	#[yaserde(rename = "ExpressionProfile", prefix="nsi1")]
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
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	prefix = "nsi1",
    default_namespace="nsi1"
)]
pub struct UpdateProfileRequestType {
	#[yaserde(rename = "profile", prefix="nsi1")]
	pub profile: Profile, 
	#[yaserde(rename = "profileAttributesToDelete", prefix="nsi1")]
	pub profile_attributes_to_delete: Option<ProfileAttributesToDelete>, 
}
pub type UpdateProfileResponse = Option<String>;

pub type FindDocuments = FindDocumentsRequestType;

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "documentAttributes",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	prefix = "nsi1",
    default_namespace="nsi1"
)]
pub struct DocumentAttributes {
	#[yaserde(rename = "ResourceID", prefix="nsi1")]
	pub resource_id: bool, 
	#[yaserde(rename = "Name", prefix="nsi1")]
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
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	prefix = "nsi1",
    default_namespace="nsi1"
)]
pub struct Document {
	#[yaserde(rename = "ResourceID", prefix="nsi1")]
	pub resource_id: String, 
	#[yaserde(rename = "Name", prefix="nsi1")]
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
	namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
	prefix = "nsi1",
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
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
	prefix = "nsi1",
    default_namespace="nsi1"
)]
pub struct UpdateDocumentRequestType {
	#[yaserde(rename = "document", prefix="nsi1")]
	pub document: DocumentBaseType, 
}
pub type UpdateDocumentResponse = Option<String>;

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

pub type CreateRelationshipsResponse = Option<String>;

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
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	prefix = "nsi1",
    default_namespace="nsi1"

)]
pub struct DeleteRelationshipsRequestType {
	#[yaserde(rename = "sourceHandle", prefix="nsi1")]
	pub source_handle: Handle, 
	#[yaserde(rename = "targetHandles", prefix="nsi1")]
	pub target_handles: TargetHandles, 
}
pub type DeleteRelationships = DeleteRelationshipsRequestType;

pub type DeleteRelationshipsResponse = Option<String>;

}

pub mod ports {
use yaserde::{YaSerialize, YaDeserialize};
            use yaserde::de::from_str;
            use async_trait::async_trait;
            use yaserde::ser::to_string;
            use super::*;
            pub type GetProfileMessage = messages::GetProfileMessage;

pub type GetProfileResponseMessage = messages::GetProfileResponseMessage;

pub type UpdateProfileMessage = messages::UpdateProfileMessage;

pub type UpdateProfileResponseMessage = Option<String>;

pub type FindDocumentsMessage = messages::FindDocumentsMessage;

pub type FindDocumentsResponseMessage = messages::FindDocumentsResponseMessage;

pub type CreateProfileMessage = messages::CreateProfileMessage;

pub type CreateProfileResponseMessage = messages::CreateProfileResponseMessage;

pub type ShareItemMessage = messages::ShareItemMessage;

pub type ShareItemResponseMessage = messages::ShareItemResponseMessage;

pub type CreateDocumentMessage = messages::CreateDocumentMessage;

pub type CreateDocumentResponseMessage = messages::CreateDocumentResponseMessage;

pub type UpdateDocumentMessage = messages::UpdateDocumentMessage;

pub type UpdateDocumentResponseMessage = Option<String>;

pub type CreateRelationshipsMessage = messages::CreateRelationshipsMessage;

pub type CreateRelationshipsResponseMessage = Option<String>;

pub type DeleteRelationshipsMessage = messages::DeleteRelationshipsMessage;

pub type DeleteRelationshipsResponseMessage = Option<String>;

#[async_trait]
pub trait StorageServicePortType {
	async fn get_profile (&self, get_profile_message: GetProfileMessage) -> Result<GetProfileResponseMessage,Option<SoapFault>>;
	async fn update_profile (&self, update_profile_message: UpdateProfileMessage) -> Result<UpdateProfileResponseMessage,Option<SoapFault>>;
	async fn find_documents (&self, find_documents_message: FindDocumentsMessage) -> Result<FindDocumentsResponseMessage,Option<SoapFault>>;
	async fn create_profile (&self, create_profile_message: CreateProfileMessage) -> Result<CreateProfileResponseMessage,Option<SoapFault>>;
	async fn share_item (&self, share_item_message: ShareItemMessage) -> Result<ShareItemResponseMessage,Option<SoapFault>>;
	async fn create_document (&self, create_document_message: CreateDocumentMessage) -> Result<CreateDocumentResponseMessage,Option<SoapFault>>;
	async fn update_document (&self, update_document_message: UpdateDocumentMessage) -> Result<UpdateDocumentResponseMessage,Option<SoapFault>>;
	async fn create_relationships (&self, create_relationships_message: CreateRelationshipsMessage) -> Result<CreateRelationshipsResponseMessage,Option<SoapFault>>;
	async fn delete_relationships (&self, delete_relationships_message: DeleteRelationshipsMessage) -> Result<DeleteRelationshipsResponseMessage,Option<SoapFault>>;
}
}

pub mod bindings {
use log::info;
use yaserde::{YaSerialize, YaDeserialize};
            use yaserde::de::from_str;
            use async_trait::async_trait;
            use yaserde::ser::to_string;
            use super::*;
            
            impl StorageServiceBinding {
                async fn send_soap_request<T: YaSerialize>(&self, request: &T, action: &str) -> SoapResponse {
                    let body = to_string(request).expect("failed to generate xml");
                    info!("SOAP Request: {}", body);
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
                    info!("SOAP Response: {}", txt);
                    Ok((status, txt))
                }
            }
            #[derive(Debug, Default, YaSerialize, YaDeserialize)]
                    pub struct SoapGetProfileMessage {
                        #[yaserde(rename = "GetProfile", default)]
                        pub body: ports::GetProfileMessage,
                        #[yaserde(attribute)]
                        pub xmlns: Option<String>,
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
        pub struct GetProfileMessageSoapEnvelope {
            #[yaserde(rename = "Header", prefix = "soap")]
            pub header: Option<RequestHeaderContainer>,
            #[yaserde(rename = "Body", prefix = "soap")]
            pub body: SoapGetProfileMessage,
        }
        
        impl GetProfileMessageSoapEnvelope {
            pub fn new(body: SoapGetProfileMessage) -> Self {
                GetProfileMessageSoapEnvelope {
                    body,
                    header: None,
                }
            }
        }
                    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
                    pub struct SoapGetProfileResponseMessage {
                    #[yaserde(rename = "GetProfileResponse", default)]
                    pub body: ports::GetProfileResponseMessage,
                         #[yaserde(rename = "Fault", default)]
                            pub fault: Option<SoapFault>,
                            
                }
                #[derive(Debug, Default, YaSerialize, YaDeserialize)]
        #[yaserde(
            rename = "Envelope",
            namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
            namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
            namespace = "xsd: http://www.w3.org/2001/XMLSchema"
            prefix = "soap"
        )]
        pub struct GetProfileResponseMessageSoapEnvelope {
            #[yaserde(rename = "Header", prefix = "soap")]
            pub header: Option<RequestHeaderContainer>,
            #[yaserde(rename = "Body", prefix = "soap")]
            pub body: SoapGetProfileResponseMessage,
        }
        
        impl GetProfileResponseMessageSoapEnvelope {
            pub fn new(body: SoapGetProfileResponseMessage) -> Self {
                GetProfileResponseMessageSoapEnvelope {
                    body,
                    header: None
                }
            }
        }        
        
                #[derive(Debug, Default, YaSerialize, YaDeserialize)]
                    pub struct SoapUpdateProfileMessage {
                        #[yaserde(rename = "UpdateProfile", default)]
                        pub body: ports::UpdateProfileMessage,
                        #[yaserde(attribute)]
                        pub xmlns: Option<String>,
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
        pub struct UpdateProfileMessageSoapEnvelope {
            #[yaserde(rename = "Header", prefix = "soap")]
            pub header: Option<RequestHeaderContainer>,
            #[yaserde(rename = "Body", prefix = "soap")]
            pub body: SoapUpdateProfileMessage,
        }
        
        impl UpdateProfileMessageSoapEnvelope {
            pub fn new(body: SoapUpdateProfileMessage) -> Self {
                UpdateProfileMessageSoapEnvelope {
                    body,
                    header: None,
                }
            }
        }        
        
                    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
                    pub struct SoapUpdateProfileResponseMessage {
                    #[yaserde(rename = "UpdateProfileResponse", default)]
                    pub body: Option<String>,
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
        pub struct UpdateProfileResponseMessageSoapEnvelope {
            #[yaserde(rename = "Header", prefix = "soap")]
            pub header: Option<RequestHeaderContainer>,
            #[yaserde(rename = "Body", prefix = "soap")]
            pub body: SoapUpdateProfileResponseMessage,
        }
        
        impl UpdateProfileResponseMessageSoapEnvelope {
            pub fn new(body: SoapUpdateProfileResponseMessage) -> Self {
                UpdateProfileResponseMessageSoapEnvelope {
                    body,
                    header: None,
                }
            }
        }        
        
                #[derive(Debug, Default, YaSerialize, YaDeserialize)]
                    pub struct SoapFindDocumentsMessage {
                        #[yaserde(rename = "FindDocuments", default)]
                        pub body: ports::FindDocumentsMessage,
                        #[yaserde(attribute)]
                        pub xmlns: Option<String>,
                    }
                    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
        #[yaserde(
            rename = "Envelope",
            namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soap"
        )]
        pub struct FindDocumentsMessageSoapEnvelope {
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
            pub body: SoapFindDocumentsMessage,
        }
        
        impl FindDocumentsMessageSoapEnvelope {
            pub fn new(body: SoapFindDocumentsMessage) -> Self {
                FindDocumentsMessageSoapEnvelope {
                    encoding_style: SOAP_ENCODING.to_string(),
                    tnsattr: Option::Some("http://www.msn.com/webservices/storage/2008".to_string()),
                    body,
                    urnattr: None,
                    xsiattr: None,
                    header: None,
                }
            }
        }        
        
                    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
                    pub struct SoapFindDocumentsResponseMessage {
                    #[yaserde(rename = "FindDocumentsResponseMessage", default)]
                    pub body: ports::FindDocumentsResponseMessage,
                         #[yaserde(rename = "Fault", default)]
                            pub fault: Option<SoapFault>,
                            
                }
                #[derive(Debug, Default, YaSerialize, YaDeserialize)]
        #[yaserde(
            rename = "Envelope",
            namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soap"
        )]
        pub struct FindDocumentsResponseMessageSoapEnvelope {
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
            pub body: SoapFindDocumentsResponseMessage,
        }
        
        impl FindDocumentsResponseMessageSoapEnvelope {
            pub fn new(body: SoapFindDocumentsResponseMessage) -> Self {
                FindDocumentsResponseMessageSoapEnvelope {
                    encoding_style: SOAP_ENCODING.to_string(),
                    tnsattr: Option::Some("http://www.msn.com/webservices/storage/2008".to_string()),
                    body,
                    urnattr: None,
                    xsiattr: None,
                    header: None,
                }
            }
        }        
        
                #[derive(Debug, Default, YaSerialize, YaDeserialize)]
                    pub struct SoapCreateProfileMessage {
                        #[yaserde(rename = "CreateProfile", default)]
                        pub body: ports::CreateProfileMessage,
                        #[yaserde(attribute)]
                        pub xmlns: Option<String>,
                    }
                    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
        #[yaserde(
            rename = "Envelope",
            namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soap"
        )]
        pub struct CreateProfileMessageSoapEnvelope {
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
            pub body: SoapCreateProfileMessage,
        }
        
        impl CreateProfileMessageSoapEnvelope {
            pub fn new(body: SoapCreateProfileMessage) -> Self {
                CreateProfileMessageSoapEnvelope {
                    encoding_style: SOAP_ENCODING.to_string(),
                    tnsattr: Option::Some("http://www.msn.com/webservices/storage/2008".to_string()),
                    body,
                    urnattr: None,
                    xsiattr: None,
                    header: None,
                }
            }
        }        
        
                    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
                    pub struct SoapCreateProfileResponseMessage {
                    #[yaserde(rename = "CreateProfileResponseMessage", default)]
                    pub body: ports::CreateProfileResponseMessage,
                         #[yaserde(rename = "Fault", default)]
                            pub fault: Option<SoapFault>,
                            
                }
                #[derive(Debug, Default, YaSerialize, YaDeserialize)]
        #[yaserde(
            rename = "Envelope",
            namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soap"
        )]
        pub struct CreateProfileResponseMessageSoapEnvelope {
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
            pub body: SoapCreateProfileResponseMessage,
        }
        
        impl CreateProfileResponseMessageSoapEnvelope {
            pub fn new(body: SoapCreateProfileResponseMessage) -> Self {
                CreateProfileResponseMessageSoapEnvelope {
                    encoding_style: SOAP_ENCODING.to_string(),
                    tnsattr: Option::Some("http://www.msn.com/webservices/storage/2008".to_string()),
                    body,
                    urnattr: None,
                    xsiattr: None,
                    header: None,
                }
            }
        }        
        
                #[derive(Debug, Default, YaSerialize, YaDeserialize)]
                    pub struct SoapShareItemMessage {
                        #[yaserde(rename = "ShareItem", default)]
                        pub body: ports::ShareItemMessage,
                        #[yaserde(attribute)]
                        pub xmlns: Option<String>,
                    }
                    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
        #[yaserde(
            rename = "Envelope",
            namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soap"
        )]
        pub struct ShareItemMessageSoapEnvelope {
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
            pub body: SoapShareItemMessage,
        }
        
        impl ShareItemMessageSoapEnvelope {
            pub fn new(body: SoapShareItemMessage) -> Self {
                ShareItemMessageSoapEnvelope {
                    encoding_style: SOAP_ENCODING.to_string(),
                    tnsattr: Option::Some("http://www.msn.com/webservices/storage/2008".to_string()),
                    body,
                    urnattr: None,
                    xsiattr: None,
                    header: None,
                }
            }
        }        
        
                    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
                    pub struct SoapShareItemResponseMessage {
                    #[yaserde(rename = "ShareItemResponseMessage", default)]
                    pub body: ports::ShareItemResponseMessage,
                         #[yaserde(rename = "Fault", default)]
                            pub fault: Option<SoapFault>,
                            
                }
                #[derive(Debug, Default, YaSerialize, YaDeserialize)]
        #[yaserde(
            rename = "Envelope",
            namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soap"
        )]
        pub struct ShareItemResponseMessageSoapEnvelope {
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
            pub body: SoapShareItemResponseMessage,
        }
        
        impl ShareItemResponseMessageSoapEnvelope {
            pub fn new(body: SoapShareItemResponseMessage) -> Self {
                ShareItemResponseMessageSoapEnvelope {
                    encoding_style: SOAP_ENCODING.to_string(),
                    tnsattr: Option::Some("http://www.msn.com/webservices/storage/2008".to_string()),
                    body,
                    urnattr: None,
                    xsiattr: None,
                    header: None,
                }
            }
        }        
        
                #[derive(Debug, Default, YaSerialize, YaDeserialize)]
                    pub struct SoapCreateDocumentMessage {
                        #[yaserde(rename = "CreateDocument", default)]
                        pub body: ports::CreateDocumentMessage,
                        #[yaserde(attribute)]
                        pub xmlns: Option<String>,
                    }
                    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
        #[yaserde(
            rename = "Envelope",
            namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soap"
        )]
        pub struct CreateDocumentMessageSoapEnvelope {
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
            pub body: SoapCreateDocumentMessage,
        }
        
        impl CreateDocumentMessageSoapEnvelope {
            pub fn new(body: SoapCreateDocumentMessage) -> Self {
                CreateDocumentMessageSoapEnvelope {
                    encoding_style: SOAP_ENCODING.to_string(),
                    tnsattr: Option::Some("http://www.msn.com/webservices/storage/2008".to_string()),
                    body,
                    urnattr: None,
                    xsiattr: None,
                    header: None,
                }
            }
        }        
        
                    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
                    pub struct SoapCreateDocumentResponseMessage {
                    #[yaserde(rename = "CreateDocumentResponseMessage", default)]
                    pub body: ports::CreateDocumentResponseMessage,
                         #[yaserde(rename = "Fault", default)]
                            pub fault: Option<SoapFault>,
                            
                }
                #[derive(Debug, Default, YaSerialize, YaDeserialize)]
        #[yaserde(
            rename = "Envelope",
            namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soap"
        )]
        pub struct CreateDocumentResponseMessageSoapEnvelope {
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
            pub body: SoapCreateDocumentResponseMessage,
        }
        
        impl CreateDocumentResponseMessageSoapEnvelope {
            pub fn new(body: SoapCreateDocumentResponseMessage) -> Self {
                CreateDocumentResponseMessageSoapEnvelope {
                    encoding_style: SOAP_ENCODING.to_string(),
                    tnsattr: Option::Some("http://www.msn.com/webservices/storage/2008".to_string()),
                    body,
                    urnattr: None,
                    xsiattr: None,
                    header: None,
                }
            }
        }        
        
        #[derive(Debug, Default, YaSerialize, YaDeserialize)]
        pub struct SoapUpdateDocumentMessage {
            #[yaserde(rename = "UpdateDocument", default)]
            pub body: ports::UpdateDocumentMessage,
            #[yaserde(attribute)]
            pub xmlns: Option<String>,
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
        pub struct UpdateDocumentMessageSoapEnvelope {
            #[yaserde(rename = "Header", prefix = "soap")]
            pub header: Option<RequestHeaderContainer>,
            #[yaserde(rename = "Body", prefix = "soap")]
            pub body: SoapUpdateDocumentMessage,
        }
        
        impl UpdateDocumentMessageSoapEnvelope {
            pub fn new(body: SoapUpdateDocumentMessage) -> Self {
                UpdateDocumentMessageSoapEnvelope {
                    body,
                    header: None,
                }
            }
        }        
        
                    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
                    pub struct SoapUpdateDocumentResponseMessage {
                    #[yaserde(rename = "UpdateDocumentResponse", default)]
                    pub body: Option<String>,
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
        pub struct UpdateDocumentResponseMessageSoapEnvelope {
            #[yaserde(rename = "Header", prefix = "soap")]
            pub header: Option<RequestHeaderContainer>,
            #[yaserde(rename = "Body", prefix = "soap")]
            pub body: SoapUpdateDocumentResponseMessage,
        }
        
        impl UpdateDocumentResponseMessageSoapEnvelope {
            pub fn new(body: SoapUpdateDocumentResponseMessage) -> Self {
                UpdateDocumentResponseMessageSoapEnvelope {
                    body,
                    header: None,
                }
            }
        }        
        
                #[derive(Debug, Default, YaSerialize, YaDeserialize)]
                    pub struct SoapCreateRelationshipsMessage {
                        #[yaserde(rename = "CreateRelationships", default)]
                        pub body: ports::CreateRelationshipsMessage,
                        #[yaserde(attribute)]
                        pub xmlns: Option<String>,
                    }
                    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
        #[yaserde(
            rename = "Envelope",
            namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soap"
        )]
        pub struct CreateRelationshipsMessageSoapEnvelope {
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
            pub body: SoapCreateRelationshipsMessage,
        }
        
        impl CreateRelationshipsMessageSoapEnvelope {
            pub fn new(body: SoapCreateRelationshipsMessage) -> Self {
                CreateRelationshipsMessageSoapEnvelope {
                    encoding_style: SOAP_ENCODING.to_string(),
                    tnsattr: Option::Some("http://www.msn.com/webservices/storage/2008".to_string()),
                    body,
                    urnattr: None,
                    xsiattr: None,
                    header: None,
                }
            }
        }        
        
                    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
                    pub struct SoapCreateRelationshipsResponseMessage {
                    #[yaserde(rename = "CreateRelationshipsResponseMessage", default)]
                    pub body: Option<String>,
                         #[yaserde(rename = "Fault", default)]
                            pub fault: Option<SoapFault>,
                            
                }
                #[derive(Debug, Default, YaSerialize, YaDeserialize)]
        #[yaserde(
            rename = "Envelope",
            namespace = "soap: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soap"
        )]
        pub struct CreateRelationshipsResponseMessageSoapEnvelope {
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
            pub body: SoapCreateRelationshipsResponseMessage,
        }
        
        impl CreateRelationshipsResponseMessageSoapEnvelope {
            pub fn new(body: SoapCreateRelationshipsResponseMessage) -> Self {
                CreateRelationshipsResponseMessageSoapEnvelope {
                    encoding_style: SOAP_ENCODING.to_string(),
                    tnsattr: Option::Some("http://www.msn.com/webservices/storage/2008".to_string()),
                    body,
                    urnattr: None,
                    xsiattr: None,
                    header: None,
                }
            }
        }        
        
                #[derive(Debug, Default, YaSerialize, YaDeserialize)]
                    pub struct SoapDeleteRelationshipsMessage {
                        #[yaserde(rename = "DeleteRelationships", default)]
                        pub body: ports::DeleteRelationshipsMessage,
                        #[yaserde(attribute)]
                        pub xmlns: Option<String>,
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
        pub struct DeleteRelationshipsMessageSoapEnvelope {
            #[yaserde(rename = "Header", prefix = "soap")]
            pub header: Option<RequestHeaderContainer>,
            #[yaserde(rename = "Body", prefix = "soap")]
            pub body: SoapDeleteRelationshipsMessage,
        }
        
        impl DeleteRelationshipsMessageSoapEnvelope {
            pub fn new(body: SoapDeleteRelationshipsMessage) -> Self {
                DeleteRelationshipsMessageSoapEnvelope {
                    body,
                    header: None,
                }
            }
        }        
        
                    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
                    pub struct SoapDeleteRelationshipsResponseMessage {
                    #[yaserde(rename = "DeleteRelationshipsResponse", default)]
                    pub body: Option<String>,
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
        pub struct DeleteRelationshipsResponseMessageSoapEnvelope {
            #[yaserde(rename = "Header", prefix = "soap")]
            pub header: Option<RequestHeaderContainer>,
            #[yaserde(rename = "Body", prefix = "soap")]
            pub body: SoapDeleteRelationshipsResponseMessage,
        }
        
        impl DeleteRelationshipsResponseMessageSoapEnvelope {
            pub fn new(body: SoapDeleteRelationshipsResponseMessage) -> Self {
                DeleteRelationshipsResponseMessageSoapEnvelope {
                    body,
                    header: None,
                }
            }
        }        
        
                impl Default for StorageServiceBinding {
                fn default() -> Self {
                    StorageServiceBinding {
                        client: reqwest::Client::new(),
                        url: "http://www.msn.com/webservices/storage/2008".to_string(),
                        credentials: Option::None,
                     }
                }
            }
            impl StorageServiceBinding {
                pub fn new(url: &str, credentials: Option<(String,String)>) -> Self {
                    StorageServiceBinding {
                        client: reqwest::Client::new(),
                        url: url.to_string(),
                        credentials,
                    }
                }
        }
        pub struct StorageServiceBinding {
                client: reqwest::Client,
                url: String,
                credentials: Option<(String,String)>
                }
                #[async_trait]
	impl ports::StorageServicePortType for StorageServiceBinding {
	async fn get_profile (&self, get_profile_message: ports::GetProfileMessage) -> Result<ports::GetProfileResponseMessage, Option<SoapFault>> {

        let __request = GetProfileMessageSoapEnvelope::new(SoapGetProfileMessage {
            body: get_profile_message,
            xmlns: Option::Some("http://www.msn.com/webservices/storage/2008".to_string()),
        });            
        
        let (status, response) = self.send_soap_request(&__request, "http://www.msn.com/webservices/storage/2008/GetProfile")
                    .await
                    .map_err(|err| {
                        warn!("Failed to send SOAP request: {:?}", err);
                        None
                    })?;

        let r: GetProfileResponseMessageSoapEnvelope = from_str(&response).map_err(|err| {
                        warn!("Failed to unmarshal SOAP response: {:?}", err);
                        None
                    })?;
        if status.is_success() {
            Ok(r.body.body)
        } else {
            Err(r.body.fault)
        }}	async fn update_profile (&self, update_profile_message: ports::UpdateProfileMessage) -> Result<ports::UpdateProfileResponseMessage, Option<SoapFault>> {

        let __request = UpdateProfileMessageSoapEnvelope::new(SoapUpdateProfileMessage {
            body: update_profile_message,
            xmlns: Option::Some("http://www.msn.com/webservices/storage/2008".to_string()),
        });            
        
        let (status, response) = self.send_soap_request(&__request, "http://www.msn.com/webservices/storage/2008/UpdateProfile")
                    .await
                    .map_err(|err| {
                        warn!("Failed to send SOAP request: {:?}", err);
                        None
                    })?;

        let r: UpdateProfileResponseMessageSoapEnvelope = from_str(&response).map_err(|err| {
                        warn!("Failed to unmarshal SOAP response: {:?}", err);
                        None
                    })?;
        if status.is_success() {
            Ok(r.body.body)
        } else {
            Err(r.body.fault)
        }}	async fn find_documents (&self, find_documents_message: ports::FindDocumentsMessage) -> Result<ports::FindDocumentsResponseMessage, Option<SoapFault>> {

        let __request = FindDocumentsMessageSoapEnvelope::new(SoapFindDocumentsMessage {
            body: find_documents_message,
            xmlns: Option::Some("http://www.msn.com/webservices/storage/2008".to_string()),
        });            
        
        let (status, response) = self.send_soap_request(&__request, "http://www.msn.com/webservices/storage/2008/FindDocuments")
                    .await
                    .map_err(|err| {
                        warn!("Failed to send SOAP request: {:?}", err);
                        None
                    })?;

        let r: FindDocumentsResponseMessageSoapEnvelope = from_str(&response).map_err(|err| {
                        warn!("Failed to unmarshal SOAP response: {:?}", err);
                        None
                    })?;
        if status.is_success() {
            Ok(r.body.body)
        } else {
            Err(r.body.fault)
        }}	async fn create_profile (&self, create_profile_message: ports::CreateProfileMessage) -> Result<ports::CreateProfileResponseMessage, Option<SoapFault>> {

        let __request = CreateProfileMessageSoapEnvelope::new(SoapCreateProfileMessage {
            body: create_profile_message,
            xmlns: Option::Some("http://www.msn.com/webservices/storage/2008".to_string()),
        });            
        
        let (status, response) = self.send_soap_request(&__request, "http://www.msn.com/webservices/storage/2008/CreateProfile")
                    .await
                    .map_err(|err| {
                        warn!("Failed to send SOAP request: {:?}", err);
                        None
                    })?;

        let r: CreateProfileResponseMessageSoapEnvelope = from_str(&response).map_err(|err| {
                        warn!("Failed to unmarshal SOAP response: {:?}", err);
                        None
                    })?;
        if status.is_success() {
            Ok(r.body.body)
        } else {
            Err(r.body.fault)
        }}	async fn share_item (&self, share_item_message: ports::ShareItemMessage) -> Result<ports::ShareItemResponseMessage, Option<SoapFault>> {

        let __request = ShareItemMessageSoapEnvelope::new(SoapShareItemMessage {
            body: share_item_message,
            xmlns: Option::Some("http://www.msn.com/webservices/storage/2008".to_string()),
        });            
        
        let (status, response) = self.send_soap_request(&__request, "http://www.msn.com/webservices/storage/2008/ShareItem")
                    .await
                    .map_err(|err| {
                        warn!("Failed to send SOAP request: {:?}", err);
                        None
                    })?;

        let r: ShareItemResponseMessageSoapEnvelope = from_str(&response).map_err(|err| {
                        warn!("Failed to unmarshal SOAP response: {:?}", err);
                        None
                    })?;
        if status.is_success() {
            Ok(r.body.body)
        } else {
            Err(r.body.fault)
        }}	async fn create_document (&self, create_document_message: ports::CreateDocumentMessage) -> Result<ports::CreateDocumentResponseMessage, Option<SoapFault>> {

        let __request = CreateDocumentMessageSoapEnvelope::new(SoapCreateDocumentMessage {
            body: create_document_message,
            xmlns: Option::Some("http://www.msn.com/webservices/storage/2008".to_string()),
        });            
        
        let (status, response) = self.send_soap_request(&__request, "http://www.msn.com/webservices/storage/2008/CreateDocument")
                    .await
                    .map_err(|err| {
                        warn!("Failed to send SOAP request: {:?}", err);
                        None
                    })?;

        let r: CreateDocumentResponseMessageSoapEnvelope = from_str(&response).map_err(|err| {
                        warn!("Failed to unmarshal SOAP response: {:?}", err);
                        None
                    })?;
        if status.is_success() {
            Ok(r.body.body)
        } else {
            Err(r.body.fault)
        }}	async fn update_document (&self, update_document_message: ports::UpdateDocumentMessage) -> Result<ports::UpdateDocumentResponseMessage, Option<SoapFault>> {

        let __request = UpdateDocumentMessageSoapEnvelope::new(SoapUpdateDocumentMessage {
            body: update_document_message,
            xmlns: Option::Some("http://www.msn.com/webservices/storage/2008".to_string()),
        });            
        
        let (status, response) = self.send_soap_request(&__request, "http://www.msn.com/webservices/storage/2008/UpdateDocument")
                    .await
                    .map_err(|err| {
                        warn!("Failed to send SOAP request: {:?}", err);
                        None
                    })?;

        let r: UpdateDocumentResponseMessageSoapEnvelope = from_str(&response).map_err(|err| {
                        warn!("Failed to unmarshal SOAP response: {:?}", err);
                        None
                    })?;
        if status.is_success() {
            Ok(r.body.body)
        } else {
            Err(r.body.fault)
        }}	async fn create_relationships (&self, create_relationships_message: ports::CreateRelationshipsMessage) -> Result<ports::CreateRelationshipsResponseMessage, Option<SoapFault>> {

        let __request = CreateRelationshipsMessageSoapEnvelope::new(SoapCreateRelationshipsMessage {
            body: create_relationships_message,
            xmlns: Option::Some("http://www.msn.com/webservices/storage/2008".to_string()),
        });            
        
        let (status, response) = self.send_soap_request(&__request, "http://www.msn.com/webservices/storage/2008/CreateRelationships")
                    .await
                    .map_err(|err| {
                        warn!("Failed to send SOAP request: {:?}", err);
                        None
                    })?;

        let r: CreateRelationshipsResponseMessageSoapEnvelope = from_str(&response).map_err(|err| {
                        warn!("Failed to unmarshal SOAP response: {:?}", err);
                        None
                    })?;
        if status.is_success() {
            Ok(r.body.body)
        } else {
            Err(r.body.fault)
        }}	async fn delete_relationships (&self, delete_relationships_message: ports::DeleteRelationshipsMessage) -> Result<ports::DeleteRelationshipsResponseMessage, Option<SoapFault>> {

        let __request = DeleteRelationshipsMessageSoapEnvelope::new(SoapDeleteRelationshipsMessage {
            body: delete_relationships_message,
            xmlns: Option::Some("http://www.msn.com/webservices/storage/2008".to_string()),
        });            
        
        let (status, response) = self.send_soap_request(&__request, "http://www.msn.com/webservices/storage/2008/DeleteRelationships")
                    .await
                    .map_err(|err| {
                        warn!("Failed to send SOAP request: {:?}", err);
                        None
                    })?;

        let r: DeleteRelationshipsResponseMessageSoapEnvelope = from_str(&response).map_err(|err| {
                        warn!("Failed to unmarshal SOAP response: {:?}", err);
                        None
                    })?;
        if status.is_success() {
            Ok(r.body.body)
        } else {
            Err(r.body.fault)
        }}}
}

pub mod services {
use yaserde::{YaSerialize, YaDeserialize};
            use yaserde::de::from_str;
            use async_trait::async_trait;
            use yaserde::ser::to_string;
            use super::*;
            pub struct StorageService {}
               impl StorageService {
                
            pub fn new_client(credentials: Option<(String, String)>) -> bindings::StorageServiceBinding {
                bindings::StorageServiceBinding::new("https://storage.msn.com/storageservice/SchematizedStore.asmx", credentials)
            }
        }
}

pub mod factories {
    use chrono::Local;

    use crate::{generated::{msnstorage_service::{types::{StorageUserHeader, GetProfileResultType, GetProfileResponse, AffinityCacheHeader, ExpressionProfile}, RequestHeaderContainer, bindings::SoapGetProfileResponseMessage, messages::GetProfileResponseMessage}, msnstorage_datatypes::types::{DocumentStream, DocumentStreams, DocumentBaseType}}, models::uuid::UUID};

    use super::bindings::{GetProfileResponseMessageSoapEnvelope, UpdateDocumentResponseMessageSoapEnvelope, SoapUpdateDocumentResponseMessage, UpdateProfileResponseMessageSoapEnvelope, SoapUpdateProfileResponseMessage, DeleteRelationshipsResponseMessageSoapEnvelope, SoapDeleteRelationshipsResponseMessage};


    pub struct GetProfileResponseFactory;

    impl GetProfileResponseFactory {

        pub fn get_response(uuid: UUID, cache_key: String, matrix_token: String, display_name: String, psm: String, image_mxid: Option<String>) -> GetProfileResponseMessageSoapEnvelope {


            let now = Local::now();

            let affinity_cache_header = AffinityCacheHeader{ cache_key: Some(cache_key)};
            let storage_user_header = StorageUserHeader{ puid: 0, cid: Some(uuid.to_hex_cid()), ticket_token: format!("t={}", matrix_token), is_admin: Some(false), application_id: Some(0), device_id: Some(0), is_trusted_device: Some(false), is_strong_auth: Some(false), language_preference: Some(0), claims: Vec::new() };
    
            let headers = RequestHeaderContainer{ affinity_cache_header: Some(affinity_cache_header), storage_application_header: None, storage_user_header: Some(storage_user_header) };
            let mut document_stream_array : Vec<DocumentStream> = Vec::new();


            let mut static_user_tile_public_url = String::new();
            if let Some(found_image_id) = image_mxid {
                let user_tile_static = DocumentStream{ document_stream_name: Some(String::from("UserTileStatic")), mime_type: Some(String::from("image/jpeg")), data: None, data_size: 0, pre_auth_url: Some(format!("http://127.0.0.1/storage/usertile/{}/static", &found_image_id)), pre_auth_url_partner: None, document_stream_type: String::from("UserTileStatic"), write_mode: Some(String::from("Overwrite")), stream_version: Some(0), sha1_hash: None, genie: Some(false), stream_data_status: Some(String::from("None")), stream_status: Some(String::from("None")), is_alias_for_default:Some(false), expiration_date_time: Some(String::from("0001-01-01T00:00:00")) };
                document_stream_array.push(user_tile_static);

                let user_tile_small = DocumentStream{ document_stream_name: Some(String::from("UserTileSmall")), mime_type: None, data: None, data_size: 0, pre_auth_url: Some(format!("http://127.0.0.1/storage/usertile/{}/small", &found_image_id)), pre_auth_url_partner: None, document_stream_type: String::from("Named"), write_mode: Some(String::from("Overwrite")), stream_version: Some(0), sha1_hash: None, genie: Some(false), stream_data_status: Some(String::from("None")), stream_status: Some(String::from("None")), is_alias_for_default:Some(false), expiration_date_time: Some(String::from("0001-01-01T00:00:00")) };
                document_stream_array.push(user_tile_small);

                static_user_tile_public_url = format!("http://127.0.0.1/storage/usertile/{}/static", &found_image_id);
            } 
           

            let document_streams = DocumentStreams{ document_stream: document_stream_array };
    
            let photo = DocumentBaseType{ resource_id: Some(format!("{}!205", uuid.to_hex_cid())), name: None, item_type: None, date_modified: None, document_streams: document_streams };
    
            let expression_profile = ExpressionProfile{ resource_id: format!("{}!118", uuid.to_hex_cid()), date_modified: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(), version: Some(205), flags: None, photo: photo, attachments: None, personal_status: Some(psm), personal_status_last_modified:  Some(now.format("%Y-%m-%dT%H:%M:%SZ").to_string()), display_name: Some(display_name), display_name_last_modified: Some(now.format("%Y-%m-%dT%H:%M:%SZ").to_string()), static_user_tile_public_url };
    
            let get_profile_result = GetProfileResultType { resource_id: format!("{}!106", uuid.to_hex_cid()), date_modified: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(), expression_profile: expression_profile };
    
            let get_profile_response = GetProfileResponse{ get_profile_result: get_profile_result };
    
            let request_body = GetProfileResponseMessage{ get_profile_response: get_profile_response };
    
            let body = SoapGetProfileResponseMessage{ body: request_body, fault: None };
    
    
            return GetProfileResponseMessageSoapEnvelope{ header: Some(headers), body: body };
        }
    }

    pub struct UpdateDocumentResponseFactory;
    impl UpdateDocumentResponseFactory {

        pub fn get_response(matrix_token: String, cache_key: String) -> UpdateDocumentResponseMessageSoapEnvelope {                
            let affinity_cache_header = AffinityCacheHeader{ cache_key: Some(cache_key)};
            let storage_user_header = StorageUserHeader{ puid: 0, cid: None, ticket_token: format!("t={}", matrix_token), is_admin: Some(false), application_id: Some(0), device_id: Some(0), is_trusted_device: Some(false), is_strong_auth: Some(false), language_preference: Some(0), claims: Vec::new() };
    
            let headers = RequestHeaderContainer{ affinity_cache_header: Some(affinity_cache_header), storage_application_header: None, storage_user_header: Some(storage_user_header) };
    
            return UpdateDocumentResponseMessageSoapEnvelope{ header: Some(headers), body: SoapUpdateDocumentResponseMessage{ body: Some(String::new()), fault: None} };
        }
    }


    pub struct UpdateProfileResponseFactory;
    impl UpdateProfileResponseFactory {

        pub fn get_response(matrix_token: String, cache_key: String) -> UpdateProfileResponseMessageSoapEnvelope {        
            let affinity_cache_header = AffinityCacheHeader{ cache_key: Some(cache_key)};
            let storage_user_header = StorageUserHeader{ puid: 0, cid: None, ticket_token: format!("t={}", matrix_token), is_admin: Some(false), application_id: Some(0), device_id: Some(0), is_trusted_device: Some(false), is_strong_auth: Some(false), language_preference: Some(0), claims: Vec::new() };
    
            let headers = RequestHeaderContainer{ affinity_cache_header: Some(affinity_cache_header), storage_application_header: None, storage_user_header: Some(storage_user_header) };
    
            return  UpdateProfileResponseMessageSoapEnvelope { header: Some(headers), body: SoapUpdateProfileResponseMessage { body: Some(String::new()), fault: None }};
        }
    }

    pub struct DeleteRelationshipsResponseFactory;
    impl DeleteRelationshipsResponseFactory {

        pub fn get_response(matrix_token: String, cache_key: String) -> DeleteRelationshipsResponseMessageSoapEnvelope {        

            let affinity_cache_header = AffinityCacheHeader{ cache_key: Some(cache_key)};
            let storage_user_header = StorageUserHeader{ puid: 0, cid: None, ticket_token: format!("t={}", matrix_token), is_admin: Some(false), application_id: Some(0), device_id: Some(0), is_trusted_device: Some(false), is_strong_auth: Some(false), language_preference: Some(0), claims: Vec::new() };

            let headers = RequestHeaderContainer{ affinity_cache_header: Some(affinity_cache_header), storage_application_header: None, storage_user_header: Some(storage_user_header) };

            return DeleteRelationshipsResponseMessageSoapEnvelope { header: Some(headers), body: SoapDeleteRelationshipsResponseMessage { body: Some(String::new()), fault: None }};
        }
    }

}

#[cfg(test)]
mod tests {
    use log::{warn, debug};
    use yaserde::de::from_str;
    use yaserde::ser::to_string;

    use crate::{generated::{msnstorage_datatypes::types::{DocumentBaseType, DocumentStreams, DocumentStream}, msnstorage_service::bindings::{DeleteRelationshipsResponseMessageSoapEnvelope, SoapDeleteRelationshipsResponseMessage}}, models::uuid::UUID};

    use super::{bindings::{GetProfileMessageSoapEnvelope, GetProfileResponseMessageSoapEnvelope, SoapGetProfileResponseMessage, UpdateDocumentMessageSoapEnvelope, UpdateDocumentResponseMessageSoapEnvelope, SoapUpdateDocumentResponseMessage, UpdateProfileMessageSoapEnvelope, UpdateProfileResponseMessageSoapEnvelope, SoapUpdateProfileResponseMessage, DeleteRelationshipsMessageSoapEnvelope}, RequestHeaderContainer, types::{AffinityCacheHeader, StorageApplicationHeader, StorageUserHeader, GetProfileResponse, GetProfileResultType, ExpressionProfile}, messages::GetProfileResponseMessage};


    #[test]
    fn test_get_profile_request() {
        let request = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?><soap:Envelope xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:soapenc=\"http://schemas.xmlsoap.org/soap/encoding/\"><soap:Header><AffinityCacheHeader xmlns=\"http://www.msn.com/webservices/storage/2008\"><CacheKey>KaFdoq7oNpMgw0Te5UxLIJkz6Vd09Z4RpNGVWKm7hEqokd7UK7kw5IUTxuXWLeK3f_7_nFrcH2JT13ThgLDVkL-xBFIACtZWE19ZR4G74VGggxyvefbPLO8wKo5i8ljqgQRti15GZMB4FgCH7Q0MoE6TA46esQws4VxVGYAph8LTzT2b53Ma6T3JnUtl1rc9XsgfatXXvYO0zFLQOamoGtetUqGtmxfQxDOUhg</CacheKey></AffinityCacheHeader><StorageApplicationHeader xmlns=\"http://www.msn.com/webservices/storage/2008\"><ApplicationID>Messenger Client 9.0</ApplicationID><Scenario>Initial</Scenario></StorageApplicationHeader><StorageUserHeader xmlns=\"http://www.msn.com/webservices/storage/2008\"><Puid>0</Puid><TicketToken>t=0bfusc4tedT0k3n</TicketToken></StorageUserHeader></soap:Header><soap:Body><GetProfile xmlns=\"http://www.msn.com/webservices/storage/2008\"><profileHandle><Alias><Name>-9187808514398587066</Name><NameSpace>MyCidStuff</NameSpace></Alias><RelationshipName>MyProfile</RelationshipName></profileHandle><profileAttributes><ResourceID>true</ResourceID><DateModified>true</DateModified><ExpressionProfileAttributes><ResourceID>true</ResourceID><DateModified>true</DateModified><DisplayName>true</DisplayName><DisplayNameLastModified>true</DisplayNameLastModified><PersonalStatus>true</PersonalStatus><PersonalStatusLastModified>true</PersonalStatusLastModified><StaticUserTilePublicURL>true</StaticUserTilePublicURL><Photo>true</Photo><Attachments>true</Attachments><Flags>true</Flags></ExpressionProfileAttributes></profileAttributes></GetProfile></soap:Body></soap:Envelope>");

        let request_deserialized : GetProfileMessageSoapEnvelope = from_str(&request).unwrap();
        
        assert_eq!(request_deserialized.header.unwrap().storage_user_header.unwrap().ticket_token, "t=0bfusc4tedT0k3n");
        assert_eq!(request_deserialized.body.body.get_profile_request.profile_handle.alias.unwrap().name_space, String::from("MyCidStuff"));
        assert_eq!(request_deserialized.body.body.get_profile_request.profile_attributes.expression_profile_attributes.attachments.unwrap(), true);
    }

    #[test]
    fn test_get_profile_response() {


        let affinity_cache_header = AffinityCacheHeader{ cache_key: Some(String::from("cache_key"))};
        let storage_user_header = StorageUserHeader{ puid: 0, cid: Some(String::from("0")), ticket_token: String::from("token"), is_admin: Some(true), application_id: Some(0), device_id: Some(0), is_trusted_device: Some(false), is_strong_auth: Some(false), language_preference: Some(0), claims: Vec::new() };

        let headers = RequestHeaderContainer{ affinity_cache_header: Some(affinity_cache_header), storage_application_header: None, storage_user_header: Some(storage_user_header) };

        let document = DocumentStream{ document_stream_name: Some(String::from("Name")), mime_type: None, data: Some(String::from("data")), data_size: 123, pre_auth_url: Some(String::from("pre_auth_url")), pre_auth_url_partner: Some(String::from("pre_auth_url_partner")), document_stream_type: String::from("stream type"), write_mode: None, stream_version: None, sha1_hash: None, genie: Some(false), stream_data_status: None, stream_status: None, is_alias_for_default: None, expiration_date_time: Some(String::from("0001-01-01T00:00:00")) };

        let mut document_stream_array : Vec<DocumentStream> = Vec::new();

        document_stream_array.push(document);

        let document_streams = DocumentStreams{ document_stream: document_stream_array };

        let photo = DocumentBaseType{ resource_id: Some(String::from("ressource_id")), name: Some(String::from("name")), item_type: None, date_modified: None, document_streams: document_streams };

        let expression_profile = ExpressionProfile{ resource_id: String::from("ressource_id"), date_modified: String::from("date_modified"), version: Some(205), flags: None, photo: photo, attachments: None, personal_status: Some(String::from("PSM")), personal_status_last_modified:  Some(String::from("PSM_LAST_MODIFIED")), display_name: Some(String::from("DISPLAY NAME")), display_name_last_modified: Some(String::from("DISPLAY_NAME_LAST_MODIFIED")), static_user_tile_public_url:  String::from("STATIC USER TITLE PUBLIC URL") };

        let get_profile_result = GetProfileResultType { resource_id: String::from("ressource_id"), date_modified: String::from("date_modified"), expression_profile: expression_profile };

        let get_profile_response = GetProfileResponse{ get_profile_result: get_profile_result };

        let request_body = GetProfileResponseMessage{ get_profile_response: get_profile_response };

        let body = SoapGetProfileResponseMessage{ body: request_body, fault: None };


        let response = GetProfileResponseMessageSoapEnvelope{ header: Some(headers), body: body };

        let response_serialized = to_string(&response).unwrap();

        println!("DEBUG: {}", response_serialized);
    }

    #[test]
    fn test_update_document_request() {

        let request = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?><soap:Envelope xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:soapenc=\"http://schemas.xmlsoap.org/soap/encoding/\"><soap:Header><AffinityCacheHeader xmlns=\"http://www.msn.com/webservices/storage/2008\"><CacheKey>12r1:8nBBE6vX1J4uPKajtbem5XBIblimCwAhIziAeEAwYD0AMiaztryWvcZthkN9oX_pl2scBKXfKvRvuWKYdHUNuRkgiyV9rzcDpnDIDiM6vdcEB6d82wjjnL4TAFAc5X8i-C94mNfQvujUk470P7fz9qbWfK6ANcEtygDb-oWsYVfEBrxl6geTUg9tGT7yCIsls7ECcLyqwsROuAbWCrued_VPKiUgSIvqG8gaA</CacheKey></AffinityCacheHeader><StorageApplicationHeader xmlns=\"http://www.msn.com/webservices/storage/2008\"><ApplicationID>Messenger Client 9.0</ApplicationID><Scenario>RoamingIdentityChanged</Scenario></StorageApplicationHeader><StorageUserHeader xmlns=\"http://www.msn.com/webservices/storage/2008\"><Puid>0</Puid><TicketToken>t=0bfuc4t3dT0k3n</TicketToken></StorageUserHeader></soap:Header><soap:Body><UpdateDocument xmlns=\"http://www.msn.com/webservices/storage/2008\"><document xsi:type=\"Photo\"><ResourceID>195c865c2afaaa3b!205</ResourceID><Name>Fall</Name><DocumentStreams><DocumentStream xsi:type=\"PhotoStream\"><DocumentStreamType>UserTileStatic</DocumentStreamType><MimeType>image/png</MimeType>
        <Data>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</Data><DataSize>0</DataSize></DocumentStream></DocumentStreams></document></UpdateDocument></soap:Body></soap:Envelope>");
        let request_deserialized : UpdateDocumentMessageSoapEnvelope = from_str(&request).unwrap();

    }

    #[test]
    fn test_update_document_response() {

        let uuid = UUID::new();
        let cache_key = String::from("cachekey");
        let ticket_token = String::from("t0k3n");

        let affinity_cache_header = AffinityCacheHeader{ cache_key: Some(cache_key)};
        let storage_user_header = StorageUserHeader{ puid: 0, cid: Some(uuid.to_hex_cid()), ticket_token: format!("t={}", ticket_token), is_admin: Some(false), application_id: Some(0), device_id: Some(0), is_trusted_device: Some(false), is_strong_auth: Some(false), language_preference: Some(0), claims: Vec::new() };

        let headers = RequestHeaderContainer{ affinity_cache_header: Some(affinity_cache_header), storage_application_header: None, storage_user_header: Some(storage_user_header) };

        let response = UpdateDocumentResponseMessageSoapEnvelope{ header: Some(headers), body: SoapUpdateDocumentResponseMessage{ body: Some(String::new()), fault: None} };
        
        let response_serialized = to_string(&response).unwrap();

        println!("DEBUG: {}", response_serialized);
    
    }

    #[test]
    fn test_update_profile_request() {
        let request = "<?xml version=\"1.0\" encoding=\"utf-8\"?><soap:Envelope xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:soapenc=\"http://schemas.xmlsoap.org/soap/encoding/\"><soap:Header><AffinityCacheHeader xmlns=\"http://www.msn.com/webservices/storage/2008\"><CacheKey>12r1:8nBBE6vX1J4uPKajtbem5XBIblimCwAhIziAeEAwYD0AMiaztryWvcZthkN9oX_pl2scBKXfKvRvuWKYdHUNuRkgiyV9rzcDpnDIDiM6vdcEB6d82wjjnL4TAFAjc5X8i-C94mNfQvujUk470P7fz9qbWfK6ANcEtygDb-oWsYVfEBrxl6geTUg9tGT7yCIsls7ECcLyqwsROuAbWCrued_VPKiUgSIvqG8gaA</CacheKey></AffinityCacheHeader><StorageApplicationHeader xmlns=\"http://www.msn.com/webservices/storage/2008\"><ApplicationID>Messenger Client 9.0</ApplicationID><Scenario>RoamingIdentityChanged</Scenario></StorageApplicationHeader><StorageUserHeader xmlns=\"http://www.msn.com/webservices/storage/2008\"><Puid>0</Puid><TicketToken>t=0bfusc4t3dT0k3n</TicketToken></StorageUserHeader></soap:Header><soap:Body><UpdateProfile xmlns=\"http://www.msn.com/webservices/storage/2008\"><profile><ResourceID>195c865c2afaaa3b!106</ResourceID><ExpressionProfile><FreeText>Update</FreeText><DisplayName>aeonclf</DisplayName><Flags>0</Flags></ExpressionProfile></profile><profileAttributesToDelete><ExpressionProfileAttributes><PersonalStatus>true</PersonalStatus></ExpressionProfileAttributes></profileAttributesToDelete></UpdateProfile></soap:Body></soap:Envelope>";
        let request_deserialized : UpdateProfileMessageSoapEnvelope = from_str(request).unwrap();

    }


    #[test]
    fn test_update_profile_response() {

        let uuid = UUID::new();
        let cache_key = String::from("cachekey");
        let ticket_token = String::from("t0k3n");

        let affinity_cache_header = AffinityCacheHeader{ cache_key: Some(cache_key)};
        let storage_user_header = StorageUserHeader{ puid: 0, cid: Some(uuid.to_hex_cid()), ticket_token: format!("t={}", ticket_token), is_admin: Some(false), application_id: Some(0), device_id: Some(0), is_trusted_device: Some(false), is_strong_auth: Some(false), language_preference: Some(0), claims: Vec::new() };

        let headers = RequestHeaderContainer{ affinity_cache_header: Some(affinity_cache_header), storage_application_header: None, storage_user_header: Some(storage_user_header) };

      let response =  UpdateProfileResponseMessageSoapEnvelope { header: Some(headers), body: SoapUpdateProfileResponseMessage { body: Some(String::new()), fault: None }};
      
      let response_serialized = to_string(&response).unwrap();

      println!("DEBUG: {}", response_serialized);
    }
 
    #[test]
    fn test_delete_relationships_request() {
        let request = "<?xml version=\"1.0\" encoding=\"utf-8\"?><soap:Envelope xmlns:soap=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\" xmlns:soapenc=\"http://schemas.xmlsoap.org/soap/encoding/\"><soap:Header><AffinityCacheHeader xmlns=\"http://www.msn.com/webservices/storage/2008\"><CacheKey>12r1:8nBBE6vX1J4uPKajtbem5XBIblimCwAhIziAeEAwYD0AMiaztryWvcZthkN9oX_pl2scBKXfKvRvuWKYdHUNuRkgiyV9rzcDpnDIDiM6vdcEB6d82wjjnL4TAFAjc5X8i-C94mNfQvujUk470P7fz9qbWfK6ANcEtygDb-oWsYVfEBrxl6geTUg9tGT7yCIsls7ECcLyqwsROuAbWCrued_VPKiUgSIvqG8gaA</CacheKey></AffinityCacheHeader><StorageApplicationHeader xmlns=\"http://www.msn.com/webservices/storage/2008\"><ApplicationID>Messenger Client 9.0</ApplicationID><Scenario>RoamingIdentityChanged</Scenario></StorageApplicationHeader><StorageUserHeader xmlns=\"http://www.msn.com/webservices/storage/2008\"><Puid>0</Puid><TicketToken>t=0busc4t3dT0k3n</TicketToken></StorageUserHeader></soap:Header><soap:Body><DeleteRelationships xmlns=\"http://www.msn.com/webservices/storage/2008\"><sourceHandle><ResourceID>195c865c2afaaa3b!118</ResourceID></sourceHandle><targetHandles><ObjectHandle><ResourceID>195c865c2afaaa3b!205</ResourceID></ObjectHandle></targetHandles></DeleteRelationships></soap:Body></soap:Envelope>";
        let request_deserialized : DeleteRelationshipsMessageSoapEnvelope = from_str(request).unwrap();
    }

    #[test]
    fn test_delete_relationships_response() {
        
        let uuid = UUID::new();
        let cache_key = String::from("cachekey");
        let ticket_token = String::from("t0k3n");

        let affinity_cache_header = AffinityCacheHeader{ cache_key: Some(cache_key)};
        let storage_user_header = StorageUserHeader{ puid: 0, cid: Some(uuid.to_hex_cid()), ticket_token: format!("t={}", ticket_token), is_admin: Some(false), application_id: Some(0), device_id: Some(0), is_trusted_device: Some(false), is_strong_auth: Some(false), language_preference: Some(0), claims: Vec::new() };

        let headers = RequestHeaderContainer{ affinity_cache_header: Some(affinity_cache_header), storage_application_header: None, storage_user_header: Some(storage_user_header) };

      let response =  DeleteRelationshipsResponseMessageSoapEnvelope { header: Some(headers), body: SoapDeleteRelationshipsResponseMessage { body: Some(String::new()), fault: None }};
      
      let response_serialized = to_string(&response).unwrap();

      println!("DEBUG: {}", response_serialized);
    }



}
