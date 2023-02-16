use std::{time::Duration, sync::{atomic::{AtomicBool, Ordering}, Arc}};
use base64::{Engine, engine::general_purpose};

use log::info;
use matrix_sdk::{config::SyncSettings, Client, ruma::{OwnedUserId, events::{room::{member::{MembershipState, RoomMemberEventContent, RoomMemberEvent, SyncRoomMemberEvent, StrippedRoomMemberEvent}, message::{SyncRoomMessageEvent, MessageType, RoomMessageEventContent}}, presence::PresenceEvent, OriginalSyncMessageLikeEvent, SyncEphemeralRoomEvent, EphemeralRoomEvent, typing::{TypingEventContent, SyncTypingEvent}, direct::{DirectEventContent, DirectEvent}, OriginalSyncStateEvent, GlobalAccountDataEventType, AnyGlobalAccountDataEvent, GlobalAccountDataEvent}, api::client::{filter::{FilterDefinition, RoomFilter}, sync::sync_events::v3::{Filter, GlobalAccountData}}, presence::PresenceState, RoomId}, room::Room, LoopCtrl, event_handler::Ctx};
use tokio::{sync::{broadcast::{Sender, self}, oneshot}};

use crate::{repositories::{repository::Repository, msn_user_repository::MSNUserRepository}, generated::{msnab_sharingservice::factories::{ContactFactory, MemberFactory, AnnotationFactory}, msnab_datatypes::types::{MemberState, RoleId, ContactTypeEnum, ArrayOfAnnotation}, payloads::{factories::NotificationFactory, PresenceStatus}}, models::{uuid::UUID, msg_payload::factories::MsgPayloadFactory, ab_data::AbData, capabilities::ClientCapabilitiesFactory, msn_user::MSNUser, switchboard::switchboard::Switchboard, notification::msn_client::MSNClient}, AB_DATA_REPO, MSN_CLIENT_LOCATOR, MATRIX_CLIENT_LOCATOR};

use super::{identifiers::matrix_id_to_msn_addr, matrix::get_direct_target_that_isnt_me, emoji::emoji_to_smiley};


// Registering custom event handler context:
#[derive(Debug, Clone)] // The context will be cloned for event handler.
struct WLMatrixContext {
    me: MSNUser,
    sender: Sender<String>,
}

pub async fn start_matrix_loop(matrix_client: Client, msn_user: MSNUser, sender: Sender<String>) -> oneshot::Sender<()> {

    matrix_client.add_event_handler_context(WLMatrixContext { me: msn_user.clone(), sender: sender.clone() });
    
    // Registering all events

    matrix_client.add_event_handler({
        |ev: PresenceEvent, client: Client, context: Ctx<WLMatrixContext>| async move {
                handle_presence_event(ev, client, context.me.clone(), context.sender.clone()).await;
            
        }
    });

    matrix_client.add_event_handler({
        |ev: StrippedRoomMemberEvent, room: Room, client: Client, context: Ctx<WLMatrixContext>| async move {
            handle_stripped_room_member_event(ev, client, context.me.clone(), context.sender.clone()).await;
            
        }
    });


    matrix_client.add_event_handler({
        |ev: SyncRoomMemberEvent, room: Room, client: Client, context: Ctx<WLMatrixContext>| async move{
            handle_sync_room_member_event(ev, room, client, context.me.clone(), context.sender.clone()).await;
            
        }
    });


    matrix_client.add_event_handler({
        |ev: DirectEvent, client: Client, context: Ctx<WLMatrixContext>| async move {
            handle_direct_event(ev, client).await;
            
        }
    });


    matrix_client.add_event_handler({
        |ev: SyncTypingEvent, room: Room, client: Client, context: Ctx<WLMatrixContext>| async move {
            handle_sync_typing_event(ev, room, client, context.me.clone()).await;
            
        }
    });

    matrix_client.add_event_handler({
        |ev: SyncRoomMessageEvent, room: Room, client: Client, context: Ctx<WLMatrixContext> | async move {
            handle_sync_room_message_event(ev, room, client, context.sender.clone()).await;
            
        }
    });
    let (stop_sender, mut stop_receiver) = oneshot::channel::<()>();


    let _result = tokio::spawn(async move{


        let mut settings = get_sync_settings();
        let mut retry_count=0;
        let max_retry_count=3;

        let sync_token = matrix_client.store().get_sync_token().await;
        if let Ok(Some(token)) = sync_token {
            settings = settings.token(token);
        }

        loop {
            tokio::select! {
                sync_result = matrix_client.sync_once(settings.clone()) => {
                    if let Ok(sync_result) = sync_result {
                        settings = settings.token(sync_result.next_batch);
                        retry_count = 0;
                    } else {
                        if retry_count < max_retry_count {
                            retry_count += 1;
                        } else {
                            break;
                        }
                    }
                },
                _stop_signal = &mut stop_receiver => {
                    break;
                },    
            }
        }
    });

    return stop_sender;
   
}


