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
	rename = "UpdateProfileMessage",
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
	rename = "UpdateDocumentMessage",
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
	rename = "DeleteRelationshipsMessage",
)]
pub struct DeleteRelationshipsMessage {
	#[yaserde(flatten, default)]
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
	pub cid: Option<i64>,
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
    pub language_preference: Option<i32>
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
	pub personal_status: String, 
	#[yaserde(rename = "PersonalStatusLastModified", prefix="nsi1")]
	pub personal_status_last_modified: String, 
	#[yaserde(rename = "DisplayName", prefix="nsi1")]
	pub display_name: String, 
	#[yaserde(rename = "DisplayNameLastModified", prefix="nsi1")]
	pub display_name_last_modified: String, 
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
)]
pub struct UpdateProfileRequestType {
	#[yaserde(rename = "profile", default)]
	pub profile: Profile, 
	#[yaserde(rename = "profileAttributesToDelete", default)]
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
)]
pub struct UpdateDocumentRequestType {
	#[yaserde(rename = "document", default)]
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
)]
pub struct DeleteRelationshipsRequestType {
	#[yaserde(rename = "sourceHandle", default)]
	pub source_handle: Handle, 
	#[yaserde(rename = "targetHandles", default)]
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
use yaserde::{YaSerialize, YaDeserialize};
            use yaserde::de::from_str;
            use async_trait::async_trait;
            use yaserde::ser::to_string;
            use super::*;
            
            impl StorageServiceBinding {
                async fn send_soap_request<T: YaSerialize>(&self, request: &T, action: &str) -> SoapResponse {
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
            prefix = "soap"
        )]
        pub struct UpdateProfileMessageSoapEnvelope {
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
            pub body: SoapUpdateProfileMessage,
        }
        
