use crate::msnp::error::PayloadError;
use crate::p2p::v2::slp::raw_slp_payload::{RawSlpPayload, TryFromRawSlpPayload};
use crate::p2p::v2::slp::{SlpHeaders, SlpStatus};
use crate::shared::models::uuid::Uuid;
use anyhow::anyhow;
use linked_hash_map::LinkedHashMap;
use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

pub struct Bridges(pub Vec<TransportBridge>);

impl Bridges {
    pub fn contains(&self, transport: &TransportBridge) -> bool {
        self.0.contains(transport)
    }
}
impl FromStr for Bridges {
    type Err = PayloadError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut out = Vec::new();

        for current in s.trim().split_whitespace() {
            out.push(TransportBridge::from_str(current).expect("Infaillible to not fail"));
        }

        Ok(Bridges(out))
    }
}

impl Display for Bridges {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();

        for current in &self.0 {
            out.push_str(&format!("{} ", current));
        }

        let trimmed = out.trim();

        write!(f, "{}", trimmed)
    }
}

pub struct NetId(Ipv4Addr);

impl TryFrom<u32> for NetId {
    type Error = PayloadError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let octets = value.to_le_bytes();
        if octets.len() != 4 {
            return Err(PayloadError::AnyError(anyhow!("Unexpected length for `NetId` {:?}. Should be 4 but was {}.", octets, octets.len())))
        }

        let ip = Ipv4Addr::from_octets(octets);
        Ok(Self(ip))
    }
}
impl Into<u32> for NetId {

    fn into(self) -> u32 {
        let octets = self.0.octets();
        let out = u32::from_le_bytes(octets);
        out
    }
}

pub struct TransportInviteRequestPayload {
    headers: SlpHeaders,
    net_id: NetId,
    connection_type: TransportConnectionType,
    tcp_connection_type: TransportConnectionType,
    upnp_nat: bool,
    is_connection_firewalled: bool,
    ipv6_global: Option<Ipv6Addr>,
    ipv4_internal_address: Ipv4Addr,
    ipv4_internal_port: u32,

    //Only if we are firewall'd
    ipv4_external_address: Option<Ipv4Addr>,
    ipv4_external_port: Option<u32>,

    capabilities_flags: u32,
    nat_traversal_msg_type: NatTraversalMesssageType,
    bridges: Bridges,
    nonce: TransportNonce,

    listening: bool,
}

impl TryFromRawSlpPayload for TransportInviteRequestPayload {
    type Err = PayloadError;