fn check_notify_address_book_update(token: String, me: MSNUser, sender: Sender<String>) {
    let ab_data_repo  = AB_DATA_REPO.clone();
    let ab_data = ab_data_repo.find(&token).unwrap();
    info!("has data ?:  {}", ab_data.has_data().to_string());

    if ab_data.has_data() {
        send_ab_notification(me, sender);
    }

}

fn send_ab_notification(me: MSNUser, sender: Sender<String>) {
    let payload = NotificationFactory::test(&me.get_uuid(), me.get_msn_addr());
    let payload_serialized = payload.to_string();
    //send updated AB message

    sender.send(format!("NOT {payload_size}\r\n{payload}\r\n", payload_size = payload_serialized.len(), payload = payload_serialized)).unwrap();
}

fn get_sync_settings() -> SyncSettings {
    let mut filters = FilterDefinition::default();
    let mut room_filters = RoomFilter::default();
    room_filters.include_leave = true;
    filters.room = room_filters;
    return SyncSettings::new().timeout(Duration::from_secs(5)).filter(Filter::FilterDefinition(filters)).set_presence(PresenceState::Offline);
}

async fn handle_messages(matrix_client: Client, room_id: &RoomId, switchboard: &Switchboard, msg_event: &OriginalSyncMessageLikeEvent<RoomMessageEventContent>) {
    info!("Handle message!");

    let user_repo = MSNUserRepository::new(matrix_client);

    let sender = user_repo.get_msnuser(&room_id, &msg_event.sender).await.unwrap();

    if let MessageType::Text(content) = &msg_event.content.msgtype {
        let msg = MsgPayloadFactory::get_message(emoji_to_smiley(&content.body));
        switchboard.on_message_received(msg, sender, Some(msg_event.event_id.to_string()));
    }
}

async fn handle_directs(ev: &OriginalSyncStateEvent<RoomMemberEventContent>, room: &Room, client: &Client, mtx_token: &String, msn_addr: &String) {

    let joined_members = room.joined_members().await.unwrap_or(Vec::new());

    if joined_members.len() >= 0 && joined_members.len() <= 2 {
        //1O1 DM Room
        info!("Room is One on One Direct !!");
        handle_1v1_dm(ev, room, client, mtx_token, msn_addr).await;
    } else {
        info!("Room is Group DM!!");
        //Group DMs

    }
}

