pub mod request {
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::soap::abch::sharing_service::find_membership::request::FindMembershipRequestSoapEnvelope;
    use crate::soap::error::SoapMarshallError;
    use crate::soap::storage_service::headers::StorageServiceHeaders;
    use crate::soap::storage_service::msnstorage_datatypes::{Handle, ProfileAttributes};
    use crate::soap::traits::xml::TryFromXml;

    #[cfg(test)]
    mod tests{
        use yaserde::de::from_str;
        use crate::soap::abch::ab_service::ab_find_contacts_paged::request::AbfindContactsPagedMessageSoapEnvelope;
        use crate::soap::storage_service::get_profile::request::GetProfileMessageSoapEnvelope;

        #[test]
        fn deser() {
            let request_body = r#"<?xml version="1.0" encoding="utf-8"?>
                            <soap:Envelope xmlns:soap="http://schemas.xmlsoap.org/soap/envelope/"
                                xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                                xmlns:xsd="http://www.w3.org/2001/XMLSchema"
                                xmlns:soapenc="http://schemas.xmlsoap.org/soap/encoding/">
                                <soap:Header>
                                    <AffinityCacheHeader xmlns="http://www.msn.com/webservices/storage/2008">
                                        <CacheKey>12r1:8nBBE6vX1J4uPKajtbem5XBIblimCwAhIziAeEAwYD0AMiaztryWvcZthkN9oX_pl2scBKXfKvRvuWKYdHUNuRkgiyV9rzcDpnDIDiM6vdcEB6d82wjjnL4TAFAjc5X8i-C94mNfQvujUk470P7fz9qbWfK6ANcEtygDb-oWsYVfEBrxl6geTUg9tGT7yCIsls7ECcLyqwsROuAbWCrued_VPKiUgSIvqG8gaA</CacheKey>
                                    </AffinityCacheHeader>
                                    <StorageApplicationHeader xmlns="http://www.msn.com/webservices/storage/2008">
                                        <ApplicationID>Messenger Client 9.0</ApplicationID>
                                        <Scenario>Initial</Scenario>
                                    </StorageApplicationHeader>
                                    <StorageUserHeader xmlns="http://www.msn.com/webservices/storage/2008">
                                        <Puid>0</Puid>
                                        <TicketToken>t=t0ken</TicketToken>
                                    </StorageUserHeader>
                                </soap:Header>
                                <soap:Body>
                                    <GetProfile xmlns="http://www.msn.com/webservices/storage/2008">
                                        <profileHandle>
                                            <Alias>
                                                <Name>-2351581371273912302</Name>
                                                <NameSpace>MyCidStuff</NameSpace>
                                            </Alias>
                                            <RelationshipName>MyProfile</RelationshipName>
                                        </profileHandle>
                                        <profileAttributes>
                                            <ResourceID>true</ResourceID>
                                            <DateModified>true</DateModified>
                                            <ExpressionProfileAttributes>
                                                <ResourceID>true</ResourceID>
                                                <DateModified>true</DateModified>
                                                <DisplayName>true</DisplayName>
                                                <DisplayNameLastModified>true
                                                </DisplayNameLastModified>
                                                <PersonalStatus>true</PersonalStatus>
                                                <PersonalStatusLastModified>true</PersonalStatusLastModified>
                                                <StaticUserTilePublicURL>true</StaticUserTilePublicURL>
                                                <Photo>true</Photo>
                                                <Attachments>true</Attachments>
                                                <Flags>true</Flags>
                                            </ExpressionProfileAttributes>
                                        </profileAttributes>
                                    </GetProfile>
                                </soap:Body>
                            </soap:Envelope>"#;

            let r : GetProfileMessageSoapEnvelope = from_str(&request_body).unwrap();

        }


    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetProfileMessage {
        #[yaserde(rename = "GetProfile", default)]
        pub body: GetProfileRequestType
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "GetProfile",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
    prefix = "nsi1",
    default_namespace = "nsi1"
    )]
    pub struct GetProfileRequestType {
        #[yaserde(rename = "profileHandle", prefix="nsi1")]
        pub profile_handle: Handle,
        #[yaserde(rename = "profileAttributes", prefix="nsi1")]
        pub profile_attributes: ProfileAttributes,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct GetProfileMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: StorageServiceHeaders,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetProfileMessage,
    }

