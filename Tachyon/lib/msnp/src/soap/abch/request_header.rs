use yaserde_derive::{YaDeserialize, YaSerialize};


#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(rename = "Header")]
pub struct RequestHeaderContainer{

    #[yaserde(rename="ABApplicationHeader")]
    pub application_header: AbapplicationHeader,

    #[yaserde(rename="ABAuthHeader")]
    pub ab_auth_header: AbauthHeader,


}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "ABApplicationHeader",
namespace = "soap: http://www.msn.com/webservices/AddressBook",
prefix = "soap"
default_namespace="soap"
)]
pub struct AbapplicationHeader {
    #[yaserde(rename = "ApplicationId", prefix="soap")]
    pub application_id: String,
    #[yaserde(rename = "IsMigration", prefix="soap")]
    pub is_migration: bool,
    #[yaserde(rename = "PartnerScenario", prefix="soap")]
    pub partner_scenario: String,
    #[yaserde(rename = "CacheKey", prefix="soap")]
    pub cache_key: Option<String>,
    #[yaserde(rename = "BrandId", prefix="soap")]
    pub brand_id: Option<String>,
}


#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "ABAuthHeader",
namespace = "soap: http://www.msn.com/webservices/AddressBook",
prefix = "soap"
default_namespace="soap"
)]
pub struct AbauthHeader {
    #[yaserde(rename = "ManagedGroupRequest", prefix="soap")]
    pub managed_group_request: bool,
    #[yaserde(rename = "TicketToken", prefix="soap")]
    pub ticket_token: String,
}