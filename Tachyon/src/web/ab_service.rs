use std::{str::{from_utf8, FromStr}};
use std::any::Any;

use actix_web::{HttpRequest, HttpResponse, HttpResponseBuilder, post, web};
use http::{header::HeaderName, StatusCode};
use log::{info, warn};
use matrix_sdk::{Client, Error, Room, RoomMemberships};
use matrix_sdk::deserialized_responses::RawAnySyncOrStrippedState;
use matrix_sdk::ruma::events::room::member::{MembershipState, StrippedRoomMemberEvent, SyncRoomMemberEvent};
use matrix_sdk::ruma::events::room::message::RoomMessageEventContent;
use matrix_sdk::ruma::events::{AnySyncStateEvent, StateEventType};
use matrix_sdk::ruma::serde::Raw;
use matrix_sdk::ruma::UserId;
use substring::Substring;
use tokio::join;
use yaserde::{de::from_str, ser::to_string};

use crate::{AB_LOCATOR, generated::{msnab_datatypes::types::{ContactType, ContactTypeEnum}, msnab_sharingservice::{bindings::{AbfindContactsPagedMessageSoapEnvelope, AbfindContactsPagedResponseMessageSoapEnvelope, AbgroupAddMessageSoapEnvelope}, factory::{ABGroupAddResponseFactory, ContactFactory, FindContactsPagedResponseFactory, UpdateDynamicItemResponseFactory}}}, MATRIX_CLIENT_LOCATOR, models::{msn_user::MSNUser, uuid::UUID}, MSN_CLIENT_LOCATOR, repositories::repository::Repository, web::error::WebError};
use crate::generated::msnab_datatypes::types::{ContactInfoType, MessengerMemberInfo};
use crate::generated::msnab_sharingservice::bindings::{AbcontactAddMessageSoapEnvelope, AbcontactDeleteMessageSoapEnvelope, AbcontactUpdateMessageSoapEnvelope, AddMemberMessageSoapEnvelope, DeleteMemberMessageSoapEnvelope};
use crate::generated::msnab_sharingservice::factory::{ABContactAddResponseFactory, ABContactDeleteFactory, ABContactUpdateFactory, AddMemberResponseFactory, DeleteMemberResponseFactory};
use crate::generated::msnab_sharingservice::types::AbauthHeader;
use crate::models::msn_user::PartialMSNUser;
use crate::generated::msn_ab_faults;
use crate::matrix::direct_target_resolver::resolve_direct_target;
use crate::matrix::sync_room_member_event_handler::handle_sync_room_member_event;
use crate::models::notification::msn_client::MSNClient;
use crate::repositories::msn_client_locator::MSNClientLocator;
use super::webserver::DEFAULT_CACHE_KEY;

/* Address Book */
#[post("/abservice/abservice.asmx")]
pub async fn soap_adress_book_service(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    if let Some(soap_action_header) = request
        .headers()
        .get(HeaderName::from_str("SOAPAction").unwrap())
    {
        if let Ok(soap_action) = from_utf8(soap_action_header.as_bytes()) {
            let name = soap_action.split("/").last().unwrap_or(soap_action);
            let soap_action_owned = soap_action.to_owned();
            info!("{}Request: {}", &name, from_utf8(&body)?);

            match soap_action {
                "http://www.msn.com/webservices/AddressBook/ABFindContactsPaged" => {
                    return ab_find_contacts_paged(body, request, soap_action_owned.clone()).await;
                },
                "http://www.msn.com/webservices/AddressBook/ABContactAdd" => {
                    return ab_contact_add(body, request, soap_action_owned.clone()).await;
                },
                "http://www.msn.com/webservices/AddressBook/ABContactDelete" => {
                    return ab_contact_delete(body, request).await;
                },
                "http://www.msn.com/webservices/AddressBook/ABContactUpdate" => {
                    return ab_contact_update(body, request).await;
                }
                "http://www.msn.com/webservices/AddressBook/ABGroupAdd" => {
                    return ab_group_add(body, request).await;
                },
                "http://www.msn.com/webservices/AddressBook/UpdateDynamicItem" => {
                    return update_dynamic_item(body, request).await;
                },
                _ => {}
            }
        } else {
            info!("AbService UnknownRequest: {}", from_utf8(&body)?);

        }
    }

    return Ok(HttpResponseBuilder::new(StatusCode::BAD_REQUEST)
        .append_header(("Content-Type", "application/soap+xml"))
        .finish());
}