    impl TryFromXml for GetProfileMessageSoapEnvelope {
        type Error = SoapMarshallError;

        fn try_from_xml(xml_str: &str) -> Result<Self, Self::Error> {
            yaserde::de::from_str::<Self>(&xml_str).map_err(|e| Self::Error::DeserializationError { message: e})
        }
    }

    impl GetProfileMessageSoapEnvelope {
        pub fn new(body: SoapGetProfileMessage) -> Self {
            GetProfileMessageSoapEnvelope {
                body,
                header: StorageServiceHeaders::default(),
            }
        }
    }



}

pub mod response {
    use chrono::Local;
    use yaserde::ser::to_string;
    use yaserde_derive::{YaDeserialize, YaSerialize};
    use crate::shared::models::uuid::Uuid;
    use crate::soap::abch::request_header::RequestHeaderContainer;
    use crate::soap::abch::sharing_service::find_membership::response::FindMembershipResponseMessageSoapEnvelope;
    use crate::soap::error::SoapMarshallError;
    use crate::soap::storage_service::fault::SoapFault;
    use crate::soap::storage_service::headers::{AffinityCacheHeader, StorageServiceHeaders, StorageUserHeader};
    use crate::soap::storage_service::msnstorage_datatypes::{DocumentBaseType, DocumentStream, DocumentStreams};
    use crate::soap::traits::xml::ToXml;

    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    pub struct SoapGetProfileResponseMessage {
        #[yaserde(rename = "GetProfileResponse", default)]
        pub body: GetProfileResponse,
        #[yaserde(rename = "Fault", default)]
        pub fault: Option<SoapFault>,

    }

    #[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
    #[yaserde(
    rename = "GetProfileResponse",
    namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
    prefix = "nsi1",
    default_namespace="nsi1"
    )]
    pub struct GetProfileResponse {
        #[yaserde(rename = "GetProfileResult", prefix="nsi1")]
        pub get_profile_result: GetProfileResultType,
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
    rename = "Attachments",
    )]
    pub struct Attachments {
        #[yaserde(rename = "Document", default)]
        pub document: Vec<DocumentBaseType>,
    }


    #[derive(Debug, Default, YaSerialize, YaDeserialize)]
    #[yaserde(
    rename = "Envelope",
    namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
    namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
    namespace = "xsd: http://www.w3.org/2001/XMLSchema",
    prefix = "soapenv"
    )]
    pub struct GetProfileResponseMessageSoapEnvelope {
        #[yaserde(rename = "Header", prefix = "soapenv")]
        pub header: StorageServiceHeaders,
        #[yaserde(rename = "Body", prefix = "soapenv")]
        pub body: SoapGetProfileResponseMessage,
    }



    impl ToXml for GetProfileResponseMessageSoapEnvelope {
        type Error = SoapMarshallError;
        fn to_xml(&self) -> Result<String, Self::Error>  {
            to_string(self).map_err(|e| SoapMarshallError::SerializationError { message: e})
        }

    }



    impl GetProfileResponseMessageSoapEnvelope {
        pub fn new(uuid: Uuid, cache_key: String, display_name: String, psm: String, image_name: Option<String>) -> GetProfileResponseMessageSoapEnvelope {


            let now = Local::now();

            let affinity_cache_header = AffinityCacheHeader{ cache_key: Some(cache_key)};

            let headers = StorageServiceHeaders{ storage_application: None, storage_user: None, affinity_cache: Some(affinity_cache_header) };
            let mut document_stream_array : Vec<DocumentStream> = Vec::new();


            let mut static_user_tile_public_url = String::new();
            if let Some(found_image_id) = image_name {
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

            let get_profile_response = GetProfileResponse{ get_profile_result };

            let body = SoapGetProfileResponseMessage{ body: get_profile_response, fault: None };


            return GetProfileResponseMessageSoapEnvelope{ header: headers, body };
        }


    }


}