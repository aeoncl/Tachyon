use std::convert::Infallible;
use std::fmt::{Display, Formatter};
use std::net::Ipv6Addr;
use std::str::FromStr;
use crate::msnp::notification::models::ip_address::IpAddress;
use crate::p2p::v2::slp::raw_slp_payload::SlpHeaders;
use crate::p2p::v2::slp::SlpHeaders;
use crate::shared::models::uuid::Uuid;

pub struct TransportReqInvitePayload {
    headers: SlpHeaders,
    net_id: u32,
    connection_type: TransportConnectionType,
    tcp_connection_type: TransportConnectionType,
    upnp_nat: bool,
    icf: bool,
    ipv4_internal: IpAddress,
    ipv6_global: Option<Ipv6Addr>,
    capabilities_flags: u32,
    nat_traversal_msg_type: NatTraversalMesssageType,
    bridges: Vec<TransportBridge>,
    hashed_nonce: Uuid
}


pub struct TransportSuccessResponseInvitePayload {
    headers: SlpHeaders,
    session_id: u32,
    listening: bool,
    need_connecting_endpoint_info: bool,
    connection_type: TransportConnectionType,
    tcp_connection_type: TransportConnectionType,
    upnp_nat: bool,
    ipv6_global: Option<Ipv6Addr>,
    capabilities_flags: u32,
    ipv4_external: IpAddress,
    nat_traversal_messsage_type: NatTraversalMesssageType,
    hashed_nonce: Uuid
}

pub struct TransportErrorResponseInvitePayload {
    headers: SlpHeaders,
    bridge: TransportBridge,
    nonce: Uuid,
    capabilities_flags: u32,
}


pub enum TransportConnectionType {
    Firewall,
    UnknownYet(String)
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


pub enum TransportBridge {
    SBBridge,
    TCPv1,
    UnknownYet(String),
}

impl FromStr for TransportBridge {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SBBridge" => Ok(TransportBridge::SBBridge),
            "TCPv1" => Ok(TransportBridge::TCPv1),
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
            TransportBridge::TCPv1 => {
                write!(f, "TCPv1")
            }
            TransportBridge::UnknownYet(bridge) => {
                write!(f, "{}", bridge)
            }
        }
    }
}