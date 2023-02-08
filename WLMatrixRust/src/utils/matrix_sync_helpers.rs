use std::{time::Duration, sync::{Arc}, str::from_utf8};
use tokio::sync::{Mutex, MutexGuard};

use chashmap::{ReadGuard, WriteGuard};
use log::info;
use matrix_sdk::{config::SyncSettings, Client, ruma::{OwnedUserId, events::{room::{member::{MembershipState, RoomMemberEventContent, RoomMemberEvent, SyncRoomMemberEvent, StrippedRoomMemberEvent}, message::{SyncRoomMessageEvent, MessageType, RoomMessageEventContent}}, presence::PresenceEvent, OriginalSyncMessageLikeEvent, SyncEphemeralRoomEvent, EphemeralRoomEvent, typing::{TypingEventContent, SyncTypingEvent}, direct::{DirectEventContent, DirectEvent}, OriginalSyncStateEvent, GlobalAccountDataEventType, AnyGlobalAccountDataEvent, GlobalAccountDataEvent}, api::client::{filter::{FilterDefinition, RoomFilter}, sync::sync_events::v3::{Filter, GlobalAccountData}}, presence::PresenceState}, room::Room};
use tokio::{join, sync::broadcast::{Sender, self}};

use crate::{CLIENT_DATA_REPO, MATRIX_CLIENT_REPO, repositories::{matrix_client_repository::MatrixClientRepository, client_data_repository::ClientDataRepository, repository::Repository}, generated::{msnab_sharingservice::factories::{ContactFactory, MemberFactory, AnnotationFactory}, msnab_datatypes::types::{MemberState, RoleId, ContactTypeEnum, ArrayOfAnnotation}, payloads::{factories::NotificationFactory, PresenceStatus}}, models::{uuid::UUID, switchboard_handle::SwitchboardHandle, msg_payload::factories::MsgPayloadFactory, ab_data::AbData, capabilities::ClientCapabilitiesFactory}, AB_DATA_REPO};

use super::{identifiers::matrix_id_to_msn_addr, matrix::get_direct_target_that_isnt_me, emoji::emoji_to_smiley};

