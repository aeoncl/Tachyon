use std::fmt::{Display, Formatter};
use std::str::FromStr;
use directories::ProjectDirs;
use msnp::shared::models::presence_status::PresenceStatus;
use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize)]
pub struct TachyonConfig {

    //#[serde(rename = "defaultPresence")]
    //default_presence: PresenceStatus,
    #[serde(rename = "simulatePresence")]
    pub simulate_presence: bool,
    #[serde(rename = "disableSsl")]
    pub disable_ssl: bool,
    #[serde(rename = "enableLogging")]
    pub enable_logging: bool,

}

impl Default for TachyonConfig {
    fn default() -> Self {
        Self {
            simulate_presence: false,
            disable_ssl: false,
            enable_logging: false,
        }
    }
}


impl Display for TachyonConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
       write!(f, "{}", serde_json::to_string(self).expect("to json"))
    }
}

impl FromStr for TachyonConfig {
    type Err = serde_json::error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {

        serde_json::from_str(s)
    }
}