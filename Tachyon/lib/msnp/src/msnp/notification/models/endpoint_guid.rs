use std::{fmt::Display, str::FromStr};

use crate::{msnp::error::{CommandError}, shared::models::uuid::Uuid};
use anyhow::anyhow;

#[derive(Debug, Clone)]
pub struct EndpointGuid(pub Uuid);

impl FromStr for EndpointGuid {
    type Err = CommandError;

    fn from_str(endpoint_guid: &str) -> Result<Self, Self::Err> {
        let trimmed = endpoint_guid.trim().strip_prefix('{')
            .ok_or(CommandError::ArgumentParseError { argument: endpoint_guid.to_string(), command: String::new(), source: anyhow!("Error stripping {{ prefix from GUID: {}", &endpoint_guid)})?
            .strip_suffix('}')
            .ok_or(CommandError::ArgumentParseError { argument: endpoint_guid.to_string(), command: String::new(), source: anyhow!("Error stripping }} suffix from GUID: {}", &endpoint_guid)})?;

        Uuid::from_str(trimmed).map(|uuid: Uuid| EndpointGuid(uuid)).map_err(|e| CommandError::ArgumentParseError { argument: endpoint_guid.to_string(), command: String::new(), source: e.into() })    }
}

impl Display for EndpointGuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{guid}}}", guid = self.0)
    }
}