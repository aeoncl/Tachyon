use crate::shared::models::capabilities::{Capabilities, ClientCapabilities};
use crate::shared::models::endpoint_id::EndpointId;
use crate::shared::traits::SerializeMsnp;

//Notifies a client that someone has joined the SB
//SB >> JOI aeon@lukewarmail.com Aeon 2789003324:48
//SB >> JOI aeon@lukewarmail.com;{4059a9be-d326-4394-bc29-3d4f7a7c757a} Aeon 2789003324:48
//Like IRO, if MPOP is enabled, Send multiple Join, one without endpoint and then all the endpoints that are present in the SB.

pub struct JoiServer {

    endpoint_id: EndpointId,
    display_name: String,
    capabilities: ClientCapabilities

}

impl SerializeMsnp for JoiServer {
    fn serialize_msnp(&self) -> Vec<u8> {
        format!("JOI {endpoint_id} {display_name} {capabilities}\r\n",endpoint_id =  self.endpoint_id, display_name = self.display_name, capabilities = self.capabilities).into_bytes()
    }
}