use std::mem;
use std::str::FromStr;
use anyhow::anyhow;
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::any;
use lazy_static_include::syn::ReturnType::Default;
use log::{debug, warn};
use matrix_sdk::{Client, RoomMemberships};
use matrix_sdk::room::RoomMember;
use matrix_sdk::ruma::events::room::member::MembershipState;
use matrix_sdk::ruma::{OwnedUserId, UserId};
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::shared::models::uuid::Uuid;
use msnp::soap::abch::ab_service::ab_find_contacts_paged::request::AbfindContactsPagedMessageSoapEnvelope;
use msnp::soap::abch::ab_service::ab_find_contacts_paged::response::AbfindContactsPagedResponseMessageSoapEnvelope;
use msnp::soap::abch::msnab_datatypes::{AbHandleType, AddressBookType, CircleRelationshipRole, ContactType, ContactTypeEnum, RelationshipState};
use msnp::soap::abch::msnab_faults::SoapFaultResponseEnvelope;
use msnp::soap::traits::xml::ToXml;
use crate::matrix::directs::resolve_direct_target;
use crate::notification::client_store::{ClientData, Contact};
use crate::shared::identifiers::MatrixIdCompatible;
use crate::shared::traits::ToUuid;
use crate::web::soap::error::ABError;
use crate::web::soap::error::ABError::InternalServerError;
use crate::web::soap::shared;

pub async fn ab_find_contacts_paged(request : AbfindContactsPagedMessageSoapEnvelope, token: TicketToken, client: Client, mut client_data: ClientData) -> Result<Response, ABError> {
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
        return handle_user_contact_list(request, client, &mut client_data).await;
    } else {
        //Handle Circle Request
        return handle_circle_request(request, &ab_id, client, &mut client_data).await;
    }

    Err(anyhow!("Unsupported AB Id"))?
}

async fn handle_circle_request(request: AbfindContactsPagedMessageSoapEnvelope, ab_id: &str, client: Client, client_data: &mut ClientData) -> Result<Response, ABError> {
    let body = request.body.body;
    let cache_key = request.header.expect("to be here").application_header.cache_key.unwrap_or_default();

    if body.filter_options.deltas_only {

        let contacts = client_data.inner.soap_holder.circle_contacts.remove(ab_id).map(|(id, contacts)| contacts).unwrap_or(Vec::new());

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
        let me = found.get_member_no_sync(me).await?.unwrap();


        let mut contacts = Vec::new();

        let mut members = if matches!(me.membership(), MembershipState::Join) {found.members(RoomMemberships::JOIN.union(RoomMemberships::INVITE)).await? } else { found.members_no_sync(RoomMemberships::JOIN.union(RoomMemberships::INVITE)).await?  };

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

async fn handle_user_contact_list(request : AbfindContactsPagedMessageSoapEnvelope, client: Client, client_data: &mut ClientData) -> Result<Response, ABError> {
    let body = request.body.body;
    let cache_key = request.header.expect("to be here").application_header.cache_key.unwrap_or_default();
    let user_id = client.user_id().expect("UserId to be present");
    let me_user = client_data.get_user_clone()?;
    let uuid = &me_user.uuid;
    let msn_addr = me_user.get_email_address();

    if body.filter_options.deltas_only {
        let (contacts, circles) = {

           let mut contacts = Vec::new();
           let mut circles = Vec::new();
           for current in get_delta_contact_list(client_data)?.drain(..) {
                match current {
                    Contact::Contact(contact) => {
                        contacts.push(contact);
                    }
                    Contact::Circle(circle_data) => {
                        circles.push(circle_data);
                    }
                }
            }
            (contacts, circles)
        };

        let soap_body = AbfindContactsPagedResponseMessageSoapEnvelope::new_individual(uuid.clone(), &cache_key, msn_addr.as_str(), msn_addr.as_str(), contacts, circles,false);

        Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))

        //Ok(shared::build_soap_response(SoapFaultResponseEnvelope::new_fullsync_required("http://www.msn.com/webservices/AddressBook/ABFindContactsPaged").to_xml()?, StatusCode::OK))
    } else {
        // Full contact list demanded.
        //TODO Circle fullsync
        let mut contacts = get_fullsync_contact_list(&client, user_id).await?;
        let soap_body = AbfindContactsPagedResponseMessageSoapEnvelope::new_individual(uuid.clone(), &cache_key, msn_addr.as_str(), msn_addr.as_str(), contacts, Vec::new(),false );
        Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))
    }

}

fn get_delta_contact_list(client_data: &mut ClientData) -> Result<Vec<Contact>, ABError> {
    let mut contacts = client_data.get_contact_holder_mut().unwrap();

    Ok(mem::replace(&mut *contacts, Vec::new()))
}

async fn get_fullsync_contact_list(matrix_client: &Client, me: &UserId) -> Result<Vec<ContactType>, ABError> {
    let mut out = Vec::new();

    for joined_room in matrix_client.joined_rooms() {
        if joined_room.is_direct().await? {
            let direct_target = resolve_direct_target(&joined_room.direct_targets(), &joined_room, me, matrix_client).await?;
            match direct_target {
                None => {
                    warn!("SOAP|ABCH|ABFindContactsPaged: Could not resolve direct target for direct joined room: {}", joined_room.room_id());
                    continue;
                }
                Some(direct_target) => {

                    let target_usr = MsnUser::with_email_addr(EmailAddress::from_user_id(&direct_target));
                    let target_uuid = target_usr.uuid;
                    let target_msn_addr = target_usr.endpoint_id.email_addr.to_string();

                    match joined_room.get_member(&direct_target).await? {

                        None => {
                            //If member is not here, still consider him a contact, if we want to click on him and create a dm room with him.
                            let contact = ContactType::new(&target_uuid, &target_msn_addr, &target_msn_addr, ContactTypeEnum::Live, false);
                            out.push(contact);
                            debug!("SOAP|ABCH|ABFindContactsPaged: + Live(None) - {}", &target_msn_addr);
                        }

                        Some(member) => {
                            match member.membership() {
                                //If member is here, handle memberships
                                MembershipState::Invite => {
                                    let contact = ContactType::new(&target_uuid, &target_msn_addr, &target_msn_addr, ContactTypeEnum::LivePending, false);
                                    out.push(contact);
                                    debug!("SOAP|ABCH|ABFindContactsPaged: + LivePending(Invite) - {}", &target_msn_addr);
                                }
                                _ => {
                                    let contact = ContactType::new(&target_uuid, &target_msn_addr, &target_msn_addr, ContactTypeEnum::Live, false);
                                    out.push(contact);
                                    debug!("SOAP|ABCH|ABFindContactsPaged: + Live({}) - {}", member.membership() ,&target_msn_addr);
                                }
                            }
                        }

                    }
                }
            }
        }
    }
    Ok(out)
}