use crate::msnp::error::CommandError;
use crate::msnp::raw_command_parser::RawCommand;
use crate::shared::models::capabilities::ClientCapabilities;
use crate::shared::models::endpoint_id::EndpointId;
use crate::shared::traits::{IntoBytes, TryFromRawCommand};

//Notifies a client that someone has joined the SB
//SB >> JOI aeon@lukewarmail.com Aeon 2789003324:48
//SB >> JOI aeon@lukewarmail.com;{4059a9be-d326-4394-bc29-3d4f7a7c757a} Aeon 2789003324:48
//Like IRO, if MPOP is enabled, Send multiple Join, one without endpoint and then all the endpoints that are present in the SB.

pub struct JoiServer {
    pub endpoint_id: EndpointId,
    pub display_name: String,
    pub capabilities: ClientCapabilities
}

impl TryFromRawCommand for JoiServer {
    type Err = CommandError;

    fn try_from_raw(_raw: RawCommand) -> Result<Self, Self::Err> {
        todo!()
    }

}

impl IntoBytes for JoiServer {

    fn into_bytes(self) -> Vec<u8> {
        format!("JOI {endpoint_id} {display_name} {capabilities}\r\n",endpoint_id =  self.endpoint_id, display_name = self.display_name, capabilities = self.capabilities).into_bytes()
    }
}