    fn try_from_raw_slp_payload(mut payload: RawSlpPayload) -> Result<Self, Self::Err>
    where
        Self: Sized
    {
        let net_id = {
            let raw_net_id = payload.body.remove("NetID").ok_or(anyhow!("Missing slp `NetID` body"))?.parse::<u32>()?;
            NetId::try_from(raw_net_id)?
        };

        let connection_type = {
          let raw_connection_type = payload.body.remove("ConnectionType").ok_or(anyhow!("Missing slp `ConnectionType` body"))?;
            TransportConnectionType::from_str(&raw_connection_type).map_err(|err| anyhow!(err))?
        };

        let tcp_connection_type = {
            let raw_connection_type = payload.body.remove("TCP-Conn-Type").ok_or(anyhow!("Missing slp `TCP-Conn-Type` body"))?;
            TransportConnectionType::from_str(&raw_connection_type).map_err(|err| anyhow!(err))?
        };

        let upnp_nat = payload.body.remove("UPnPNat").ok_or(anyhow!("Missing slp `UPnPNat` body"))?.parse::<bool>().map_err(|err| anyhow!("Could not parse `UPnPNat` body to bool: {}", err))?;
        let is_connection_firewalled = payload.body.remove("ICF").ok_or(anyhow!("Missing slp `ICF` body"))?.parse::<bool>().map_err(|err| anyhow!("Could not parse `ICF` body to bool: {}", err))?;
        let ipv4_internal_address = {
          let ipv4_internal_address_raw = payload.body.remove("IPv4Internal-Addrs").ok_or(anyhow!("Missing slp `IPv4Internal-Addrs` body"))?;
          Ipv4Addr::from_str(&ipv4_internal_address_raw).map_err(|err| anyhow!(err))?
        };

        let ipv4_internal_port = payload.body.remove("IPv4Internal-Port").ok_or(anyhow!("Missing slp `IPv4Internal-Port` body"))?.parse::<u32>()?;

        let ipv4_external_address = {
            if let Some(ipv4_external_address_raw) = payload.body.remove("IPv4External-Addrs") {
                Some(Ipv4Addr::from_str(&ipv4_external_address_raw).map_err(|err| anyhow!(err))?)
            } else {
                None
            }
        };

        let ipv4_external_port = {
            if let Some(ipv4_external_port_raw) = payload.body.remove("IPv4External-Port"){
                Some(ipv4_external_port_raw.parse::<u32>()?)
            } else {
                None
            }
        };

        let ipv6_global = {
            if let Some(ipv6_global_raw) = payload.body.remove("IPv6-global") {
                if ipv6_global_raw.is_empty() {
                    None
                } else {
                    Some(Ipv6Addr::from_str(&ipv6_global_raw).map_err(|err| anyhow!(err))?)
                }
            } else {
                None
            }
        };

        let capabilities_flags = payload.body.remove("Capabilities-Flags").ok_or(anyhow!("Missing slp `Capabilities-Flags` body"))?.parse::<u32>()?;

        let nat_traversal_msg_type = {
            let raw_nat_traversal_msg_type = payload.body.remove("Nat-Trav-Msg-Type").ok_or(anyhow!("Missing slp `Nat-Trav-Msg-Type` body"))?;
            NatTraversalMesssageType::from_str(&raw_nat_traversal_msg_type).map_err(|err| anyhow!(err))?
        };

        let bridges = {
            let raw_bridges = payload.body.remove("Bridges").ok_or(anyhow!("Missing slp Bridges` body"))?;
            Bridges::from_str(&raw_bridges)?
        };

        let nonce = TransportNonce::from_slp_body(&payload.body)?.ok_or(anyhow!("Missing slp `Nonce` or `Hashed-Nonce` body"))?;

        let listening = payload.body.remove("").unwrap_or("Listening".to_string()).parse::<bool>().map_err(|err| anyhow!("Could not parse `Listening` body to bool: {}", err))?;


        let headers = SlpHeaders::try_from_headers(payload.headers)?;

        Ok(Self {
            headers,
            net_id,
            connection_type,
            tcp_connection_type,
            upnp_nat,
            is_connection_firewalled,
            ipv4_internal_address,
            ipv4_internal_port,
            ipv4_external_address,
            ipv4_external_port,
            ipv6_global,
            capabilities_flags,
            nat_traversal_msg_type,
            bridges,
            nonce,
            listening,
        })
    }
}


pub struct TransportInviteResponseSuccessPayload {
    headers: SlpHeaders,
    status: SlpStatus,

    connection_type: TransportConnectionType,
    tcp_connection_type: TransportConnectionType,
    upnp_nat: bool,
    is_connection_firewalled: bool,
    ipv6_global: Option<Ipv6Addr>,
    ipv4_internal_address: Ipv4Addr,
    ipv4_internal_port: u32,

    //Only if we are firewall'd
    ipv4_external_address: Option<Ipv4Addr>,
    ipv4_external_port: Option<u32>,

    capabilities_flags: u32,
    nat_traversal_msg_type: NatTraversalMesssageType,
    nonce: TransportNonce,
    listening: bool,
    
    need_connecting_endpoint_info: bool,
    bridge: TransportBridge,
}

pub struct TransportInviteResponseErrorPayload {
    headers: SlpHeaders,
    bridge: TransportBridge,
    nonce: TransportNonce,
    capabilities_flags: u32,
}

pub struct TransportDestinationAddressUpdatePayload {
    headers: SlpHeaders,

}

pub enum TransportNonce {
    Sha1(Uuid),
    PlainText(Uuid)
}

impl TransportNonce {
    pub fn to_slp_property(&self) -> (String, String) {
        match self {
            TransportNonce::Sha1(uuid) => {
                ("Hashed-Nonce".to_owned(), format!("{{{}}}", uuid.to_string()))
            }
            TransportNonce::PlainText(uuid) => {
                ("Nonce".to_owned(), format!("{{{}}}", uuid.to_string()))
            }
        }
    }

