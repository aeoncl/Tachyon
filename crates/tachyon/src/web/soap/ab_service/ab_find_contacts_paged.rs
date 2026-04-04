use crate::tachyon::identifiers::MatrixIdCompatible;
use crate::web::soap::error::ABError;
use crate::web::soap::shared;
use anyhow::anyhow;
use axum::http::StatusCode;
use axum::response::Response;
use matrix_sdk::ruma::events::room::member::MembershipState;
use matrix_sdk::{Client, RoomMemberships};
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::shared::models::uuid::Uuid;
use msnp::soap::abch::ab_service::ab_find_contacts_paged::request::AbfindContactsPagedMessageSoapEnvelope;
use msnp::soap::abch::ab_service::ab_find_contacts_paged::response::AbfindContactsPagedResponseMessageSoapEnvelope;
use msnp::soap::abch::msnab_datatypes::{CircleRelationshipRole, ContactType, ContactTypeEnum, RelationshipState};
use msnp::soap::traits::xml::ToXml;
use std::str::FromStr;
use crate::matrix::handlers::contact_handlers::{compute_all_contacts};
use crate::tachyon::tachyon_client::TachyonClient;
use crate::notification::models::soap_holder::AddressBookContact;

pub(super) async fn ab_find_contacts_paged(request : AbfindContactsPagedMessageSoapEnvelope, _token: TicketToken, client: Client, mut tachyon_client: TachyonClient) -> Result<Response, ABError> {
    let body = &request.body.body;

    let ab_id = {
        match body.ab_handle.as_ref(){
            None => {
                "00000000-0000-0000-0000-000000000000".to_string()
            }
            Some(ab_handle) => {
                ab_handle.ab_id.as_str().to_string()
            }
        }
    };

    if &ab_id == "00000000-0000-0000-0000-000000000000" {
        //Handle User Request
        handle_user_contact_list(request, client, &mut tachyon_client).await
    } else {
        //Handle Circle Request
        handle_circle_request(request, &ab_id, client, &mut tachyon_client).await
    }
}

async fn handle_user_contact_list(request : AbfindContactsPagedMessageSoapEnvelope, client: Client, client_data: &mut TachyonClient) -> Result<Response, ABError> {
    let body = request.body.body;
    let cache_key = request.header.expect("to be here").application_header.cache_key.unwrap_or(Uuid::new().to_string());
    let me_user = client_data.own_user();
    let _uuid = &me_user.uuid;
    let _msn_addr = me_user.get_email_address();

    if body.filter_options.deltas_only {

        let contacts = get_delta_contact_list(client_data)?;
        
        let soap_body = AbfindContactsPagedResponseMessageSoapEnvelope::new_individual(&me_user, &cache_key, contacts, vec![], false);

        Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))

        //Ok(shared::build_soap_response(SoapFaultResponseEnvelope::new_fullsync_required("http://www.msn.com/webservices/AddressBook/ABFindContactsPaged").to_xml()?, StatusCode::OK))
    } else {
        // Full contact list demanded.
        let (contacts, circles) = {
            let mut contacts = Vec::new();
            let mut circles = Vec::new();
            for current in compute_all_contacts(client).await.drain(..) {
                match current {
                    AddressBookContact::Contact(contact) => {
                        contacts.push(contact);
                    }
                    AddressBookContact::Circle(circle) => {
                        circles.push(circle);
                    }
                }
            }
            (contacts, circles)
        };

        let soap_body = AbfindContactsPagedResponseMessageSoapEnvelope::new_individual(&me_user, &cache_key, contacts, circles,false );
        Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))
    }

}

fn get_delta_contact_list(client_data: &mut TachyonClient) -> Result<Vec<ContactType>, ABError> {
    let mut current_contacts = Vec::new();

    let mut contact_holder = client_data.soap_holder().contacts.lock().unwrap();
    
    for contact in contact_holder.drain(..) {
        match contact {
            AddressBookContact::Contact(contact) => {
                current_contacts.push(contact);
            }
            AddressBookContact::Circle(_) => {}
        }
    }

    Ok(current_contacts)
}



async fn handle_circle_request(request: AbfindContactsPagedMessageSoapEnvelope, ab_id: &str, client: Client, client_data: &mut TachyonClient) -> Result<Response, ABError> {
    let body = request.body.body;
    let cache_key = request.header.expect("to be here").application_header.cache_key.unwrap_or_default();

    if body.filter_options.deltas_only {

        let contacts = client_data.inner.soap_holder.circle_contacts.remove(ab_id).map(|(_id, contacts)| contacts).unwrap_or(Vec::new());

        let soap_body = AbfindContactsPagedResponseMessageSoapEnvelope::new_circle(ab_id, &cache_key, contacts);
        Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))

    } else {
        //TODO have hashmap to avoid computing all the rooms uuid
        let ab_id_uuid = Uuid::from_str(ab_id).unwrap();
        let me = client.user_id().expect("to be here");

        let rooms = client.rooms();
        let found = rooms.iter().find(|r| {
            let room_uuid = Uuid::from_seed(r.room_id().as_str());
            room_uuid == ab_id_uuid
        });

        if found.is_none() {
            return Err(ABError::InternalServerError(anyhow!("todo")));
        };

        let found = found.unwrap();
        let _me = found.get_member_no_sync(me).await?.unwrap();


        let mut contacts = Vec::new();

        let mut members = found.members_no_sync(RoomMemberships::JOIN.union(RoomMemberships::INVITE)).await?;

        for current in members.drain(..){
            match current.membership() {
                MembershipState::Join => {
                    let msn_user = MsnUser::with_email_addr(EmailAddress::from_user_id(current.user_id()));
                    contacts.push(ContactType::new_circle_member_contact(&msn_user.uuid, msn_user.get_email_address().as_str(), current.display_name().unwrap_or(msn_user.get_email_address().as_str()), ContactTypeEnum::Live, RelationshipState::Accepted , CircleRelationshipRole::Member , false));
                }
                _ => {
                    let msn_user = MsnUser::with_email_addr(EmailAddress::from_user_id(current.user_id()));
                    contacts.push(ContactType::new_circle_member_contact(&msn_user.uuid, msn_user.get_email_address().as_str(), current.display_name().unwrap_or(msn_user.get_email_address().as_str()), ContactTypeEnum::LivePending, RelationshipState::WaitingResponse , CircleRelationshipRole::Member,false));
                }
            }
        }


        let soap_body = AbfindContactsPagedResponseMessageSoapEnvelope::new_circle(ab_id, &cache_key, contacts);
        Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))
    }


}
