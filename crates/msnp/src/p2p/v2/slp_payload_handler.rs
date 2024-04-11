use anyhow::anyhow;


use crate::msnp::error::PayloadError;

use super::{error::P2PError, factories::{P2PPayloadFactory, SlpPayloadFactory}, p2p_payload::P2PPayload, slp_payload::SlpPayload};

pub struct SlpPayloadHandler;

impl SlpPayloadHandler {

    pub fn handle(slp_payload: &SlpPayload) -> Result<SlpPayload, P2PError> {
        let content_type = slp_payload.get_content_type().ok_or(anyhow!("Couldn't get content type from SLP Payload: {:?}", &slp_payload))?;
            match content_type.as_str() {
                "application/x-msnmsgr-transreqbody" => {
              //  let slp_payload_response = SlpPayloadFactory::get_200_ok_direct_connect_bad_port(&slp_payload)?;
                 //let mut slp_payload_response = SlpPayloadFactory::get_500_error_direct_connect(slp_payload, String::from("TCPv1"))?; //todo unwrap_or error slp message
                // if self.test > 0 {
                 let slp_payload_response = SlpPayloadFactory::get_500_error_direct_connect(slp_payload, String::from("TCPv1")).unwrap(); //todo unwrap_or error slp message
                //  }

                // self.test += 1;
                   
                  // let mut p2p_payload_response = P2PPayloadFactory::get_sip_text_message();
                  // p2p_payload_response.set_payload(slp_payload_response.to_string().as_bytes().to_owned());
                   return Ok(slp_payload_response);
                // return Err(Errors::PayloadNotComplete);

                },
                "application/x-msnmsgr-sessionreqbody" => {
                    return Ok(SlpPayloadFactory::get_200_ok_session(slp_payload)?);

                },
                "application/x-msnmsgr-transrespbody" => {
                    let bridge = slp_payload.get_body_property(&String::from("Bridge")).unwrap();
                    let slp_payload_response = SlpPayloadFactory::get_500_error_direct_connect(slp_payload, bridge.to_owned())?;
                    return Ok(slp_payload_response);
                },
                "application/x-msnmsgr-sessionclosebody" => {
                    return Err(P2PError::SessionClosed { payload: format!("{:?}", &slp_payload), sauce: anyhow!("payload was not complete") });
                }
                _ => {
                   return Err(PayloadError::PayloadNotHandled { payload: format!("{:?}", &slp_payload)}.into() );
                }
            }
    }

    pub fn handle_p2p(slp_payload: &SlpPayload) -> Result<P2PPayload, P2PError> {
        let slp_payload_response = SlpPayloadHandler::handle(slp_payload)?;
        let mut p2p_payload_response = P2PPayloadFactory::get_sip_text_message();
        p2p_payload_response.set_payload(slp_payload_response.to_string().as_bytes().to_owned());
        return Ok(p2p_payload_response);
    }

}