async fn handle_1v1_dm(ev: &OriginalSyncStateEvent<RoomMemberEventContent>, room: &Room, client: &Client, mtx_token: &String, msn_addr: &String) {

    let ab_data_repo  = AB_DATA_REPO.clone();
    let ab_data = ab_data_repo.find_mut(&mtx_token).unwrap();

    let me = client.user_id().unwrap();
    if let Some(target) = get_direct_target_that_isnt_me(&room.direct_targets(), &me){

        let target_uuid = UUID::from_string(&target.to_string());
        let target_msn_addr = matrix_id_to_msn_addr(&target.to_string());
        info!("Direct Target: {}", &target_msn_addr);

        match &room {
            Room::Joined(room) => {
                let display_name = &ev.content.displayname.as_ref().unwrap_or(&target_msn_addr);
                if ev.sender == target {

                    if ev.content.membership == MembershipState::Leave || ev.content.membership == MembershipState::Ban {
                        // my friend is in reverse list and gone from the allow list.
                        let current_reverse_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Reverse, true);
                        ab_data.add_to_messenger_service(target.to_string(), current_reverse_member, RoleId::Reverse);
                    } else if ev.content.membership == MembershipState::Join {
                                            // my friend is in reverse list and allow list.

                        let current_contact = ContactFactory::get_contact(&target_uuid, &target_msn_addr, display_name, ContactTypeEnum::Live, false);
                        let current_allow_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Allow, false);
                        let current_reverse_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Reverse, false);
    
                        ab_data.add_to_contact_list(target.to_string(), current_contact);
                        ab_data.add_to_messenger_service(target.to_string(), current_allow_member, RoleId::Allow);
                        ab_data.add_to_messenger_service(target.to_string(), current_reverse_member, RoleId::Reverse);
    
                      //  if ev.sender == me {
                            //I Accepted an invite
                      //      let current_pending_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Pending, true);
                      //      ab_data.add_to_messenger_service(ev.sender.to_string(), current_pending_member, RoleId::Pending);
                       // }
                    }
                
                }
            },
            Room::Left(room) => {
                //TODO left room not working when user is already gone before us.
                if ev.sender == me && ev.content.membership == MembershipState::Leave || ev.content.membership == MembershipState::Ban { 
                  if should_i_really_delete_contact(client, target.clone()).await {
                    let msn_addr = matrix_id_to_msn_addr(&target.to_string());
                    let current_contact = ContactFactory::get_contact(&target_uuid, &msn_addr,  &msn_addr, ContactTypeEnum::Live, true);
                    let current_allow_member = MemberFactory::get_passport_member(&target_uuid, &msn_addr, MemberState::Accepted, RoleId::Allow, true);
                    ab_data.add_to_contact_list(target.to_string(), current_contact);
                    ab_data.add_to_messenger_service(target.to_string(), current_allow_member, RoleId::Allow);
                  }
                }
            },
            _ => {

            }
        }
    }

    let target = ev.state_key.clone();
    let target_uuid = UUID::from_string(&target.to_string());
    let target_msn_addr = matrix_id_to_msn_addr(&target.to_string());
    let display_name = &ev.content.displayname.as_ref().unwrap_or(&target_msn_addr);

    if ev.sender == me && ev.content.membership == MembershipState::Invite {
        // C'est mon poto pending seulement dans l'allowlist
         let current_contact = ContactFactory::get_contact(&target_uuid, &target_msn_addr, display_name, ContactTypeEnum::LivePending, false);
         let current_allow_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Allow, false);
     
         ab_data.add_to_contact_list(target.to_string(), current_contact);
         ab_data.add_to_messenger_service(target.to_string(), current_allow_member, RoleId::Allow);
     }

}

async fn handle_group_dm(ev: &SyncRoomMemberEvent, room: &Room, client: &Client, mtx_token: &String, msn_addr: &String) {
    
}

async fn should_i_really_delete_contact(client: &Client, contact: OwnedUserId) -> bool {
   let directs = client.store().get_account_data_event(GlobalAccountDataEventType::Direct).await.unwrap().unwrap(); //fix this

   let directs_parsed : GlobalAccountDataEvent<DirectEventContent> = directs.deserialize_as().unwrap();

        let content = directs_parsed.content.0;

        for current in content {
            if current.0 == contact {
                let dm_rooms = current.1;
                for dm_room in dm_rooms {
                    //For each dm room for a contact
                   //if let Some(member_event) = client.store().get_member_event(&dm_room, &contact).await.unwrap() {
                         if let Some(joined_room) = client.get_joined_room(&dm_room) {
                             //If we are still in the room
                           // if member_event.content.membership == MembershipState::Invite || member_event.content.membership == MembershipState::Join {
                                //If the contact is still in the room, don't delete it from the contact list.
                                return false;
                                
                            //};
                         }

                  // }
                }
                break;
            }
        }
    return true;
}