    pub fn from_slp_body(headers: &LinkedHashMap<String, String>) -> Result<Option<Self>, PayloadError> {
        if let Some(nonce) = headers.get("Nonce") {
            let trimmed = nonce.trim().strip_prefix('{')
                .ok_or(anyhow!("Could not strip `{{` from Nonce header: {}", nonce))?
                .strip_suffix('}')
                .ok_or(anyhow!("Could not strip `}}` from Nonce header: {}", nonce))?;

            return Ok(Some(Self::PlainText(Uuid::from_str(trimmed).map_err(|e| anyhow!(e))?)));
        };

        if let Some(nonce) = headers.get("Hashed-Nonce") {
            let trimmed = nonce.trim().strip_prefix('{')
                .ok_or(anyhow!("Could not strip `{{` from Hashed-Nonce header: {}", nonce))?
                .strip_suffix('}')
                .ok_or(anyhow!("Could not strip `}}` from Hashed-Nonce header: {}", nonce))?;
            return Ok(Some(Self::Sha1(Uuid::from_str(trimmed).map_err(|e| anyhow!(e))?)));
        }

        Ok(None)
    }
}


pub enum TransportConnectionType {
    Firewall,
    DirectConnect,
    PortRestrictNat,
    IpRestrictNat,
    SymmetricNat,
    UnknownConnect,
    UnknownYet(String)
}

impl FromStr for TransportConnectionType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let transport = match s {
            "Firewall" => {
                TransportConnectionType::Firewall
            }
            "Direct-Connect" => {
                TransportConnectionType::DirectConnect
            }
            "Port-Restrict-NAT" => {
                TransportConnectionType::PortRestrictNat
            }
            "IP-Restrict-NAT" => {
                TransportConnectionType::IpRestrictNat
            }
            "Symmetric-NAT" => {
                TransportConnectionType::SymmetricNat
            }
            "Unknown-Connect" => {
                TransportConnectionType::UnknownConnect
            }
            _ => {
                TransportConnectionType::UnknownYet(s.to_string())
            }
        };

        Ok(transport)
    }
}

impl Display for TransportConnectionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportConnectionType::Firewall => {
                write!(f, "Firewall")
            }
            TransportConnectionType::DirectConnect => {
                write!(f, "Direct-Connect")
            }
            TransportConnectionType::PortRestrictNat => {
                write!(f, "Port-Restrict-NAT")
            }
            TransportConnectionType::IpRestrictNat => {
                write!(f, "IP-Restrict-NAT")
            }
            TransportConnectionType::SymmetricNat => {
                write!(f, "Symmetric-NAT")
            }
            TransportConnectionType::UnknownConnect => {
                write!(f, "Unknown-Connect")
            }
            TransportConnectionType::UnknownYet(transport) => {
                write!(f, "{}", transport)
            }
        }
    }
}

pub enum NatTraversalMesssageType {
    DirectConnectRequest,
    DirectConnectResponse,
    UnknownYet(String),
}


impl FromStr for NatTraversalMesssageType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "WLX-Nat-Trav-Msg-Direct-Connect-Req" => Ok(NatTraversalMesssageType::DirectConnectRequest),
            "WLX-Nat-Trav-Msg-Direct-Connect-Resp" => Ok(NatTraversalMesssageType::DirectConnectResponse),
            _ => Ok(NatTraversalMesssageType::UnknownYet(s.to_string())),
        }
    }
}

impl Display for NatTraversalMesssageType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NatTraversalMesssageType::DirectConnectRequest => {
                write!(f, "WLX-Nat-Trav-Msg-Direct-Connect-Req")
            }
            NatTraversalMesssageType::DirectConnectResponse => {
                write!(f, "WLX-Nat-Trav-Msg-Direct-Connect-Resp")
            }
            NatTraversalMesssageType::UnknownYet(msg) => {
                write!(f, "{}", msg)
            }
        }
    }
}


#[derive(PartialEq, Eq, Debug)]
pub enum TransportBridge {
    SBBridge,
    TRUDPv1,
    TCPv1,
    TURNv1,
    UnknownYet(String),
}

impl FromStr for TransportBridge {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SBBridge" => Ok(TransportBridge::SBBridge),
            "TRUDPv1" => Ok(TransportBridge::TRUDPv1),
            "TCPv1" => Ok(TransportBridge::TCPv1),
            "TURNv1" => Ok(TransportBridge::TURNv1),
            _ => Ok(TransportBridge::UnknownYet(s.to_string())),
        }
    }
}

impl Display for TransportBridge {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportBridge::SBBridge => {
                write!(f, "SBBridge")
            }
            TransportBridge::TRUDPv1 => {
                write!(f, "TRUDPv1")
            }
            TransportBridge::TCPv1 => {
                write!(f, "TCPv1")
            }
            TransportBridge::TURNv1 => {
                write!(f, "TURNv1")
            }
            TransportBridge::UnknownYet(bridge) => {
                write!(f, "{}", bridge)
            }
        }
    }
}