        impl UpdateProfileMessageSoapEnvelope {
            pub fn new(body: SoapUpdateProfileMessage) -> Self {
                UpdateProfileMessageSoapEnvelope {
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
                    pub struct SoapUpdateProfileResponseMessage {
                    #[yaserde(rename = "UpdateProfileResponseMessage", default)]
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
        pub struct UpdateProfileResponseMessageSoapEnvelope {
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
            pub body: SoapUpdateProfileResponseMessage,
        }
        
        impl UpdateProfileResponseMessageSoapEnvelope {
            pub fn new(body: SoapUpdateProfileResponseMessage) -> Self {
                UpdateProfileResponseMessageSoapEnvelope {
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
            prefix = "soap"
        )]
        pub struct UpdateDocumentMessageSoapEnvelope {
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
            pub body: SoapUpdateDocumentMessage,
        }
        
        impl UpdateDocumentMessageSoapEnvelope {
            pub fn new(body: SoapUpdateDocumentMessage) -> Self {
                UpdateDocumentMessageSoapEnvelope {
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
                    pub struct SoapUpdateDocumentResponseMessage {
                    #[yaserde(rename = "UpdateDocumentResponseMessage", default)]
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
        pub struct UpdateDocumentResponseMessageSoapEnvelope {
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
            pub body: SoapUpdateDocumentResponseMessage,
        }
        
        impl UpdateDocumentResponseMessageSoapEnvelope {
            pub fn new(body: SoapUpdateDocumentResponseMessage) -> Self {
                UpdateDocumentResponseMessageSoapEnvelope {
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
            prefix = "soap"
        )]
        pub struct DeleteRelationshipsMessageSoapEnvelope {
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
            pub body: SoapDeleteRelationshipsMessage,
        }
        
        impl DeleteRelationshipsMessageSoapEnvelope {
            pub fn new(body: SoapDeleteRelationshipsMessage) -> Self {
                DeleteRelationshipsMessageSoapEnvelope {
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
                    pub struct SoapDeleteRelationshipsResponseMessage {
                    #[yaserde(rename = "DeleteRelationshipsResponseMessage", default)]
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
        pub struct DeleteRelationshipsResponseMessageSoapEnvelope {
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
            pub body: SoapDeleteRelationshipsResponseMessage,
        }
        
        impl DeleteRelationshipsResponseMessageSoapEnvelope {
            pub fn new(body: SoapDeleteRelationshipsResponseMessage) -> Self {
                DeleteRelationshipsResponseMessageSoapEnvelope {
                    encoding_style: SOAP_ENCODING.to_string(),
                    tnsattr: Option::Some("http://www.msn.com/webservices/storage/2008".to_string()),
                    body,
                    urnattr: None,
                    xsiattr: None,
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

    use super::bindings::GetProfileResponseMessageSoapEnvelope;


    pub struct GetProfileResponseFactory;

    impl GetProfileResponseFactory {

        pub fn get_empty_response(uuid: UUID, cache_key: String, matrix_token: String, display_name: String, psm: String) -> GetProfileResponseMessageSoapEnvelope {


            let now = Local::now();

            let affinity_cache_header = AffinityCacheHeader{ cache_key: Some(cache_key)};
            let storage_user_header = StorageUserHeader{ puid: 0, cid: Some(uuid.to_decimal_cid()), ticket_token: format!("t={}", matrix_token), is_admin: Some(true), application_id: Some(0), device_id: Some(0), is_trusted_device: Some(false), is_strong_auth: Some(false), language_preference: Some(0) };
    
            let headers = RequestHeaderContainer{ affinity_cache_header: Some(affinity_cache_header), storage_application_header: None, storage_user_header: Some(storage_user_header) };
    
            let profile_pic_stream = DocumentStream{ document_stream_name: Some(String::from("UserTileSmall")), mime_type: None, data: None, data_size: 0, pre_auth_url: Some(String::from("http://127.0.0.1/storage/usertile/689f6a4e-d8bb-568d-a1c9-da0398db7d8a/small")), pre_auth_url_partner: None, document_stream_type: String::from("Named"), write_mode: Some(String::from("Overwrite")), stream_version: Some(0), sha1_hash: None, genie: Some(false), stream_data_status: Some(String::from("None")), stream_status: Some(String::from("None")), is_alias_for_default:Some(false)};

            let mut document_stream_array : Vec<DocumentStream> = Vec::new();
        
            document_stream_array.push(profile_pic_stream);

            let document_streams = DocumentStreams{ document_stream: document_stream_array };
    
            let photo = DocumentBaseType{ resource_id: Some(format!("{}!205", uuid.to_decimal_cid())), name: None, item_type: None, date_modified: None, document_streams: document_streams };
    
            let expression_profile = ExpressionProfile{ resource_id: format!("{}!118", uuid.to_decimal_cid()), date_modified: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(), version: Some(205), flags: None, photo: photo, attachments: None, personal_status: psm, personal_status_last_modified:  now.format("%Y-%m-%dT%H:%M:%SZ").to_string(), display_name: display_name, display_name_last_modified: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(), static_user_tile_public_url:  format!("http://127.0.0.1/storage/usertile/{}/static", uuid.to_string()) };
    
            let get_profile_result = GetProfileResultType { resource_id: format!("{}!106", uuid.to_decimal_cid()), date_modified: now.format("%Y-%m-%dT%H:%M:%SZ").to_string(), expression_profile: expression_profile };
    
            let get_profile_response = GetProfileResponse{ get_profile_result: get_profile_result };
    
            let request_body = GetProfileResponseMessage{ get_profile_response: get_profile_response };
    
            let body = SoapGetProfileResponseMessage{ body: request_body, fault: None };
    
    
            return GetProfileResponseMessageSoapEnvelope{ header: Some(headers), body: body };
        }

    }

}

#[cfg(test)]
mod tests {
    use log::{warn, debug};
    use yaserde::de::from_str;
    use yaserde::ser::to_string;

    use crate::generated::msnstorage_datatypes::types::{DocumentBaseType, DocumentStreams, DocumentStream};

    use super::{bindings::{GetProfileMessageSoapEnvelope, GetProfileResponseMessageSoapEnvelope, SoapGetProfileResponseMessage}, RequestHeaderContainer, types::{AffinityCacheHeader, StorageApplicationHeader, StorageUserHeader, GetProfileResponse, GetProfileResultType, ExpressionProfile}, messages::GetProfileResponseMessage};


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
        let storage_user_header = StorageUserHeader{ puid: 0, cid: Some(-1), ticket_token: String::from("token"), is_admin: Some(true), application_id: Some(0), device_id: Some(0), is_trusted_device: Some(false), is_strong_auth: Some(false), language_preference: Some(0)};

        let headers = RequestHeaderContainer{ affinity_cache_header: Some(affinity_cache_header), storage_application_header: None, storage_user_header: Some(storage_user_header) };

        let document = DocumentStream{ document_stream_name: Some(String::from("Name")), mime_type: None, data: Some(String::from("data")), data_size: 123, pre_auth_url: Some(String::from("pre_auth_url")), pre_auth_url_partner: Some(String::from("pre_auth_url_partner")), document_stream_type: String::from("stream type"), write_mode: None, stream_version: None, sha1_hash: None, genie: Some(false), stream_data_status: None, stream_status: None, is_alias_for_default: None};

        let mut document_stream_array : Vec<DocumentStream> = Vec::new();

        document_stream_array.push(document);

        let document_streams = DocumentStreams{ document_stream: document_stream_array };

        let photo = DocumentBaseType{ resource_id: Some(String::from("ressource_id")), name: Some(String::from("name")), item_type: None, date_modified: None, document_streams: document_streams };

        let expression_profile = ExpressionProfile{ resource_id: String::from("ressource_id"), date_modified: String::from("date_modified"), version: Some(205), flags: None, photo: photo, attachments: None, personal_status: String::from("PSM"), personal_status_last_modified:  String::from("PSM_LAST_MODIFIED"), display_name: String::from("DISPLAY NAME"), display_name_last_modified: String::from("DISPLAY_NAME_LAST_MODIFIED"), static_user_tile_public_url:  String::from("STATIC USER TITLE PUBLIC URL") };

        let get_profile_result = GetProfileResultType { resource_id: String::from("ressource_id"), date_modified: String::from("date_modified"), expression_profile: expression_profile };

        let get_profile_response = GetProfileResponse{ get_profile_result: get_profile_result };

        let request_body = GetProfileResponseMessage{ get_profile_response: get_profile_response };

        let body = SoapGetProfileResponseMessage{ body: request_body, fault: None };


        let response = GetProfileResponseMessageSoapEnvelope{ header: Some(headers), body: body };

        let response_serialized = to_string(&response).unwrap();

        println!("DEBUG: {}", response_serialized);
    }

}
