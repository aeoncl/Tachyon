use anyhow::anyhow;
use axum::http::StatusCode;
use axum::response::Response;
use log::info;
use matrix_sdk::Client;
use matrix_sdk::ruma::events::room::member::MembershipState;
use matrix_sdk_ui::Timeline;
use matrix_sdk_ui::timeline::{RoomExt, TimelineBuilder};
use msnp::msnp::models::contact::Contact;
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::role_list::RoleList;

use msnp::shared::models::ticket_token::TicketToken;
use msnp::shared::models::uuid::Uuid;
use msnp::soap::abch::msnab_datatypes::{Annotation, ArrayOfAnnotation, BaseMember, MemberState};
use msnp::soap::abch::msnab_faults::SoapFaultResponseEnvelope;
use msnp::soap::abch::sharing_service::find_membership::request::FindMembershipRequestSoapEnvelope;
use msnp::soap::abch::sharing_service::find_membership::response::factory::FindMembershipResponseFactory;
use msnp::soap::traits::xml::ToXml;
use crate::matrix::contacts::contact_service::MembershipDiff;
use crate::notification::client_store::{ClientData, ClientStoreFacade};
use crate::shared::identifiers::MatrixIdCompatible;
use crate::shared::traits::ToUuid;

use crate::web::soap::error::ABError;
use crate::web::soap::shared;

pub async fn find_membership(request : FindMembershipRequestSoapEnvelope, token: TicketToken, client: Client, mut client_data: ClientData) -> Result<Response, ABError> {

    let cache_key = request.header.expect("to be here").application_header.cache_key.unwrap_or(Uuid::new().to_string());


    let deltas_only = request.body.request.deltas_only.unwrap_or(false);
    let own_user = client_data.get_user_clone().unwrap();

    if deltas_only {
        // Fetch from store. TODO
        // Ok(shared::build_soap_response(SoapFaultResponseEnvelope::new_fullsync_required("http://www.msn.com/webservices/AddressBook/FindMembership").to_xml()?, StatusCode::OK))
        let members= get_delta_sync(&mut client_data)?;

        let msg_service = FindMembershipResponseFactory::get_messenger_service(members, false);
        let soap_body = FindMembershipResponseFactory::get_response(&own_user, &cache_key, msg_service);

        Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))

    } else {
        let members = get_fullsync_members(&client).await?;
        let msg_service = FindMembershipResponseFactory::get_messenger_service(members, true);
        let soap_body = FindMembershipResponseFactory::get_response(&own_user, &cache_key, msg_service);
        Ok(shared::build_soap_response(soap_body.to_xml()?, StatusCode::OK))
    }
}

fn get_delta_sync(client_data: &mut ClientData) -> Result<Vec<BaseMember>, ABError> {
    let mut members = Vec::new();
    let contact_service = client_data.get_contact_service();
    let mut memberships = contact_service.inner.pending_members.lock().unwrap();
    let contact_list = client_data.get_contact_list().lock().unwrap();

    for member in memberships.drain(..) {
        match member {
            MembershipDiff::AddMembership { user_id, list_type } => {
                let msn_user = MsnUser::with_email_addr(EmailAddress::from_user_id(&user_id));
                let member = BaseMember::new_passport_member(&msn_user, MemberState::Accepted, list_type, false);
                 members.push(member);
            }
            MembershipDiff::AddInviteMembership { user_id, message } => {
                let msn_user = MsnUser::with_email_addr(EmailAddress::from_user_id(&user_id));
                let inviter_member = BaseMember::new_invite_passsport_member(&msn_user, message,false);
                members.push(inviter_member);
            }
            MembershipDiff::RemoveMembership { user_id, list_type } => {
                let msn_user = MsnUser::with_email_addr(EmailAddress::from_user_id(&user_id));
                let member = BaseMember::new_passport_member(&msn_user, MemberState::Accepted, list_type, true);
                members.push(member);
            }
            MembershipDiff::ClearMemberships { user_id } => {
                let msn_user = MsnUser::with_email_addr(EmailAddress::from_user_id(&user_id));
                match contact_list.get_contact(msn_user.get_email_address()) {
                    None => {
                    }
                    Some(contact) => {
                        contact.get_roles().drain(..).for_each(|role| {
                            let member = BaseMember::new_passport_member(&msn_user, MemberState::Accepted, role, true);
                            members.push(member);
                        });
                    }
                }
            }
        }

    }

    Ok(members)
}

