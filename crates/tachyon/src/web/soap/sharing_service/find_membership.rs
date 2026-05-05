use anyhow::anyhow;
use axum::http::StatusCode;
use axum::response::Response;

use crate::matrix::handlers::membership_handlers::compute_all_memberships;
use crate::tachyon::services::session::user_service::UserService;
use crate::tachyon::tachyon_client::TachyonClient;
use crate::web::soap::error::ABError;
use crate::web::soap::shared;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::shared::models::uuid::Uuid;
use msnp::soap::abch::msnab_datatypes::BaseMember;
use msnp::soap::abch::sharing_service::find_membership::request::FindMembershipRequestSoapEnvelope;
use msnp::soap::abch::sharing_service::find_membership::response::factory::FindMembershipResponseFactory;
use msnp::soap::traits::xml::ToXml;

pub async fn find_membership(
    request: FindMembershipRequestSoapEnvelope,
    _token: TicketToken,
    mut tachyon_client: TachyonClient,
) -> Result<Response, ABError> {
    let cache_key = request
        .header
        .expect("to be here")
        .application_header
        .cache_key
        .unwrap_or(Uuid::new().to_string());

    let deltas_only = request.body.request.deltas_only.unwrap_or(false);
    let own_user = tachyon_client.own_user();

    if deltas_only {
        let members = get_delta_sync(&tachyon_client)?;

        let msg_service = FindMembershipResponseFactory::get_messenger_service(members, false);
        let soap_body =
            FindMembershipResponseFactory::get_response(&own_user, &cache_key, msg_service);

        Ok(shared::build_soap_response(
            soap_body.to_xml()?,
            StatusCode::OK,
        ))
        //Ok(shared::build_soap_response(SoapFaultResponseEnvelope::new_fullsync_required("http://www.msn.com/webservices/AddressBook/FindMembership").to_xml()?, StatusCode::OK))
    } else {
        let members = compute_all_memberships(
            tachyon_client.matrix_client().clone(),
            &tachyon_client.users(),
        )
        .await;
        let msg_service = FindMembershipResponseFactory::get_messenger_service(members, true);
        let soap_body =
            FindMembershipResponseFactory::get_response(&own_user, &cache_key, msg_service);
        Ok(shared::build_soap_response(
            soap_body.to_xml()?,
            StatusCode::OK,
        ))
    }
}


fn get_delta_sync(client_data: &TachyonClient) -> Result<Vec<BaseMember>, ABError> {
    let mut member_holder =
        client_data.soap_holder().memberships.lock().map_err(|e| {
            ABError::InternalServerError(anyhow!("Could not lock member holder mutex"))
        })?;

    let members: Vec<BaseMember> = member_holder.drain(..).collect();

    Ok(members)
}
