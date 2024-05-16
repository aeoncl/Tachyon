use std::fmt::Display;
use yaserde::ser::to_string_with_config;
use yaserde_derive::{YaDeserialize, YaSerialize};
use crate::msnp::notification::command::uun::UunPayload;
use crate::msnp::notification::models::endpoint_data::{ClientType, EndpointData};
use crate::shared::models::endpoint_id::EndpointId;
use crate::shared::models::presence_status::PresenceStatus;

pub struct UbxServer {
    tr_id: u128,
    destination: EndpointId,
    payload: UbxPayload
}

pub enum UbxPayload {
    ExtendedPresence(ExtendedPresence)
}

#[derive(Debug, Clone, Default, YaSerialize, YaDeserialize)]
pub struct ExtendedPresence {
    #[yaserde(rename = "PSM")]
    pub psm: String,
    #[yaserde(rename = "CurrentMedia")]
    pub current_media: String,
    #[yaserde(rename = "EndpointData")]
    pub endpoint_data: EndpointData
}

impl Display for ExtendedPresence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        let yaserde_cfg = yaserde::ser::Config{
            perform_indent: false,
            write_document_declaration: false,
            indent_string: None
        };

        if let Ok(serialized) = to_string_with_config(self, &yaserde_cfg) {
            return write!(f, "{}", serialized);
        } else {
            return Err(std::fmt::Error);
        }

    }
}
