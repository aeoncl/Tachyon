use yaserde_derive::{YaDeserialize, YaSerialize};
use crate::soap::abch::msnab_datatypes::Guid;

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(rename = "WNApplicationHeaderType")]
pub struct WnapplicationHeaderType {
    #[yaserde(rename = "ApplicationId", default)]
    pub application_id: Guid,
    #[yaserde(rename = "RenderingApplicationId", default)]
    pub rendering_application_id: Option<Guid>,
}
pub type WnapplicationHeader = WnapplicationHeaderType;

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(rename = "WNAuthHeaderType")]
pub struct WnauthHeaderType {
    #[yaserde(rename = "TicketToken", default)]
    pub ticket_token: String,
}
pub type WnauthHeader = WnauthHeaderType;

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(rename = "WNServiceHeaderType")]
pub struct WnserviceHeaderType {
    #[yaserde(rename = "Version", default)]
    pub version: String,
    #[yaserde(rename = "CacheKey", default)]
    pub cache_key: Option<String>,
    #[yaserde(rename = "CacheKeyChanged", default)]
    pub cache_key_changed: Option<bool>,
    #[yaserde(rename = "PreferredHostName", default)]
    pub preferred_host_name: Option<String>,
    #[yaserde(rename = "InExperimentalSample", default)]
    pub in_experimental_sample: Option<bool>,
}