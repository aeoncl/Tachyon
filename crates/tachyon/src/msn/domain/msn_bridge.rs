use crate::msn::domain::switchboard::switchboard::Switchboard;
use dashmap::DashMap;
use itertools::Itertools;
use msnp::msnp::notification::models::msnp_version::MsnpVersion;
use msnp::msnp::notification::models::msnp_version::MsnpVersion::MSNP18;
use msnp::p2p::p2p_version::P2PVersion;
use msnp::shared::models::uuid::Uuid;
use std::mem;
use std::sync::{Arc, RwLock};
use thiserror::Error;

pub type BridgeId = Uuid;

pub struct MsnpBridgeRepository {
    bridges: DashMap<BridgeId, MsnpBridge>
}

impl MsnpBridgeRepository {
    pub fn add_bridge(&self, bridge: MsnpBridge) -> MsnpBridge {
        let _ = self.bridges.insert(bridge.id().to_owned(), bridge.clone());
        bridge
    }

    pub fn get_bridge(&self, id: &BridgeId) -> Option<MsnpBridge> {
        self.bridges.iter().find(|b| b.id() == id).map(|b| b.clone())
    }

    pub fn remove_bridge(&self, id: &BridgeId) {
        let _ = self.bridges.remove(id);
    }
}

impl Default for MsnpBridgeRepository {
    fn default() -> Self {
        Self {
            bridges: Default::default(),
        }
    }
}

pub struct MsnpBridgeInner {
    id: BridgeId,
    configuration: BridgeConfiguration,
    switchboards: DashMap<String, Switchboard>,
    state: RwLock<MsnpBridgeState>
}

#[derive(Clone)]
pub struct MsnpBridge {
    inner: Arc<MsnpBridgeInner>
}
impl MsnpBridge {
    
    pub fn new(configuration: BridgeConfiguration) -> Self {
        Self {
            inner: Arc::new(MsnpBridgeInner {
                id: Uuid::new(),
                configuration,
                switchboards: Default::default(),
                state: Default::default(),
            }),
        }
    }

    pub fn id(&self) -> &BridgeId {
        &self.inner.id
    }

    pub fn supports_protocol_version(&self, protocol_version: &MsnpVersion) -> bool {
        self.inner.configuration.supported_versions.iter().contains(protocol_version)
    }

    pub fn supported_protocol_versions(&self) -> &[MsnpVersion] {
        self.inner.configuration.supported_versions.as_slice()
    }

    pub fn protocol_version(&self) -> Result<MsnpVersion, MsnpError> {
        let state = self.inner.state.read().map_err(|e| MsnpError::LockError(e.to_string()))?;
        state.protocol_version()
    }

    pub fn end_negotiation(&self) -> Result<(), MsnpError> {
        let mut state = self.inner.state.write().map_err(|e| MsnpError::LockError(e.to_string()))?;
        let new_state = state.end_negotiation()?;
        *state = new_state;
        Ok(())
    }

    pub fn set_protocol_version(&self, protocol_version: &MsnpVersion) -> Result<(), MsnpError> {

        if !self.supports_protocol_version(protocol_version) {
            return Err(MsnpError::UnsupportedProtocolVersion)
        }

        let mut state = self.inner.state.write().map_err(|e| MsnpError::LockError(e.to_string()))?;
        state.set_protocol_version(protocol_version.clone())
    }

    pub fn status(&self) -> Result<MsnpBridgeStatus, MsnpError> {
        let state= self.inner.state.read().map_err(|e| MsnpError::LockError(e.to_string()))?;
        Ok(state.status())
    }
}

#[derive(Error, Debug)]
pub enum MsnpError {
    #[error("Unsupported MSNP Protocol Version")]
    UnsupportedProtocolVersion,
    #[error("Negotiation is not finished")]
    NotNegotiatedYet,
    #[error("Negotiation is already done")]
    AlreadyNegotiated,
    #[error("Mutex Lock Error: {0}")]
    LockError(String)
}

#[derive(Clone)]
pub struct BridgeConfiguration {
    supported_versions: Vec<MsnpVersion>,
    p2p_version: P2PVersion
}

pub enum MsnpBridgeStatus {
    Negotiating, Authenticating, Ready
}

pub enum MsnpBridgeState {
    Negotiating {
        protocol_version: Option<MsnpVersion>
    },
    Authenticating {
        protocol_version: MsnpVersion
    },
    Ready {
        protocol_version: MsnpVersion
    }
}

impl MsnpBridgeState {
    pub fn protocol_version(&self) -> Result<MsnpVersion, MsnpError> {
        match self {
            MsnpBridgeState::Negotiating { protocol_version } => protocol_version.as_ref().map(|v| v.clone()).ok_or(MsnpError::NotNegotiatedYet),
            MsnpBridgeState::Authenticating { protocol_version } => Ok(protocol_version.clone()),
            MsnpBridgeState::Ready { protocol_version } => Ok(protocol_version.clone())
        }
    }

    pub fn set_protocol_version(&mut self, new_value: MsnpVersion) -> Result<(), MsnpError> {
        match self {
            MsnpBridgeState::Negotiating { protocol_version } => {
                let _ = mem::replace(protocol_version, Some(new_value));
                Ok(())
            }
            _ => {
                Err(MsnpError::AlreadyNegotiated)
            }
        }
    }

    pub fn end_negotiation(&self) -> Result<Self, MsnpError> {
        match self {
            MsnpBridgeState::Negotiating { protocol_version } => {
                let Some(protocol_version) = protocol_version else {
                    return Err(MsnpError::NotNegotiatedYet);
                };

                Ok(Self::Authenticating {
                    protocol_version: protocol_version.clone(),
                })
            },
            _ => Err(MsnpError::AlreadyNegotiated)
        }
    }

    fn status(&self) -> MsnpBridgeStatus {
        match self {
            MsnpBridgeState::Negotiating { .. } => MsnpBridgeStatus::Negotiating,
            MsnpBridgeState::Authenticating { .. } => MsnpBridgeStatus::Authenticating,
            MsnpBridgeState::Ready { .. } => MsnpBridgeStatus::Ready
        }
    }
}

impl Default for MsnpBridgeState {
    fn default() -> Self {
        Self::Negotiating {
            protocol_version: None,
        }
    }
}

impl BridgeConfiguration {
    pub fn new_msnp_18() -> Self {
        Self  {
            supported_versions: vec![MSNP18],
            p2p_version: P2PVersion::P2PV2,
        }
    }

    pub fn supported_versions(&self) -> &[MsnpVersion] {
        self.supported_versions.as_slice()
    }

    pub fn p2p_version(&self) -> &P2PVersion {
        &self.p2p_version
    }

}