async fn handle_presence_event(ev: PresenceEvent, client: Client, me: MSNUser, msn_ns_sender: Sender<String>) {
                    
    if ev.sender == client.user_id().unwrap() { 
        return;
    }

    let event_sender = &ev.sender;
    let sender_msn_addr = matrix_id_to_msn_addr(&event_sender.to_string());
    let sender_machine_guid = UUID::from_string(&sender_msn_addr).to_string().to_uppercase();

    let presence_status : PresenceStatus = ev.content.presence.into();
    if let PresenceStatus::FLN = presence_status {
        msn_ns_sender.send(format!("FLN 1:{msn_addr}\r\n", msn_addr = sender_msn_addr));
    } else {


        //let test = ev.sender.to_string();
        //let test_vec= client.store().get_custom_value(test.as_bytes()).await.unwrap().unwrap_or("Michel".as_bytes().to_owned());
        //let test3 = format!("{:?}", &test_vec);   

        let room = client.find_dm_room(event_sender).await.unwrap().unwrap();
        let profile = client.store().get_profile(room.room_id(), event_sender).await.unwrap().unwrap();
        let original = profile.as_original().unwrap();
        
        let display_name = original.content.displayname.as_ref().unwrap_or(&sender_msn_addr);

        //let msn_obj = "<msnobj/>";
        let msn_obj = "";
        let capabilities = ClientCapabilitiesFactory::get_default_capabilities();
        msn_ns_sender.send(format!("NLN {status} 1:{msn_addr} {nickname} {client_capabilities} {msn_obj}\r\n", client_capabilities= &capabilities ,msn_addr= &sender_msn_addr, status = presence_status.to_string(), nickname= &display_name, msn_obj = msn_obj));
        //msn_ns_sender.send(format!("NLN {status} 1:{msn_addr} {nickname} 2788999228:48 {msn_obj}\r\n", msn_addr= &sender_msn_addr, status = presence_status.to_string(), nickname= test3, msn_obj = msn_obj));

        let ubx_payload = format!("<PSM>{status_msg}</PSM><CurrentMedia></CurrentMedia><EndpointData id=\"{{{machine_guid}}}\"><Capabilities>{client_capabilities}</Capabilities></EndpointData>", status_msg = ev.content.status_msg.unwrap_or(String::new()), client_capabilities= &capabilities, machine_guid = &sender_machine_guid);
        //let ubx_payload = format!("<PSM>{status_msg}</PSM><CurrentMedia></CurrentMedia>", status_msg = ev.content.status_msg.unwrap_or(String::new()));
        msn_ns_sender.send(format!("UBX 1:{msn_addr} {ubx_payload_size}\r\n{ubx_payload}", msn_addr = &sender_msn_addr, ubx_payload_size= ubx_payload.len(), ubx_payload=ubx_payload));
    }
}

async fn handle_stripped_room_member_event(ev: StrippedRoomMemberEvent, client: Client, me: MSNUser, msn_ns_sender: Sender<String>) {
    let ab_data_repo  = AB_DATA_REPO.clone();
    let ab_data = ab_data_repo.find_mut(&client.access_token().unwrap()).unwrap();

    if ev.content.membership == MembershipState::Invite && ev.state_key == client.user_id().unwrap() && ev.content.is_direct.unwrap_or(false) { 

        let target_uuid = UUID::from_string(&ev.sender.to_string());
        let target_msn_addr = matrix_id_to_msn_addr(&ev.sender.to_string());

        let mut current_pending_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Pending, false);
        current_pending_member.display_name = None;
        let annotation = AnnotationFactory::get_invite(ev.content.reason.unwrap_or(String::new()));
        let mut annotations = Vec::new();
        annotations.push(annotation);
        current_pending_member.annotations=Some(ArrayOfAnnotation{ annotation: annotations });
        ab_data.add_to_messenger_service(ev.sender.to_string(), current_pending_member, RoleId::Pending);

    } else if ev.content.membership == MembershipState::Leave {

        let target_uuid = UUID::from_string(&ev.sender.to_string());
        let target_msn_addr = matrix_id_to_msn_addr(&ev.sender.to_string());

        let mut current_pending_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Pending, true);
        current_pending_member.display_name = None;
        ab_data.add_to_messenger_service(ev.sender.to_string(), current_pending_member, RoleId::Pending);
    }
}

