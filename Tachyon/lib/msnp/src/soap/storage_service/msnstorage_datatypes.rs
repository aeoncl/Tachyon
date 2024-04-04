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
	rename = "Alias",
)]
pub struct Alias {
	#[yaserde(rename = "Name", default)]
	pub name: String, 
	#[yaserde(rename = "NameSpace", default)]
	pub name_space: String, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "Handle",
)]
pub struct Handle {
	#[yaserde(rename = "Alias", default)]
	pub alias: Option<Alias>, 
	#[yaserde(rename = "RelationshipName", default)]
	pub relationship_name: Option<String>, 
	#[yaserde(rename = "ResourceID", default)]
	pub resource_id: Option<String>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "profileAttributes",
)]
pub struct ProfileAttributes {
	#[yaserde(rename = "ResourceID", default)]
	pub resource_id: bool, 
	#[yaserde(rename = "DateModified", default)]
	pub date_modified: bool, 
	#[yaserde(rename = "ExpressionProfileAttributes", default)]
	pub expression_profile_attributes: ExpressionProfileAttributesType, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "DocumentStream",
)]
pub struct DocumentStream {
	#[yaserde(rename = "DocumentStreamName", default)]
	pub document_stream_name: Option<String>, 
	#[yaserde(rename = "MimeType", default)]
	pub mime_type: String, 
	#[yaserde(rename = "Data", default)]
	pub data: Option<String>, 
	#[yaserde(rename = "DataSize", default)]
	pub data_size: i32, 
	#[yaserde(rename = "PreAuthURL", default)]
	pub pre_auth_url: Option<String>, 
	#[yaserde(rename = "PreAuthURLPartner", default)]
	pub pre_auth_url_partner: Option<String>, 
	#[yaserde(rename = "DocumentStreamType", default)]
	pub document_stream_type: String, 
	#[yaserde(rename = "WriteMode", default)]
	pub write_mode: Option<String>, 
	#[yaserde(rename = "StreamVersion", default)]
	pub stream_version: Option<i32>, 
	#[yaserde(rename = "SHA1Hash", default)]
	pub sha1_hash: Option<String>, 
	#[yaserde(rename = "Genie", default)]
	pub genie: Option<bool>, 
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
)]
pub struct ExpressionProfileAttributesType {
	#[yaserde(rename = "ResourceID", default)]
	pub resource_id: Option<bool>, 
	#[yaserde(rename = "DateModified", default)]
	pub date_modified: Option<bool>, 
	#[yaserde(rename = "DisplayName", default)]
	pub display_name: Option<bool>, 
	#[yaserde(rename = "DisplayNameLastModified", default)]
	pub display_name_last_modified: Option<bool>, 
	#[yaserde(rename = "PersonalStatus", default)]
	pub personal_status: Option<bool>, 
	#[yaserde(rename = "PersonalStatusLastModified", default)]
	pub personal_status_last_modified: Option<bool>, 
	#[yaserde(rename = "StaticUserTilePublicURL", default)]
	pub static_user_tile_public_url: Option<bool>, 
	#[yaserde(rename = "Photo", default)]
	pub photo: Option<bool>, 
	#[yaserde(rename = "Attachments", default)]
	pub attachments: Option<bool>, 
	#[yaserde(rename = "Flag", default)]
	pub flag: Option<bool>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "DocumentStreams",
)]
pub struct DocumentStreams {
	#[yaserde(rename = "DocumentStream", default)]
	pub document_stream: Vec<DocumentStream>, 
}
#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
	rename = "documentBaseType",
)]
pub struct DocumentBaseType {
	#[yaserde(rename = "ResourceID", default)]
	pub resource_id: Option<String>, 
	#[yaserde(rename = "Name", default)]
	pub name: Option<String>, 
	#[yaserde(rename = "ItemType", default)]
	pub item_type: String, 
	#[yaserde(rename = "DateModified", default)]
	pub date_modified: String, 
	#[yaserde(rename = "DocumentStreams", default)]
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

