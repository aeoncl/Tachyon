use crate::matrix::extensions::msn_user_resolver::FindRoomFromEmail;
use crate::tachyon::tachyon_client::TachyonClient;
use crate::web::soap::error::ABError;
use crate::web::soap::shared;
use anyhow::anyhow;
use axum::http::StatusCode;
use axum::response::Response;
use matrix_sdk::Client;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::shared::models::uuid::Uuid;
use msnp::soap::abch::ab_service::ab_contact_delete::request::AbcontactDeleteMessageSoapEnvelope;
use msnp::soap::abch::ab_service::ab_contact_delete::response::AbcontactDeleteResponseMessageSoapEnvelope;
use msnp::soap::abch::msnab_faults::SoapFaultResponseEnvelope;
use msnp::soap::traits::xml::ToXml;
use std::str::FromStr;


//TODO Contact Delete is called after a delay of 10 secs. Check if we find something in contact dll regarding SHIELDS
pub(super) async fn ab_contact_delete(request : AbcontactDeleteMessageSoapEnvelope, _token: TicketToken, client: Client, tachyon_client: TachyonClient, soap_action: &str) -> Result<Response, ABError> {

    if request.body.ab_contact_delete.ab_id.body != "00000000-0000-0000-0000-000000000000" {
        return Err(ABError::InternalServerError(anyhow!("Invalid AB ID")));
    }

    let mut contacts = request.body.ab_contact_delete.contacts.ok_or(anyhow!("Invalid contacts")).map(|c| c.contact)?;

    if contacts.len() != 1 {
        return Err(ABError::InternalServerError(anyhow!("Only one contact can be deleted at a time")));
    }

    let contact = contacts.drain(..).next().ok_or(anyhow!("No contacts to delete"))?;

    let contact_id = contact.contact_id.ok_or(anyhow!("Contact ID missing"))?;
    let contact_uuid = Uuid::from_str(&contact_id).map_err(|_| anyhow!("Invalid contact ID"))?;

    let contact = {
        let contact_list = tachyon_client.get_contact_list().lock().map_err(|e| anyhow!("Could not mutex lock contact list {}", e))?;
        match contact_list.find_contact_by_uuid(&contact_uuid) {
            Some(c) => c.clone(),
            None => {
                return Ok(shared::build_soap_response(SoapFaultResponseEnvelope::new_contact_doesnt_exist(soap_action, &contact_uuid).to_xml()?, StatusCode::OK));
            }
        }
    };

    match client.find_room_from_email(&contact.email_address)? {
        Some(room) => {
            room.leave().await?;

            let cache_key = request.header.unwrap().application_header.cache_key.unwrap();
            let soap_body = AbcontactDeleteResponseMessageSoapEnvelope::get_response(&cache_key);
            Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))

        },
        None => {
            Ok(shared::build_soap_response(SoapFaultResponseEnvelope::new_contact_doesnt_exist(soap_action, &contact_uuid).to_xml()?, StatusCode::OK))
        },
    }

}
