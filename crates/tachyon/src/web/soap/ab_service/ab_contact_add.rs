use std::str::FromStr;
use std::time::Duration;
use anyhow::anyhow;
use axum::http::StatusCode;
use axum::response::Response;
use log::debug;
use matrix_sdk::{Client, Room, RoomState};
use matrix_sdk::ruma::events::room::member::MembershipState;
use matrix_sdk::ruma::UserId;
use tokio::task;
use tokio::time::sleep;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::not::factories::NotificationFactory;
use msnp::msnp::notification::command::not::{NotServer, NotificationPayload};
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::msn_user::MsnUser;
use msnp::shared::models::ticket_token::TicketToken;
use msnp::shared::models::uuid::Uuid;
use msnp::soap::abch::ab_service::ab_contact_add::request::AbcontactAddMessageSoapEnvelope;
use msnp::soap::abch::ab_service::ab_contact_add::response::AbcontactAddResponseMessageSoapEnvelope;
use msnp::soap::abch::msnab_datatypes::{ContactType, ContactTypeEnum};
use msnp::soap::abch::msnab_datatypes::RoleId::Email;
use msnp::soap::abch::msnab_faults::SoapFaultResponseEnvelope;
use msnp::soap::traits::xml::ToXml;
use crate::matrix::extensions::direct::DirectRoom;
use crate::matrix::extensions::msn_user_resolver::{FindRoomFromEmail, ToEmailAddress, ToMsnUser};
use crate::notification::models::soap_holder::AddressBookContact;
use crate::shared::identifiers::MatrixIdCompatible;
use crate::shared::traits::ToUuid;
use crate::tachyon::tachyon_client::TachyonClient;
use crate::web::soap::error::ABError;
use crate::web::soap::shared;

pub(super) async fn ab_contact_add(request : AbcontactAddMessageSoapEnvelope, _token: TicketToken, client: Client, tachyon_client: TachyonClient, soap_action: &str) -> Result<Response, ABError> {

    if request.body.ab_contact_add.ab_id.body != "00000000-0000-0000-0000-000000000000" {
        return Err(ABError::InternalServerError(anyhow!("Invalid AB ID")));
    }

    let cache_key = request.header.unwrap().application_header.cache_key.unwrap();


    if let Some(contacts) = request.body.ab_contact_add.contacts.map(|c| c.contact) {
        for contact in contacts {
            if let Some(contact_info) = contact.contact_info {
                 if let Some(Ok(contact_email)) = contact_info.passport_name.map(|p| EmailAddress::from_str(&p)) {

                     //We were sent a room sha1d email
                     if let Ok(Some(room)) = client.find_room_from_email(&contact_email) {
                        if let Ok(invite) = room.invite_details().await {
                            room.join().await?;
                            return Ok(contact_create(&contact_email, &cache_key, soap_action, tachyon_client).await?);
                        }
                     }

                     //We were sent a user mapping
                     let contact_user_id = contact_email.to_owned_user_id();
                     println!("ContactUserId {}", &contact_user_id );
                     match client.get_dm_room(&contact_user_id) {
                         None => {
                             let dm = client.create_dm(&contact_user_id).await?;
                             return Ok(contact_create(&contact_email, &cache_key, soap_action, tachyon_client).await?);
                         }
                         Some(dm) => {
                                if !dm.is_valid_one_to_one_direct() {
                                    let dm = client.create_dm(&contact_user_id).await?;
                                    return Ok(contact_create(&contact_email, &cache_key, soap_action, tachyon_client).await?);
                                } else {
                                    if let Some(member) = dm.get_member(&contact_user_id).await? {
                                        match member.membership()  {
                                            MembershipState::Invite | MembershipState::Join => {
                                                //Contact already inside the room.
                                            }
                                            _ => {
                                                dm.invite_user_by_id(&contact_user_id).await?;
                                            }
                                        }
                                        return Ok(contact_already_exists(&dm, &soap_action)?);
                                    } else if let Ok(invite) = dm.invite_details().await {
                                        dm.join().await?;
                                        return Ok(contact_create(&contact_email, &cache_key, soap_action, tachyon_client).await?);
                                    }
                                }
                         }
                     };
                 }
            }
        }
    }

    Ok(shared::build_soap_response(SoapFaultResponseEnvelope::new_generic("Could not create contact".to_string()).to_xml()?, StatusCode::INTERNAL_SERVER_ERROR))

}


//TODO
//Maybe this will be irrelevent when FindByContacts is implemented
//We could also play with other error types to see if we can make msn delete the user. maybe a bad request or something
//We could also delete the user after it's created in the ADL command
async fn delete_user_contact(contact_email_addr: EmailAddress, tachyon_client: TachyonClient) -> Result<(), anyhow::Error> {

    debug!("Deleting user contact: {}", contact_email_addr);

    let user = MsnUser::with_email_addr(contact_email_addr);

    {
        let mut ab_contacts = tachyon_client.soap_holder().contacts.lock().map_err(|e| anyhow!("Failed to lock contacts: {}", e))?;
        let contact = ContactType::new(&user, ContactTypeEnum::Live, true);
        ab_contacts.push( AddressBookContact::Contact(contact));
    }



    let user = tachyon_client.own_user().unwrap();
    tachyon_client.notification_handle().send(NotificationServerCommand::NOT(NotServer {
        payload: NotificationFactory::get_abch_updated(&user.uuid, user.get_email_address()),
    })).await?;

    Ok(())
}

//We send this one because we use the room_ids as email addresses for contacts.
async fn contact_create(contact_email_addr: &EmailAddress, soap_action: &str,  cache_key: &str, tachyon_client: TachyonClient) -> Result<Response, anyhow::Error> {
    let contact_uuid = contact_email_addr.to_uuid();

    let contact_email_addr_clone = contact_email_addr.clone();

    task::spawn(async move {
        let _ = delete_user_contact(contact_email_addr_clone, tachyon_client).await;
    });


    Ok(shared::build_soap_response(AbcontactAddResponseMessageSoapEnvelope::get_response(&contact_uuid, cache_key).to_xml()?, StatusCode::OK))
    //Ok(shared::build_soap_response(SoapFaultResponseEnvelope::new_invalid_passport_user(soap_action, &contact_email_addr.to_string()).to_xml()?, StatusCode::OK))
    //Ok(shared::build_soap_response(SoapFaultResponseEnvelope::new_email_missing_at_sign(soap_action).to_xml()?, StatusCode::OK))

}

fn contact_already_exists(room: &Room, soap_action: &str) -> Result<Response, anyhow::Error> {
    let uuid = room.to_email_address()?.to_uuid();
    Ok(shared::build_soap_response(SoapFaultResponseEnvelope::new_contact_already_exists(soap_action, &uuid).to_xml()?, StatusCode::OK))

}