use yaserde_derive::{YaDeserialize, YaSerialize};


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
    #[yaserde(rename = "RelationshipName", prefix="nsi1")]
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
    pub is_alias_for_default: Option<bool>,
    #[yaserde(rename = "ExpirationDateTime", prefix="nsi1")]
    pub expiration_date_time: Option<String>,
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "PhotoStream",
)]
pub struct PhotoStream {
    #[yaserde(flatten, default)]
    pub document_stream: DocumentStream,
    #[yaserde(prefix = "xsi", rename = "type", attribute)]
    pub xsi_type: String,
    #[yaserde(rename = "SizeX", default)]
    pub size_x: Option<i32>,
    #[yaserde(rename = "SizeY", default)]
    pub size_y: Option<i32>,
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "Relationship",
namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
prefix = "nsi1",
default_namespace="nsi1"
)]
pub struct Relationship {
    #[yaserde(rename = "SourceID", prefix="nsi1")]
    pub source_id: String,
    #[yaserde(rename = "SourceType", prefix="nsi1")]
    pub source_type: String,
    #[yaserde(rename = "TargetID", prefix="nsi1")]
    pub target_id: String,
    #[yaserde(rename = "TargetType", prefix="nsi1")]
    pub target_type: String,
    #[yaserde(rename = "RelationshipName", prefix="nsi1")]
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
    pub item_type: Option<String>,
    #[yaserde(rename = "DateModified", default)]
    pub date_modified: Option<String>,
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
    #[yaserde(prefix = "xsi", rename = "type", attribute)]
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