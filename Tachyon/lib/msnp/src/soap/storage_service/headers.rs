use yaserde_derive::{YaDeserialize, YaSerialize};
use crate::shared::models::ticket_token::TicketToken;

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
prefix = "nsi1",
)]
pub struct StorageUserHeader {
    #[yaserde(rename = "Puid", default)]
    pub puid: i32,
    #[yaserde(rename = "Cid", default)]
    pub cid: Option<i32>,
    #[yaserde(rename = "TicketToken", default)]
    pub ticket_token: String,
    #[yaserde(rename = "IsAdmin", default)]
    pub is_admin: Option<bool>,
}


#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "AffinityCacheHeader",
namespace = "nsi1: http://www.msn.com/webservices/storage/2008",
prefix = "nsi1",
)]
pub struct AffinityCacheHeader {
    #[yaserde(rename = "CacheKey", default)]
    pub cache_key: Option<String>,
}