pub fn authorize(header: &AbauthHeader) -> Result<Client, WebError> {
    let ticket_token = &header.ticket_token;

    let matrix_token = ticket_token
        .substring(2, ticket_token.len())
        .to_string();

    let matrix_client =  MATRIX_CLIENT_LOCATOR.get().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    if matrix_token != matrix_client.access_token().ok_or(StatusCode::UNAUTHORIZED)? {
        return Err(StatusCode::UNAUTHORIZED)?;
    }

    return Ok(matrix_client);
}

async fn ab_group_add(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    let body = from_utf8(&body)?;

    let request = from_str::<AbgroupAddMessageSoapEnvelope>(body)?;
    let header = request.header.ok_or(StatusCode::BAD_REQUEST)?;
    let _matrix_client = authorize(&header.ab_auth_header)?;



    let new_group_guid = UUID::new(); //TODO change this when we really create the matrix space.
        let response = ABGroupAddResponseFactory::get_favorite_group_added_response(new_group_guid.to_string(), header.application_header.cache_key.unwrap_or_default());
        let response_serialized = to_string(&response)?;
    
    return Ok(HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "application/soap+xml")).body(response_serialized));
}

async fn ab_contact_delete(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    let body = from_utf8(&body)?;
    let request = from_str::<AbcontactDeleteMessageSoapEnvelope>(body)?;
    let header = request.header.ok_or(StatusCode::BAD_REQUEST)?;
    let cache_key = &header.application_header.cache_key.unwrap_or_default();
    let matrix_client = authorize(&header.ab_auth_header)?;

    let body = request.body.body.ab_contact_delete_request;
    if body.ab_id.body.as_str() != "00000000-0000-0000-0000-000000000000" {
        return Err(StatusCode::BAD_REQUEST)?
    }

    // if let Some(contact_array) = body.contacts {
    //     for contact in contact_array.contact {
    //         let contact_id = UUID::parse(contact.contact_id.ok_or(StatusCode::BAD_REQUEST)?.as_str())?;
    //         let msn_client = MSN_CLIENT_LOCATOR.get().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    //         if let Some(contact) = msn_client.get_contact_by_guid(contact_id) {
    //             let contact_mxid = contact.get_matrix_id();
    //             if let Some(room) =  matrix_client.get_dm_room(&contact_mxid){
    //                 room.leave().await;
    //             }
    //         }
    //     }
    // }

    let response = ABContactDeleteFactory::get_response(cache_key.to_string());


    let response_serialized = to_string(&response)?;
    info!("ab_contact_delete_response: {}", response_serialized);
    return Ok(HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "application/soap+xml")).body(response_serialized));
}

async fn ab_contact_update(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    let body = from_utf8(&body)?;
    let request = from_str::<AbcontactUpdateMessageSoapEnvelope>(body)?;
    let header = request.header.ok_or(StatusCode::BAD_REQUEST)?;
    let cache_key = &header.application_header.cache_key.unwrap_or_default();
    let matrix_client = authorize(&header.ab_auth_header)?;

    let response = ABContactUpdateFactory::get_response(cache_key.to_owned());

    let response_serialized = to_string(&response)?;
    info!("ab_contact_update_response: {}", response_serialized);
    return Ok(HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "application/soap+xml")).body(response_serialized));

}