async fn get_fullsync_members(matrix_client: &Client) -> Result<Vec<BaseMember>, ABError> {

    let me = matrix_client.user_id().expect("A user to be logged in when fetching fullsync members");

    // for joined_room in matrix_client.joined_rooms() {
    //     if joined_room.is_direct().await? {
    //         let direct_target = resolve_direct_target(&joined_room.direct_targets(), &joined_room, me, matrix_client).await?;
    //         if let Some(direct_target) = direct_target {

    //             if let Some(member) = joined_room.get_member(&direct_target).await? {
    //                 let target_usr = MsnUser::with_email_addr(EmailAddress::from_user_id(&direct_target));
    //                 let target_uuid = target_usr.uuid;
    //                 let target_msn_addr = target_usr.endpoint_id.email_addr.to_string();

    //                 match member.membership() {
    //                     MembershipState::Invite => {
    //                         let allow_member = BaseMember::new_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleList::Allow, false);
    //                         allow_list.push(allow_member);
    //                     }
    //                     MembershipState::Join => {
    //                         let allow_member = BaseMember::new_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleList::Allow, false);
    //                         let reverse_member = BaseMember::new_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleList::Reverse, false);
    //                         allow_list.push(allow_member);
    //                         reverse_list.push(reverse_member);
    //                     }
    //                     _ => {}
    //                 }
    //             }
    //         } else {
    //             info!("Fullsync Fetch: No direct target found for room: {}", &joined_room.room_id());
    //         }
    //     }
    // }

    // for invited_room in matrix_client.invited_rooms() {
    //     if invited_room.is_direct().await? {
    //         let direct_target = resolve_direct_target(&invited_room.direct_targets(), &invited_room, me, matrix_client).await?;
    //         if let Some(direct_target) = direct_target {
    //             if let Some(member) = invited_room.get_member(&direct_target).await? {
    //                 let target_usr = MsnUser::with_email_addr(EmailAddress::from_user_id(&direct_target));
    //                 let target_uuid = target_usr.uuid;
    //                 let target_msn_addr = target_usr.endpoint_id.email_addr.to_string();

    //                 match member.membership() {
    //                     _ => {}
    //                     MembershipState::Join => {
    //                         let current_reverse_member = BaseMember::new_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleList::Reverse, false);
    //                         let mut current_pending_member = BaseMember::new_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleList::Pending, false);
    //                         current_pending_member.display_name = Some(target_msn_addr.clone());
    //                         //TODO fetch message
    //                         let annotation = Annotation::new_invite("");
    //                         let mut annotations = Vec::new();
    //                         annotations.push(annotation);
    //                         current_pending_member.annotations = Some(ArrayOfAnnotation { annotation: annotations });

    //                         reverse_list.push(current_reverse_member);
    //                         pending_list.push(current_pending_member);
    //                     }
    //                 }

    //             }
    //         }
    //     }
    // }


    Ok(Vec::new())


}
//
//
// for diff in address_book_diff {
//         match diff {
//             AddressBookDiff::SetContact { user_id, pending } => {
//                 let msn_user = MsnUser::from_user_id(&user_id);
//                 if let Some(contact) = contact_list.get_contact(&msn_user.get_email_address()) {
//                     if pending && contact.has_role(RoleList::Pending) {
//                         continue;
//                     }
//                     if !pending && contact.has_role(RoleList::Allow) {
//                         continue;
//                     }
//
//                     if contact.has_role(RoleList::Pending) {
//                         current_contacts.push(ContactType::new(&msn_user, ContactTypeEnum::LivePending, true))
//                     } else if contact.has_role(RoleList::Allow) {
//                         current_contacts.push(ContactType::new(&msn_user, ContactTypeEnum::Live, true))
//                     }
//                 }
//
//                 if pending {
//                     current_contacts.push(ContactType::new(&msn_user, ContactTypeEnum::LivePending, false))
//                 } else {
//                     current_contacts.push(ContactType::new(&msn_user, ContactTypeEnum::Live, false))
//                 }
//             }
//             AddressBookDiff::RemoveContact { user_id } => {
//                 let msn_user = MsnUser::from_user_id(&user_id);
//                 if let Some(contact) = contact_list.get_contact(&msn_user.get_email_address()) {
//                     if contact.has_role(RoleList::Pending) {
//                         current_contacts.push(ContactType::new(&msn_user, ContactTypeEnum::LivePending, true))
//                     }
//
//                     if contact.has_role(RoleList::Allow) {
//                         current_contacts.push(ContactType::new(&msn_user, ContactTypeEnum::Live, true))
//                     }
//                 }
//             }
//             AddressBookDiff::AddMembership { user_id, list_type } => {
//
//                 let msn_user = MsnUser::from_user_id(&user_id);
//
//                 if let Some(contact) = contact_list.get_contact(&msn_user.get_email_address()) {
//                     if contact.has_role(list_type.clone()) {
//                         continue;
//                     }
//                 }
//                 current_members.push(BaseMember::new_passport_member(&msn_user, MemberState::Accepted, list_type, false))
//             },
//             AddressBookDiff::AddInviteMembership { user_id, message } => {
//                 let msn_user = MsnUser::from_user_id(&user_id);
//
//                 let current_pending_member = {
//
//                     current_pending_member
//                 };
//
//                 current_members.push(current_pending_member);
//             }
//             AddressBookDiff::RemoveMembership { user_id, list_type } => {
//
//                 let msn_user = MsnUser::from_user_id(&user_id);
//
//                 if let Some(contact) = contact_list.get_contact(&msn_user.get_email_address()) {
//                     if contact.has_role(list_type.clone()) {
//                         current_members.push(BaseMember::new_passport_member(&msn_user, MemberState::Accepted, list_type, true))
//                     }
//                 }
//
//             }
//             AddressBookDiff::ClearMemberships { user_id } => {
//                 let msn_user = MsnUser::from_user_id(&user_id);
//
//                 for list_type in RoleList::iter() {
//
//                     if let Some(contact) = contact_list.get_contact(&msn_user.get_email_address()) {
//                         if contact.has_role(list_type.clone()) {
//                             current_members.push(BaseMember::new_passport_member(&msn_user, MemberState::Accepted, list_type, true))
//                         }
//                     }
//
//                 }
//
//             }
//         }
//     }
