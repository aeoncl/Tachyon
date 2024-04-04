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
)]
pub struct GetProfileMessage {
	#[yaserde(flatten, default)]
	pub get_profile_request: types::GetProfile, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "GetProfileResponseMessage",
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
	rename = "UpdateProfileResponseMessage",
)]
pub struct UpdateProfileResponseMessage {
	#[yaserde(flatten, default)]
	pub update_profile_response: types::UpdateProfileResponse, 
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
	rename = "UpdateDocumentResponseMessage",
)]
pub struct UpdateDocumentResponseMessage {
	#[yaserde(flatten, default)]
	pub update_document_response: types::UpdateDocumentResponse, 
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
	rename = "CreateRelationshipsResponseMessage",
)]
pub struct CreateRelationshipsResponseMessage {
	#[yaserde(flatten, default)]
	pub create_relationships_response: types::CreateRelationshipsResponse, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "DeleteRelationshipsMessage",
)]
pub struct DeleteRelationshipsMessage {
	#[yaserde(flatten, default)]
	pub delete_relationships: types::DeleteRelationships, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "DeleteRelationshipsResponseMessage",
)]
pub struct DeleteRelationshipsResponseMessage {
	#[yaserde(flatten, default)]
	pub delete_relationships_response: types::DeleteRelationshipsResponse, 
}
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
	namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
	prefix = "nsi1",
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
	namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
	prefix = "nsi1",
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
	namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
	prefix = "nsi1",
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
	namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
	prefix = "nsi1",
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
            pub type GetProfileMessage = messages::GetProfileMessage;

pub type GetProfileResponseMessage = messages::GetProfileResponseMessage;

pub type UpdateProfileMessage = messages::UpdateProfileMessage;

pub type UpdateProfileResponseMessage = messages::UpdateProfileResponseMessage;

pub type FindDocumentsMessage = messages::FindDocumentsMessage;

pub type FindDocumentsResponseMessage = messages::FindDocumentsResponseMessage;

pub type CreateProfileMessage = messages::CreateProfileMessage;

pub type CreateProfileResponseMessage = messages::CreateProfileResponseMessage;

pub type ShareItemMessage = messages::ShareItemMessage;

pub type ShareItemResponseMessage = messages::ShareItemResponseMessage;

pub type CreateDocumentMessage = messages::CreateDocumentMessage;

pub type CreateDocumentResponseMessage = messages::CreateDocumentResponseMessage;

pub type UpdateDocumentMessage = messages::UpdateDocumentMessage;

pub type UpdateDocumentResponseMessage = messages::UpdateDocumentResponseMessage;

pub type CreateRelationshipsMessage = messages::CreateRelationshipsMessage;

pub type CreateRelationshipsResponseMessage = messages::CreateRelationshipsResponseMessage;

pub type DeleteRelationshipsMessage = messages::DeleteRelationshipsMessage;