async fn ab_contact_add(body: web::Bytes, request: HttpRequest, soap_action: String) -> Result<HttpResponse, WebError> {
    let body = from_utf8(&body)?;

    let request = from_str::<AbcontactAddMessageSoapEnvelope>(body)?;
    let header = request.header.ok_or(StatusCode::BAD_REQUEST)?;
    let cache_key = &header.application_header.cache_key.unwrap_or_default();

    let matrix_client = authorize(&header.ab_auth_header)?;

    let body = request.body.body.ab_contact_add_request;
    if body.ab_id.body.as_str() != "00000000-0000-0000-0000-000000000000" {
        return Err(StatusCode::BAD_REQUEST)?
    }

    if let Some(contact_array) = body.contacts {
        for contact in contact_array.contact {
            if let Some(contact_info) = contact.contact_info {
                let msn_addr = contact_info.passport_name.ok_or(StatusCode::BAD_REQUEST)?;
                let usr = PartialMSNUser::new(msn_addr);

                if let Some(found) = matrix_client.get_dm_room(&usr.get_matrix_id()) {
                    // We already have a room with this fine gentleman
                    info!("We have found a dm room with this gentleman");
                    if let Some(found_user) = found.get_member(&usr.get_matrix_id()).await? {
                        warn!("The dude was FOUND in the room: {:?}", &found_user);
                        match found_user.membership() {
                            MembershipState::Join | MembershipState::Invite => {
                                info!("The dude is joined or invited:");
                            }
                            _ => {
                                info!("The dude was found but we still need to invite him because of membership");
                                found.invite_user_by_id(&usr.get_matrix_id()).await? ;
                            }
                        }
                    } else {
                        info!("The dude is not in the room anymore, reinvite him");
                        found.invite_user_by_id(&usr.get_matrix_id()).await? ;
                    }

                    let response = msn_ab_faults::factory::get_fault_response(msn_ab_faults::factory::soap_fault::get_contact_already_exists(soap_action, &usr.get_uuid()));
                    let response_serialized = to_string(&response)?;
                    return Ok(HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "application/soap+xml")).body(response_serialized));
                }

                if !accept_invite_if_pending(&matrix_client.invited_rooms(), &usr.get_matrix_id()).await? {
                    let dm = matrix_client.create_dm(&usr.get_matrix_id()).await?;
                    if let Some(invite_msg) = extract_invite_msg(contact_info.messenger_member_info.as_ref()) {
                        dm.send( RoomMessageEventContent::text_plain(invite_msg)).await?;
                    }
                }

                let guid = usr.get_uuid();
                let response = ABContactAddResponseFactory::get_response(&guid, cache_key.to_owned());

                let response_serialized = to_string(&response)?;
                info!("ab_contact_add_response: {}", response_serialized);
                return Ok(HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "application/soap+xml")).body(response_serialized));
            }
        }
    }

    return Err(StatusCode::BAD_REQUEST)?;
}

async fn accept_invite_if_pending(invited_rooms: &[Room], target_user_id: &UserId) -> Result<bool, Error> {
    info!("Accept invite if pending ?: looking for {}", target_user_id);
        for invited_room in invited_rooms {
            let is_direct = invited_room.is_direct().await.unwrap_or(false);
            let direct_targets = invited_room.direct_targets();
            info!("current_room: ?: {} - is_direct: {}, direct_targets: {:?}", invited_room.room_id(), &is_direct, &direct_targets);

            if is_direct && direct_targets.len() == 1usize && direct_targets.iter().any(|t| t == target_user_id) {
                invited_room.join().await?;
                return Ok(true);
            }
        }
    return Ok(false);
}

fn extract_invite_msg(messenger_member_info: Option<&MessengerMemberInfo>) -> Option<String> {
   if let Some(member_info) = messenger_member_info {
        if let Some(annotations) = &member_info.pending_annotations {
           if let Some(found) = annotations.annotation.iter().find(|a| a.name.as_str() == "MSN.IM.InviteMessage") {
               return found.value.clone();
           }
        }
   }
    return None;
}

