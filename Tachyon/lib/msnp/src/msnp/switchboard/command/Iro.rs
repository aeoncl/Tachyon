use crate::shared::models::capabilities::ClientCapabilities;
use crate::shared::models::endpoint_id::EndpointId;
use crate::shared::traits::SerializeMsnp;

// Initial Roster sent after an ANS command
// SB >> IRO 1 1 2 aeon@lukewarmail.com Aeon 2789003324:48
// SB >> IRO 2 2 2 aeon@lukewarmail.com;{4059a9be-d326-4394-bc29-3d4f7a7c757a} Aeon 2789003324:48
// If MPOP (Multiple Points of Presence) Is Enabled, All participants need to join with an endpoint (more than once)
// tr_id is the same one as the ANS command
pub struct IroServer {
    tr_id: u128,
    index: u32,
    roster_count: u32,
    endpoint_id: EndpointId,
    display_name: String,
    capabilities: ClientCapabilities
}

impl SerializeMsnp for IroServer {
    fn serialize_msnp(&self) -> Vec<u8> {
        format!("IRO {tr_id} {index} {roster_count} {endpoint_id} {display_name} {capabilities}\r\n", tr_id = self.tr_id, index = self.index, roster_count = self.roster_count, endpoint_id =  self.endpoint_id, display_name = self.display_name, capabilities = self.capabilities).into_bytes()
    }
}