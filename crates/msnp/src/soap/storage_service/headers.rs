use yaserde_derive::{YaDeserialize, YaSerialize};
use crate::shared::models::ticket_token::TicketToken;
use crate::soap::error::SoapMarshallError;
use crate::soap::storage_service::get_profile::request::GetProfileMessageSoapEnvelope;
use crate::soap::traits::xml::TryFromXml;

#[derive(Debug, Default, YaSerialize, YaDeserialize)]
#[yaserde(
rename = "Envelope",
namespace = "soapenv: http://schemas.xmlsoap.org/soap/envelope/",
namespace = "xsi: http://www.w3.org/2001/XMLSchema-instance",
namespace = "xsd: http://www.w3.org/2001/XMLSchema",
prefix = "soapenv"
)]
pub struct StorageServiceRequestSoapEnvelope {
    #[yaserde(rename = "Header", prefix = "soapenv")]
    pub header: StorageServiceHeaders
}

impl TryFromXml for StorageServiceRequestSoapEnvelope {
    type Error = SoapMarshallError;

    fn try_from_xml(xml_str: &str) -> Result<Self, Self::Error> {
        yaserde::de::from_str::<Self>(&xml_str).map_err(|e| Self::Error::DeserializationError { message: e})
    }
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
pub struct StorageServiceHeaders {

    #[yaserde(rename = "StorageApplicationHeader", default)]
    pub storage_application: Option<StorageApplicationHeader>,
    #[yaserde(rename = "StorageUserHeader", default)]
    pub storage_user: Option<StorageUserHeader>,
    #[yaserde(rename = "AffinityCacheHeader", default)]
    pub affinity_cache: Option<AffinityCacheHeader>

}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "StorageApplicationHeader",
namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
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
prefix = "nsi1",
default_namespace="nsi1"
)]
pub struct AffinityCacheHeader {
    #[yaserde(rename = "CacheKey", prefix="nsi1")]
    pub cache_key: Option<String>,
}