async fn handle_sync_room_member_event(ev: SyncRoomMemberEvent, room: Room, client: Client, me: MSNUser, msn_ns_sender: Sender<String>) {
  let my_user_id = client.user_id().unwrap();

                    if let SyncRoomMemberEvent::Original(ev) = &ev {
                        info!("ABDEBUG: Original Event !!");

                        if ev.sender == my_user_id.to_owned() {
                            info!("ABDEBUG: I Changed!");
                            if let Some(previous_content) = ev.prev_content() {
                                if ev.content.displayname != previous_content.displayname {
                                    
                                    if let Some(display_name) = &ev.content.displayname {
                                       //TODO update display name
                                }
                            }else {

                            }
                            }
                        }

                        if room.is_direct() || ev.content.is_direct.unwrap_or(false) {
                            info!("Room is direct !!");
                            handle_directs(ev, &room, &client, &client.access_token().unwrap(), &me.get_msn_addr()).await;
                        } else {
                            info!("Room is not a direct !!");
                        }
                    } else if let SyncRoomMemberEvent::Redacted(ev) = &ev {
                        info!("ABDEBUG: Redacted event !!");
                    }
}

async fn handle_direct_event(ev: DirectEvent, client: Client) {
    info!("RECEIVED DIRECT EVENT!!!!");

}

async fn handle_sync_typing_event(ev: SyncTypingEvent, room: Room, client: Client, me: MSNUser) {

    let user_repo = MSNUserRepository::new(client.clone());

    let room_id = room.room_id().to_string();
        if let Some(found) = MSN_CLIENT_LOCATOR.get().unwrap().get_switchboards().find(&room_id) {
            for user_id in ev.content.user_ids {
                
                let typing_user = user_repo.get_msnuser(&room.room_id(), &user_id).await.unwrap();

                if &typing_user.get_msn_addr() != &me.get_msn_addr() {

                    let typing_user_payload = MsgPayloadFactory::get_typing_user(typing_user.get_msn_addr().clone());

                    found.on_message_received(typing_user_payload, typing_user, None);
                }
            }
        }
}

async fn handle_sync_room_message_event(ev: SyncRoomMessageEvent, room: Room, client: Client, msn_ns_sender: Sender<String>) {
    if let SyncRoomMessageEvent::Original(ev) = ev {
    
        let joined_members = room.joined_members().await.unwrap_or(Vec::new());

        let debug = room.is_direct();
        let debug_len = joined_members.len();

        if room.is_direct() && joined_members.len() > 0 && joined_members.len() <= 2 {
            let me_user_id =  client.user_id().unwrap();

            if let Some(target) = get_direct_target_that_isnt_me(&room.direct_targets(), &me_user_id){

                let room_id = room.room_id().to_string();
                let target_msn_addr = matrix_id_to_msn_addr(&target.to_string());

                if let Some(msn_client) = MSN_CLIENT_LOCATOR.get(){
                    if let Some(found) = msn_client.get_switchboards().find(&room_id) {
                        handle_messages(client.clone(), &room.room_id(), &found, &ev).await;
                    } else {
                             //sb not initialized yet
                            let sb_data = Switchboard::new(client.clone(), room.room_id().to_owned(), client.user_id().unwrap().to_owned());
                            {
                                handle_messages(client.clone(), &room.room_id(), &sb_data, &ev).await;
                            }

                            msn_client.get_switchboards().add(room_id.clone(), sb_data);
                             //send RNG command
                             let room_uuid = UUID::from_string(&room_id);

                             let session_id = room_uuid.get_most_significant_bytes_as_hex();

                             let ticket = general_purpose::STANDARD.encode(format!("{target_room_id};{token};{target_matrix_id}", target_room_id = &room_id, token = &client.access_token().unwrap(), target_matrix_id = target.to_string()));

                             let _result = msn_ns_sender.send(format!("RNG {session_id} {sb_ip_addr}:{sb_port} CKI {ticket} {invite_passport} {invite_name} U messenger.msn.com 1\r\n",
                                 sb_ip_addr = "127.0.0.1",
                                 sb_port = 1864,
                                 invite_passport = &target_msn_addr,
                                 invite_name = &target_msn_addr,
                                 session_id = session_id,
                                 ticket = ticket
                             ));
                    }  
                }
            }
        }
    }
}