pub async fn start_matrix_loop(token: String, msn_addr: String, sender: Sender<String>) -> Sender<String> {
    
    let matrix_client_repo : Arc<MatrixClientRepository> = MATRIX_CLIENT_REPO.clone();
    let matrix_client = matrix_client_repo.find(&token).unwrap().clone();

        matrix_client.add_event_handler({
            let token = token.clone();
            let msn_addr = msn_addr.clone();
            let sender = sender.clone();
            move |ev: PresenceEvent, client: Client| {
                let token = token.clone();
                let msn_addr = msn_addr.clone();
                let msn_ns_sender = sender.clone();

                async move {

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

            }

        });

        matrix_client.add_event_handler({
            let token = token.clone();
            let msn_addr = msn_addr.clone();

            move |ev: StrippedRoomMemberEvent, room: Room, client: Client| {

                let token = token.clone();
                let msn_addr = msn_addr.clone();

                async move {

                    let ab_data_repo  = AB_DATA_REPO.clone();
                    let ab_data = ab_data_repo.find_mut(&token).unwrap();

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
            }
        });


        matrix_client.add_event_handler({
            let token = token.clone();
            let msn_addr = msn_addr.clone();

            move |ev: SyncRoomMemberEvent, room: Room, client: Client| {
                let token = token.clone();
                let msn_addr = msn_addr.clone();

                async move {

                    let me = client.user_id().unwrap();

                    if let SyncRoomMemberEvent::Original(ev) = &ev {
                        info!("ABDEBUG: Original Event !!");
                        if room.is_direct() || ev.content.is_direct.unwrap_or(false) {
                            info!("Room is direct !!");
                            handle_directs(ev, &room, &client, &token, &msn_addr).await;
                        } else {
                            info!("Room is not a direct !!");
                        }
                    } else if let SyncRoomMemberEvent::Redacted(ev) = &ev {
                        info!("ABDEBUG: Redacted event !!");
                    }
                }
                  
            }
        });


        matrix_client.add_event_handler({

            let token = token.clone();
            let msn_addr = msn_addr.clone();

            move |ev: DirectEvent, client: Client| {

                async move {

                    info!("RECEIVED DIRECT EVENT!!!!");

                }
            }
        });


        matrix_client.add_event_handler({

            let token = token.clone();
            let msn_addr = msn_addr.clone();

            move |ev: SyncTypingEvent, room: Room, client: Client| {
                let token = token.clone();
                let me_msn_addr = msn_addr.clone();

                async move {
                    let room_id = room.room_id().to_string();
                    if let Some(client_data) = CLIENT_DATA_REPO.find_mut(&token){
                        if let Some(found) = client_data.switchboards.find(&room_id) {
                            let found = found.lock().await;
                            for user_id in ev.content.user_ids {
                                let typer_msn_addr = matrix_id_to_msn_addr(&user_id.to_string());
                                if(typer_msn_addr != me_msn_addr) {
                                    found.send_typing_notification_to_client(&typer_msn_addr);
                                }
                            }
                        }
                    }
                }
            }

        });

        matrix_client.add_event_handler({
            let token = token.clone();
            let msn_addr = msn_addr.clone();
            let sender = sender.clone();

            move |ev: SyncRoomMessageEvent, room: Room, client: Client| {
                let token = token.clone();
                let msn_addr = msn_addr.clone();
                let msn_ns_sender = sender.clone();
                async move {

                    let me = client.user_id().unwrap();
                    if let SyncRoomMessageEvent::Original(ev) = ev {
                    
                        let joined_members = room.joined_members().await.unwrap_or(Vec::new());

                        let debug = room.is_direct();
                        let debug_len = joined_members.len();

                        if room.is_direct() && joined_members.len() > 0 && joined_members.len() <= 2 {
                            let me =  client.user_id().unwrap();
                            if let Some(target) = get_direct_target_that_isnt_me(&room.direct_targets(), &me){

                                let room_id = room.room_id().to_string();
                                let target_msn_addr = matrix_id_to_msn_addr(&target.to_string());

                                if let Some(client_data) = CLIENT_DATA_REPO.find_mut(&token){
                                    if let Some(found) = client_data.switchboards.find(&room_id) {
                                        let found = found.lock().await;
                                        handle_messages(found, &ev);
                                    } else {
                                             //sb not initialized yet
                                            let sb_data = Arc::new(Mutex::new(SwitchboardHandle::new(client.clone(), room.room_id().to_owned(), msn_addr.clone())));
                                            {
                                                let sb_data = sb_data.lock().await;
                                                handle_messages(sb_data, &ev);
                                            }

                                             client_data.switchboards.add(room_id.clone(), sb_data);
                                             //send RNG command
                                             let room_uuid = UUID::from_string(&room_id);
     
                                             let session_id = room_uuid.get_most_significant_bytes_as_hex();
     
                                             let ticket = base64::encode(format!("{target_room_id};{token};{target_matrix_id}", target_room_id = &room_id, token = &token, target_matrix_id = target.to_string()));
     
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
            }
        });
    
        /**
         * 
         * 
         *                             if let MessageType::Text(content) = ev.content.msgtype {
                                
                                
                            }

         * 
         */

    let mut filters = FilterDefinition::default();
    let mut room_filters = RoomFilter::default();
    room_filters.include_leave = true;
    filters.room = room_filters;

    let mut settings = SyncSettings::new().timeout(Duration::from_secs(5)).filter(Filter::FilterDefinition(filters));
    settings = settings.set_presence(PresenceState::Offline);

    let my_uuid = UUID::from_string(&msn_addr);


    let (tx, mut _rx) = broadcast::channel::<String>(10);

    tokio::spawn(async move {
        loop {


    
            {
                let ab_data_repo  = AB_DATA_REPO.clone();
                let ab_data = ab_data_repo.find(&token).unwrap();
                info!("has data ?:  {}", ab_data.has_data().to_string());
    
                if ab_data.has_data() {
                    let payload = NotificationFactory::test(&my_uuid, msn_addr.clone());
                    let payload_serialized = payload.to_string();
                    //send updated AB message
    
                    sender.send(format!("NOT {payload_size}\r\n{payload}\r\n", payload_size = payload_serialized.len(), payload = payload_serialized)).unwrap();
                  // sender.send(format!("RFS\r\n")).unwrap();
                }
            }
    
            tokio::select! {
                sync_result = matrix_client.sync_once(settings.clone()) => {
                    settings = settings.token(sync_result.unwrap().next_batch);
                },
                stop_signal = _rx.recv() => {
                    let msg = stop_signal.unwrap();
                    if msg.as_str() == "STOP" {
                        break;
                    }
                },    
            }

            {
                let client_data_repo : Arc<ClientDataRepository> = CLIENT_DATA_REPO.clone();


                if let Some(client_data) = client_data_repo.find(&token) {

                    if let Some(found) = matrix_client.store().get_presence_event(&matrix_client.user_id().unwrap()).await.unwrap() {
                        let event: PresenceEvent = found.deserialize_as().unwrap();
                        

                        let mut status_msg_to_set: Option<String> = None;
                        if let Some(status_msg) = event.content.status_msg.as_ref() {
                            status_msg_to_set = Some(status_msg.to_owned());
                        }

                        matrix_client.account().set_presence(client_data.presence_status.clone().into(), status_msg_to_set).await;

                    }

                    //settings = settings.set_presence(client_data.presence_status.clone().into());
                };
            }
        
        }
    });
    return tx;
}

pub fn handle_messages(switchboard: MutexGuard<SwitchboardHandle>, msg_event: &OriginalSyncMessageLikeEvent<RoomMessageEventContent>) {

    let sender_msn_addr = matrix_id_to_msn_addr(&msg_event.sender.to_string());

    if let MessageType::Text(content) = &msg_event.content.msgtype {
        let msg = MsgPayloadFactory::get_message(emoji_to_smiley(&content.body));
        switchboard.send_message_to_client(msg, &sender_msn_addr, Some(&msg_event.event_id.to_string()));
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


// pub async fn presence_update_task(response : SyncResponse, sender: Sender<String>, matrix_client: ReadGuard<'_, String, Client>) {
    
// }

// pub async fn contact_update_task(response : SyncResponse, sender: Sender<String>, matrix_client: ReadGuard<'_, String, Client>) {
    
//     let me : OwnedUserId = matrix_client.user_id().await.unwrap();

    
// matrix_client.register_event_handler(handler)
//     for (room_id, _room) in &response.rooms.join {
    

//        let current = matrix_client.get_joined_room(room_id).unwrap();
//        if current.is_direct() { 
//            //Direct room
//            let members = current.joined_members().await.unwrap_or(Vec::new());

//            if members.len() > 0 && members.len() <= 2 {
//             for direct_target in current.direct_targets() {
//                 let direct_target_member = current.get_member(&direct_target).await.unwrap().unwrap();
//                 _room.state.events;
//                 match direct_target_member.membership() {
//                     MembershipState::Join => {
//                       
//                     MembershipState::Ban | MembershipState::Leave => {

//                     },
//                     MembershipState::Leave => {

//                     }
//                     _ => {

//                     }
//                 }
               

//             }

//         }
//        }
//     }

//     // make data availlable for FindContactsPaged & FindMembership
// }