async fn ab_find_contacts_paged(body: web::Bytes, request: HttpRequest, soap_action: String) -> Result<HttpResponse, WebError> {
    let body = from_utf8(&body)?;
    let request = from_str::<AbfindContactsPagedMessageSoapEnvelope>(body)?;
    let header = request.header.ok_or(StatusCode::BAD_REQUEST)?;

    let matrix_client = authorize(&header.ab_auth_header)?;
    let matrix_token = matrix_client.access_token().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let cache_key = &header.application_header.cache_key.unwrap_or_default();
    let msn_client = MSN_CLIENT_LOCATOR.get().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let me_mtx_id = msn_client.get_user().get_matrix_id();

    let me_display_name = match matrix_client.account().get_display_name().await {
        Err(e) => {
            warn!("Error while fetching the display name of logged user: {}", e);
            msn_client.get_user_msn_addr()
        },
        Ok(None) => {
            msn_client.get_user_msn_addr()
        }
        Ok(Some(maybe_display_name)) => {
            maybe_display_name
        }
    };

    let response : AbfindContactsPagedResponseMessageSoapEnvelope;

    if !request.body.body.ab_find_contacts_paged_request.filter_options.deltas_only {
        //Full contact list required
        info!("FindContactsPaged:: FullSync asked !");
        let contacts = get_fullsync_contact_list(&matrix_client).await?;
        info!("FindContactsPaged:: FullSync result: {:?}", &contacts);
        response = FindContactsPagedResponseFactory::get_response(UUID::from_string(&me_mtx_id.to_string()),cache_key.clone(), msn_client.get_user_msn_addr(), me_display_name, contacts, false);
    } else {
        //Only deltas
        if header.application_header.partner_scenario.as_str() == "Initial" {
            //Fetch contacts from the ADL command
            let contacts_as_msn_usr = msn_client.get_contacts(false).await;
            let contact_list = msn_user_to_contact_type(&contacts_as_msn_usr);
            response = FindContactsPagedResponseFactory::get_response(UUID::from_string(&me_mtx_id.to_string()),cache_key.clone(), msn_client.get_user_msn_addr(), me_display_name, contact_list, false);
            //    let empty_response = FindContactsPagedResponseFactory::get_response(UUID::from_string(&me_mtx_id.to_string()),cache_key.clone(), msn_client.get_user_msn_addr(), me_display_name.clone(), Vec::new());
            //   response = empty_response;
        } else {
            let (contact_list, profile_update) = AB_LOCATOR.get_contacts(&matrix_token).await.unwrap();
            response = FindContactsPagedResponseFactory::get_response(UUID::from_string(&me_mtx_id.to_string()),cache_key.clone(), msn_client.get_user_msn_addr(), me_display_name, contact_list, profile_update);
        }
    }




    let response_serialized = to_string(&response)?;
    info!("find_contacts_paged_response: {}", response_serialized);
       
    return Ok(HttpResponseBuilder::new(StatusCode::OK).append_header(("Content-Type", "application/soap+xml")).body(response_serialized));
}

async fn get_fullsync_contact_list(matrix_client: &Client) -> Result<Vec<ContactType>, Error> {
    let mut out = Vec::new();

    let me = matrix_client.user_id().expect("A user to be logged in when fetching fullsync");

    for joined_room in matrix_client.joined_rooms() {
        if joined_room.is_direct().await? {
            let direct_target = resolve_direct_target(&joined_room.direct_targets(), &joined_room, me, matrix_client).await;
            if let Some(direct_target) = direct_target {

                if let Some(member) = joined_room.get_member(&direct_target).await? {
                    let target_usr = MSNUser::from_matrix_id(direct_target.clone());
                    let target_uuid = target_usr.get_uuid();
                    let target_msn_addr = target_usr.get_msn_addr();

                    match member.membership() {
                        MembershipState::Invite => {
                            let contact = ContactFactory::get_contact(&target_uuid, &target_msn_addr, &target_msn_addr, ContactTypeEnum::LivePending, false);
                            out.push(contact);
                        }
                       _ => {
                           let contact = ContactFactory::get_contact(&target_uuid, &target_msn_addr, &target_msn_addr, ContactTypeEnum::Live, false);
                           out.push(contact);
                       }
                    }
                }
            } else {
                info!("Fullsync Fetch: No direct target found for room: {}", &joined_room.room_id());
            }

        }
    }

    return Ok(out);
}



fn msn_user_to_contact_type(contacts: &Vec<MSNUser>) -> Vec<ContactType> {
    let mut out = Vec::new();
    for contact in contacts {
        let current_contact = ContactFactory::get_contact(&contact.get_uuid(), &contact.get_msn_addr(), &contact.get_display_name(), ContactTypeEnum::Live, false);
        out.push(current_contact);
    }
    return out;
}

async fn update_dynamic_item(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {



    let response = UpdateDynamicItemResponseFactory::get_response(DEFAULT_CACHE_KEY.to_string());
    let response_serialized = to_string(&response)?;
    return Ok(HttpResponseBuilder::new(StatusCode::OK)
    .append_header(("Content-Type", "application/soap+xml"))
    .body(response_serialized));
}

