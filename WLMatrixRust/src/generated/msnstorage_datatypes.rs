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
	rename = "Alias",
	namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	prefix = "nsi1",
    default_namespace="nsi1"
)]
pub struct Alias {
	#[yaserde(rename = "Name", prefix="nsi1")]
	pub name: String, 
	#[yaserde(rename = "NameSpace", prefix="nsi1")]
	pub name_space: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "Handle",
	namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	prefix = "nsi1",
    default_namespace="nsi1"
)]
pub struct Handle {
	#[yaserde(rename = "Alias", prefix="nsi1")]
	pub alias: Option<Alias>, 
	#[yaserde(rename = "RelationshipName",  prefix="nsi1")]
	pub relationship_name: Option<String>, 
	#[yaserde(rename = "ResourceID",  prefix="nsi1")]
	pub resource_id: Option<String>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "profileAttributes",
	namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	prefix = "nsi1",
    default_namespace="nsi1"
)]
pub struct ProfileAttributes {
	#[yaserde(rename = "ResourceID", prefix="nsi1")]
	pub resource_id: bool, 
	#[yaserde(rename = "DateModified", prefix="nsi1")]
	pub date_modified: bool, 
	#[yaserde(rename = "ExpressionProfileAttributes", prefix="nsi1")]
	pub expression_profile_attributes: ExpressionProfileAttributesType, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "DocumentStream",
	namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	prefix = "nsi1",
    default_namespace="nsi1"
)]
pub struct DocumentStream {
	#[yaserde(rename = "DocumentStreamName", prefix="nsi1")]
	pub document_stream_name: Option<String>, 
	#[yaserde(rename = "MimeType", prefix="nsi1")]
	pub mime_type: Option<String>, 
	#[yaserde(rename = "Data", prefix="nsi1")]
	pub data: Option<String>, 
	#[yaserde(rename = "DataSize", prefix="nsi1")]
	pub data_size: i32, 
	#[yaserde(rename = "PreAuthURL", prefix="nsi1")]
	pub pre_auth_url: Option<String>, 
	#[yaserde(rename = "PreAuthURLPartner", prefix="nsi1")]
	pub pre_auth_url_partner: Option<String>, 
	#[yaserde(rename = "DocumentStreamType", prefix="nsi1")]
	pub document_stream_type: String, 
	#[yaserde(rename = "WriteMode", prefix="nsi1")]
	pub write_mode: Option<String>, 
	#[yaserde(rename = "StreamVersion", prefix="nsi1")]
	pub stream_version: Option<i32>, 
	#[yaserde(rename = "SHA1Hash", prefix="nsi1")]
	pub sha1_hash: Option<String>, 
	#[yaserde(rename = "Genie", prefix="nsi1")]
	pub genie: Option<bool>,
	#[yaserde(rename = "StreamDataStatus", prefix="nsi1")]
	pub stream_data_status: Option<String>,
	#[yaserde(rename = "StreamStatus", prefix="nsi1")]
	pub stream_status: Option<String>,
	#[yaserde(rename = "IsAliasForDefault", prefix="nsi1")]
	pub is_alias_for_default: Option<bool>
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "PhotoStream",
)]
pub struct PhotoStream {
	#[yaserde(flatten, default)]
	pub document_stream: DocumentStream, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
	#[yaserde(rename = "SizeX", default)]
	pub size_x: Option<i32>, 
	#[yaserde(rename = "SizeY", default)]
	pub size_y: Option<i32>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "Relationship",
)]
pub struct Relationship {
	#[yaserde(rename = "SourceID", default)]
	pub source_id: String, 
	#[yaserde(rename = "SourceType", default)]
	pub source_type: String, 
	#[yaserde(rename = "TargetID", default)]
	pub target_id: String, 
	#[yaserde(rename = "TargetType", default)]
	pub target_type: String, 
	#[yaserde(rename = "RelationshipName", default)]
	pub relationship_name: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ExpressionProfileAttributesType",
	namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	prefix = "nsi1",
    default_namespace="nsi1"
)]
pub struct ExpressionProfileAttributesType {
	#[yaserde(rename = "ResourceID", prefix="nsi1")]
	pub resource_id: Option<bool>, 
	#[yaserde(rename = "DateModified", prefix="nsi1")]
	pub date_modified: Option<bool>, 
	#[yaserde(rename = "DisplayName", prefix="nsi1")]
	pub display_name: Option<bool>, 
	#[yaserde(rename = "DisplayNameLastModified", prefix="nsi1")]
	pub display_name_last_modified: Option<bool>, 
	#[yaserde(rename = "PersonalStatus", prefix="nsi1")]
	pub personal_status: Option<bool>, 
	#[yaserde(rename = "PersonalStatusLastModified", prefix="nsi1")]
	pub personal_status_last_modified: Option<bool>, 
	#[yaserde(rename = "StaticUserTilePublicURL", prefix="nsi1")]
	pub static_user_tile_public_url: Option<bool>, 
	#[yaserde(rename = "Photo", prefix="nsi1")]
	pub photo: Option<bool>, 
	#[yaserde(rename = "Attachments", prefix="nsi1")]
	pub attachments: Option<bool>, 
	#[yaserde(rename = "Flag", prefix="nsi1")]
	pub flag: Option<bool>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "DocumentStreams",
	namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	prefix = "nsi1",
    default_namespace="nsi1"
)]
pub struct DocumentStreams {
	#[yaserde(rename = "DocumentStream", prefix="nsi1")]
	pub document_stream: Vec<DocumentStream>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "documentBaseType",
	namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
	prefix = "nsi1",
    default_namespace="nsi1"
)]
pub struct DocumentBaseType {
	#[yaserde(rename = "ResourceID", prefix="nsi1")]
	pub resource_id: Option<String>, 
	#[yaserde(rename = "Name", prefix="nsi1")]
	pub name: Option<String>, 
	#[yaserde(rename = "ItemType", prefix="nsi1")]
	pub item_type: Option<String>, 
	#[yaserde(rename = "DateModified", prefix="nsi1")]
	pub date_modified: Option<String>, 
	#[yaserde(rename = "DocumentStreams", prefix="nsi1")]
	pub document_streams: DocumentStreams, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "Photo",
)]
pub struct Photo {
	#[yaserde(flatten, default)]
	pub document_base_type: DocumentBaseType, 
#[yaserde(prefix = "xsi", rename="type", attribute)]
pub xsi_type: String,
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "ExpressionProfile",
)]
pub struct ExpressionProfile {
	#[yaserde(rename = "FreeText", default)]
	pub free_text: Option<String>, 
	#[yaserde(rename = "DisplayName", default)]
	pub display_name: Option<String>, 
	#[yaserde(rename = "PersonalStatus", default)]
	pub personal_status: Option<String>, 
	#[yaserde(rename = "Flags", default)]
	pub flags: Option<i32>, 
	#[yaserde(rename = "RoleDefinitionName", default)]
	pub role_definition_name: Option<String>, 
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

