use anyhow::anyhow;
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::any;
use log::{debug, warn};
use matrix_sdk::Client;
use matrix_sdk::room::RoomMember;
use matrix_sdk::ruma::events::room::member::MembershipState;
use matrix_sdk::ruma::{OwnedUserId, UserId};
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::shared::models::uuid::Uuid;
use msnp::soap::abch::ab_service::ab_find_contacts_paged::request::AbfindContactsPagedMessageSoapEnvelope;
use msnp::soap::abch::ab_service::ab_find_contacts_paged::response::AbfindContactsPagedResponseMessageSoapEnvelope;
use msnp::soap::abch::msnab_datatypes::{ContactType, ContactTypeEnum};
use msnp::soap::abch::msnab_faults::SoapFaultResponseEnvelope;
use msnp::soap::traits::xml::ToXml;
use crate::matrix::direct_target_resolver::resolve_direct_target;
use crate::shared::identifiers::MatrixIdCompatible;
use crate::web::soap::error::ABError;
use crate::web::soap::error::ABError::InternalServerError;
use crate::web::soap::shared;

pub async fn ab_find_contacts_paged(request : AbfindContactsPagedMessageSoapEnvelope, token: TicketToken, client: Client) -> Result<Response, ABError> {
    let body = request.body.body;
    let cache_key = request.header.expect("to be here").application_header.cache_key.unwrap_or_default();
    let user_id = client.user_id().ok_or(anyhow!("Matrix client has no user ID."))?;
    let msn_addr = EmailAddress::from_user_id(user_id).to_string();


    if body.filter_options.deltas_only {
        // Fetch from store. TODO
        Ok(shared::build_soap_response(SoapFaultResponseEnvelope::new_fullsync_required("http://www.msn.com/webservices/AddressBook/ABFindContactsPaged").to_xml()?, StatusCode::OK))
    } else {
        // Full contact list demanded.
        let contacts = get_fullsync_contact_list(&client, user_id).await?;
        let soap_body = AbfindContactsPagedResponseMessageSoapEnvelope::new(Uuid::from_seed(&user_id.to_string()), &cache_key, &msn_addr, &msn_addr, contacts, false);
        Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))
    }
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