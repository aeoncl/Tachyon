use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(rename = "Header")]
pub struct ServiceHeaderContainer {

    #[yaserde(rename="ServiceHeader")]
    pub service_header: ServiceHeader

}

impl ServiceHeaderContainer {
    pub fn new(cache_key: &str) -> ServiceHeaderContainer {
        let service_header = ServiceHeader{ version: String::from("15.01.1408.0000"), cache_key: Some(cache_key.to_string()), cache_key_changed: Some(true), preferred_host_name: Some(String::from("localhost")), session_id: Some(String::from("17340b67-dcad-48ea-89fb-5e84fbc54cf8")) };
        return ServiceHeaderContainer{ service_header };
    }
}

#[derive(Debug, Default, YaSerialize, YaDeserialize, Clone)]
#[yaserde(
rename = "ServiceHeader",
namespace = "nsi1: http://www.msn.com/webservices/AddressBook",
prefix = "nsi1",
default_namespace="nsi1"
)]
pub struct ServiceHeader {
    #[yaserde(rename = "Version", prefix="nsi1")]
    pub version: String,
    #[yaserde(rename = "CacheKey", prefix="nsi1")]
    pub cache_key: Option<String>,
    #[yaserde(rename = "CacheKeyChanged", prefix="nsi1")]
    pub cache_key_changed: Option<bool>,
    #[yaserde(rename = "PreferredHostName", prefix="nsi1")]
    pub preferred_host_name: Option<String>,
    #[yaserde(rename = "SessionId", prefix="nsi1")]
    pub session_id: Option<String>,
}