pub type DeleteRelationshipsResponseMessage = messages::DeleteRelationshipsResponseMessage;

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
            namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soapenv"
        )]
        pub struct GetProfileMessageSoapEnvelope {
            #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
            pub encoding_style: String,
            #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
            pub tnsattr: Option<String>,
            #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
            pub urnattr: Option<String>,
            #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
            pub xsiattr: Option<String>,
            #[yaserde(rename = "Header", prefix = "soapenv")]
            pub header: Option<Header>,
            #[yaserde(rename = "Body", prefix = "soapenv")]
            pub body: SoapGetProfileMessage,
        }
        
        impl GetProfileMessageSoapEnvelope {
            pub fn new(body: SoapGetProfileMessage) -> Self {
                GetProfileMessageSoapEnvelope {
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
                    pub struct SoapGetProfileResponseMessage {
                    #[yaserde(rename = "GetProfileResponseMessage", default)]
                    pub body: ports::GetProfileResponseMessage,
                         #[yaserde(rename = "Fault", default)]
                            pub fault: Option<SoapFault>,
                            
                }
                #[derive(Debug, Default, YaSerialize, YaDeserialize)]
        #[yaserde(
            rename = "Envelope",
            namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soapenv"
        )]
        pub struct GetProfileResponseMessageSoapEnvelope {
            #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
            pub encoding_style: String,
            #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
            pub tnsattr: Option<String>,
            #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
            pub urnattr: Option<String>,
            #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
            pub xsiattr: Option<String>,
            #[yaserde(rename = "Header", prefix = "soapenv")]
            pub header: Option<Header>,
            #[yaserde(rename = "Body", prefix = "soapenv")]
            pub body: SoapGetProfileResponseMessage,
        }
        
        impl GetProfileResponseMessageSoapEnvelope {
            pub fn new(body: SoapGetProfileResponseMessage) -> Self {
                GetProfileResponseMessageSoapEnvelope {
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
                    pub struct SoapUpdateProfileMessage {
                        #[yaserde(rename = "UpdateProfile", default)]
                        pub body: ports::UpdateProfileMessage,
                        #[yaserde(attribute)]
                        pub xmlns: Option<String>,
                    }
                    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
        #[yaserde(
            rename = "Envelope",
            namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soapenv"
        )]
        pub struct UpdateProfileMessageSoapEnvelope {
            #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
            pub encoding_style: String,
            #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
            pub tnsattr: Option<String>,
            #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
            pub urnattr: Option<String>,
            #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
            pub xsiattr: Option<String>,
            #[yaserde(rename = "Header", prefix = "soapenv")]
            pub header: Option<Header>,
            #[yaserde(rename = "Body", prefix = "soapenv")]
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
                    pub body: ports::UpdateProfileResponseMessage,
                         #[yaserde(rename = "Fault", default)]
                            pub fault: Option<SoapFault>,
                            
                }
                #[derive(Debug, Default, YaSerialize, YaDeserialize)]
        #[yaserde(
            rename = "Envelope",
            namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soapenv"
        )]
        pub struct UpdateProfileResponseMessageSoapEnvelope {
            #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
            pub encoding_style: String,
            #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
            pub tnsattr: Option<String>,
            #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
            pub urnattr: Option<String>,
            #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
            pub xsiattr: Option<String>,
            #[yaserde(rename = "Header", prefix = "soapenv")]
            pub header: Option<Header>,
            #[yaserde(rename = "Body", prefix = "soapenv")]
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
            namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soapenv"
        )]
        pub struct FindDocumentsMessageSoapEnvelope {
            #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
            pub encoding_style: String,
            #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
            pub tnsattr: Option<String>,
            #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
            pub urnattr: Option<String>,
            #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
            pub xsiattr: Option<String>,
            #[yaserde(rename = "Header", prefix = "soapenv")]
            pub header: Option<Header>,
            #[yaserde(rename = "Body", prefix = "soapenv")]
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
            namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soapenv"
        )]
        pub struct FindDocumentsResponseMessageSoapEnvelope {
            #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
            pub encoding_style: String,
            #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
            pub tnsattr: Option<String>,
            #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
            pub urnattr: Option<String>,
            #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
            pub xsiattr: Option<String>,
            #[yaserde(rename = "Header", prefix = "soapenv")]
            pub header: Option<Header>,
            #[yaserde(rename = "Body", prefix = "soapenv")]
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
            namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soapenv"
        )]
        pub struct CreateProfileMessageSoapEnvelope {
            #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
            pub encoding_style: String,
            #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
            pub tnsattr: Option<String>,
            #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
            pub urnattr: Option<String>,
            #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
            pub xsiattr: Option<String>,
            #[yaserde(rename = "Header", prefix = "soapenv")]
            pub header: Option<Header>,
            #[yaserde(rename = "Body", prefix = "soapenv")]
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
            namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soapenv"
        )]
        pub struct CreateProfileResponseMessageSoapEnvelope {
            #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
            pub encoding_style: String,
            #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
            pub tnsattr: Option<String>,
            #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
            pub urnattr: Option<String>,
            #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
            pub xsiattr: Option<String>,
            #[yaserde(rename = "Header", prefix = "soapenv")]
            pub header: Option<Header>,
            #[yaserde(rename = "Body", prefix = "soapenv")]
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
            namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soapenv"
        )]
        pub struct ShareItemMessageSoapEnvelope {
            #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
            pub encoding_style: String,
            #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
            pub tnsattr: Option<String>,
            #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
            pub urnattr: Option<String>,
            #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
            pub xsiattr: Option<String>,
            #[yaserde(rename = "Header", prefix = "soapenv")]
            pub header: Option<Header>,
            #[yaserde(rename = "Body", prefix = "soapenv")]
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
            namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soapenv"
        )]
        pub struct ShareItemResponseMessageSoapEnvelope {
            #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
            pub encoding_style: String,
            #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
            pub tnsattr: Option<String>,
            #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
            pub urnattr: Option<String>,
            #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
            pub xsiattr: Option<String>,
            #[yaserde(rename = "Header", prefix = "soapenv")]
            pub header: Option<Header>,
            #[yaserde(rename = "Body", prefix = "soapenv")]
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
            namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soapenv"
        )]
        pub struct CreateDocumentMessageSoapEnvelope {
            #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
            pub encoding_style: String,
            #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
            pub tnsattr: Option<String>,
            #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
            pub urnattr: Option<String>,
            #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
            pub xsiattr: Option<String>,
            #[yaserde(rename = "Header", prefix = "soapenv")]
            pub header: Option<Header>,
            #[yaserde(rename = "Body", prefix = "soapenv")]
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
            namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soapenv"
        )]
        pub struct CreateDocumentResponseMessageSoapEnvelope {
            #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
            pub encoding_style: String,
            #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
            pub tnsattr: Option<String>,
            #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
            pub urnattr: Option<String>,
            #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
            pub xsiattr: Option<String>,
            #[yaserde(rename = "Header", prefix = "soapenv")]
            pub header: Option<Header>,
            #[yaserde(rename = "Body", prefix = "soapenv")]
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
            namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soapenv"
        )]
        pub struct UpdateDocumentMessageSoapEnvelope {
            #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
            pub encoding_style: String,
            #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
            pub tnsattr: Option<String>,
            #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
            pub urnattr: Option<String>,
            #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
            pub xsiattr: Option<String>,
            #[yaserde(rename = "Header", prefix = "soapenv")]
            pub header: Option<Header>,
            #[yaserde(rename = "Body", prefix = "soapenv")]
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
                    pub body: ports::UpdateDocumentResponseMessage,
                         #[yaserde(rename = "Fault", default)]
                            pub fault: Option<SoapFault>,
                            
                }
                #[derive(Debug, Default, YaSerialize, YaDeserialize)]
        #[yaserde(
            rename = "Envelope",
            namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soapenv"
        )]
        pub struct UpdateDocumentResponseMessageSoapEnvelope {
            #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
            pub encoding_style: String,
            #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
            pub tnsattr: Option<String>,
            #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
            pub urnattr: Option<String>,
            #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
            pub xsiattr: Option<String>,
            #[yaserde(rename = "Header", prefix = "soapenv")]
            pub header: Option<Header>,
            #[yaserde(rename = "Body", prefix = "soapenv")]
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
            namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soapenv"
        )]
        pub struct CreateRelationshipsMessageSoapEnvelope {
            #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
            pub encoding_style: String,
            #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
            pub tnsattr: Option<String>,
            #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
            pub urnattr: Option<String>,
            #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
            pub xsiattr: Option<String>,
            #[yaserde(rename = "Header", prefix = "soapenv")]
            pub header: Option<Header>,
            #[yaserde(rename = "Body", prefix = "soapenv")]
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
                    pub body: ports::CreateRelationshipsResponseMessage,
                         #[yaserde(rename = "Fault", default)]
                            pub fault: Option<SoapFault>,
                            
                }
                #[derive(Debug, Default, YaSerialize, YaDeserialize)]
        #[yaserde(
            rename = "Envelope",
            namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soapenv"
        )]
        pub struct CreateRelationshipsResponseMessageSoapEnvelope {
            #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
            pub encoding_style: String,
            #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
            pub tnsattr: Option<String>,
            #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
            pub urnattr: Option<String>,
            #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
            pub xsiattr: Option<String>,
            #[yaserde(rename = "Header", prefix = "soapenv")]
            pub header: Option<Header>,
            #[yaserde(rename = "Body", prefix = "soapenv")]
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
            namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soapenv"
        )]
        pub struct DeleteRelationshipsMessageSoapEnvelope {
            #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
            pub encoding_style: String,
            #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
            pub tnsattr: Option<String>,
            #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
            pub urnattr: Option<String>,
            #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
            pub xsiattr: Option<String>,
            #[yaserde(rename = "Header", prefix = "soapenv")]
            pub header: Option<Header>,
            #[yaserde(rename = "Body", prefix = "soapenv")]
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
                    pub body: ports::DeleteRelationshipsResponseMessage,
                         #[yaserde(rename = "Fault", default)]
                            pub fault: Option<SoapFault>,
                            
                }
                #[derive(Debug, Default, YaSerialize, YaDeserialize)]
        #[yaserde(
            rename = "Envelope",
            namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
            prefix = "soapenv"
        )]
        pub struct DeleteRelationshipsResponseMessageSoapEnvelope {
            #[yaserde(rename = "encodingStyle", prefix = "soapenv", attribute)]
            pub encoding_style: String,
            #[yaserde(rename = "tns", prefix = "xmlns", attribute)]
            pub tnsattr: Option<String>,
            #[yaserde(rename = "urn", prefix = "xmlns", attribute)]
            pub urnattr: Option<String>,
            #[yaserde(rename = "xsi", prefix = "xmlns", attribute)]
            pub xsiattr: Option<String>,
            #[yaserde(rename = "Header", prefix = "soapenv")]
            pub header: Option<Header>,
            #[yaserde(rename = "Body", prefix = "soapenv")]
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

