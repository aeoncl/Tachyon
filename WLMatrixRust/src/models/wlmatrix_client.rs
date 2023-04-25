use std::{path::Path, collections::HashSet, time::Duration, f32::consts::E};

use base64::{engine::general_purpose, Engine};
use js_int::UInt;
use log::{info, warn};
use matrix_sdk::{Client, Session, config::SyncSettings, ruma::{device_id, api::client::{filter::{FilterDefinition, RoomFilter}, sync::sync_events::v3::{Filter, JoinedRoom}}, presence::PresenceState, events::{presence::PresenceEvent, room::{member::{StrippedRoomMemberEvent, SyncRoomMemberEvent, RoomMemberEventContent, MembershipState}, message::{SyncRoomMessageEvent, RoomMessageEventContent, MessageType, FileMessageEventContent}, MediaSource}, direct::{DirectEvent, DirectEventContent}, typing::SyncTypingEvent, OriginalSyncMessageLikeEvent, OriginalSyncStateEvent, GlobalAccountDataEvent, GlobalAccountDataEventType}, RoomId, OwnedUserId, UserId}, room::{Room, RoomMember}, event_handler::Ctx};
use rand::Rng;
use tokio::sync::{broadcast::Sender, oneshot};

use crate::{utils::{identifiers::{get_matrix_device_id, self}, emoji::emoji_to_smiley}, generated::{payloads::{factories::NotificationFactory, PresenceStatus}, msnab_sharingservice::factories::{MemberFactory, ContactFactory, AnnotationFactory}, msnab_datatypes::types::{ArrayOfAnnotation, RoleId, MemberState, ContactTypeEnum}}, repositories::{msn_user_repository::MSNUserRepository, repository::Repository}, models::{msg_payload::factories::MsgPayloadFactory, uuid::UUID, owned_user_id_traits::ToMsnAddr, abch::events::AddressBookEventFactory}, MSN_CLIENT_LOCATOR, AB_LOCATOR};

use super::{notification::{error::MsnpErrorCode,  events::notification_event::{NotificationEvent, NotificationEventFactory}}, msn_user::MSNUser, switchboard::switchboard::Switchboard, capabilities::ClientCapabilitiesFactory};

#[derive(Debug)]
pub struct WLMatrixClient {
    stop_loop_sender: Option<oneshot::Sender::<()>>
}


#[derive(Debug, Clone)] 
struct WLMatrixContext {
    me: MSNUser,
    event_sender: Sender<NotificationEvent>
}


impl Drop for WLMatrixClient {
    fn drop(&mut self) {
        if let Some(stop_loop_sender) = self.stop_loop_sender.take() {
            let _result = stop_loop_sender.send(());
        }
    }
}

impl WLMatrixClient {

    pub async fn login(matrix_id: OwnedUserId, token: String, store_path: &Path) -> Result<Client, MsnpErrorCode> {
        let device_id = get_matrix_device_id();
        let device_id = device_id!(device_id.as_str()).to_owned();

        match Client::builder()
            .disable_ssl_verification()
            .server_name(matrix_id.server_name())
            .sled_store(store_path, None)
            .build()
            .await
        {
            Ok(client) => {
                if let Err(err) = client
                    .restore_session(Session {
                        access_token: token,
                        refresh_token: None,
                        user_id: matrix_id,
                        device_id: device_id,
                    })
                    .await
                {
                    log::error!("An error has occured logging in via token: {}", err);
                    return Err(MsnpErrorCode::AuthFail);
                }

                if let Err(_check_connection_status) = client.whoami().await {
                    return Err(MsnpErrorCode::AuthFail);
                }
                   return Ok(client);
            }
            Err(_err) => {
                log::error!("An error has occured building the client: {}", _err);
                return Err(MsnpErrorCode::AuthFail);
            }
        }
    }

    pub async fn listen(matrix_client: Client, user: MSNUser, event_sender: Sender<NotificationEvent>) -> Result<Self,()> {
        let kill_sender = Self::start_matrix_loop(matrix_client, user, event_sender).await;
        Ok(WLMatrixClient{ stop_loop_sender: Some(kill_sender) })
    }

pub async fn start_matrix_loop(matrix_client: Client, msn_user: MSNUser, event_sender: Sender<NotificationEvent>) -> oneshot::Sender<()> {

    Self::register_events(&matrix_client, &msn_user, event_sender.clone());
    let (stop_sender, mut stop_receiver) = oneshot::channel::<()>();

    let _result = tokio::spawn(async move{
        let mut settings = Self::get_sync_settings();
        let mut retry_count=0;
        let max_retry_count=3;

        let sync_token = matrix_client.store().get_sync_token().await;

        log::info!("WLMatrix Sync - Preparing Initial Sync");
        if let Ok(Some(token)) = sync_token {
            log::info!("WLMatrix Sync - Token loaded: {}", &token);
            settings = settings.token(token);
        }

        loop {
            tokio::select! {
                sync_result = matrix_client.sync_once(settings.clone()) => {
                    if let Ok(sync_result) = sync_result {
                        log::info!("WLMatrix Sync - next batch: {}", &sync_result.next_batch);
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


fn get_sync_settings() -> SyncSettings {
    let mut filters = FilterDefinition::default();
    let mut room_filters = RoomFilter::default();
    room_filters.include_leave = true;
    filters.room = room_filters;
    return SyncSettings::new().timeout(Duration::from_secs(5)).filter(Filter::FilterDefinition(filters)).set_presence(PresenceState::Online);
}

fn register_events(matrix_client: &Client, msn_user: &MSNUser, event_sender: Sender<NotificationEvent>) {
    matrix_client.add_event_handler_context(WLMatrixContext { me: msn_user.clone(), event_sender });
    
    // Registering all events

    matrix_client.add_event_handler({
        |ev: PresenceEvent, client: Client, context: Ctx<WLMatrixContext>| async move {
                Self::handle_presence_event(ev, client, context.me.clone(), context.event_sender.clone()).await;
            
        }
    });

    matrix_client.add_event_handler({
        |ev: StrippedRoomMemberEvent, room: Room, client: Client, context: Ctx<WLMatrixContext>| async move {
            let notify_ab = Self::handle_stripped_room_member_event(ev, room, client, context.me.clone(), context.event_sender.clone()).await;
            if notify_ab {
                context.event_sender.send(NotificationEventFactory::get_ab_updated(context.me.clone()));
            }
        }
    });


    matrix_client.add_event_handler({
        |ev: SyncRoomMemberEvent, room: Room, client: Client, context: Ctx<WLMatrixContext>| async move{
            Self::handle_sync_room_member_event(ev, room, client, context.me.clone(), context.event_sender.clone()).await;
            
        }
    });


    matrix_client.add_event_handler({
        |ev: DirectEvent, client: Client, context: Ctx<WLMatrixContext>| async move {
            Self::handle_direct_event(ev, client).await;
            
        }
    });


    matrix_client.add_event_handler({
        |ev: SyncTypingEvent, room: Room, client: Client, context: Ctx<WLMatrixContext>| async move {
            Self::handle_sync_typing_event(ev, room, client, context.me.clone()).await;
            
        }
    });

    matrix_client.add_event_handler({
        |ev: SyncRoomMessageEvent, room: Room, client: Client, context: Ctx<WLMatrixContext> | async move {
            Self::handle_sync_room_message_event(ev, room, client, context.event_sender.clone()).await;
            
        }
    });
}

async fn handle_messages(matrix_client: Client, room_id: &RoomId, switchboard: &Switchboard, msg_event: &OriginalSyncMessageLikeEvent<RoomMessageEventContent>) {
    info!("Handle message!");

    let user_repo = MSNUserRepository::new(matrix_client);

    let sender = user_repo.get_msnuser(&room_id, &msg_event.sender, false).await.unwrap();

    if let MessageType::Text(content) = &msg_event.content.msgtype {
        let msg = MsgPayloadFactory::get_message(emoji_to_smiley(&content.body));
        switchboard.on_message_received(msg, sender, Some(msg_event.event_id.to_string()));
    } else if let MessageType::File(content) = &msg_event.content.msgtype {
        log::info!("Received a file: {:?}", &content);
        switchboard.on_file_received(sender, content.body.clone(), content.source.clone(), WLMatrixClient::get_size_or_default(&content),  msg_event.event_id.to_string());
    }
}

fn get_size_or_default(content: &FileMessageEventContent) -> usize {
    let mut size: i32 = 0;
    if let Some(info) = content.info.as_ref() {
        if let Ok(valid_size) = i32::try_from(info.size.unwrap_or(UInt::new(0).unwrap())) {
            size = valid_size;
        }
    }
    return usize::try_from(size).expect("Matrix file size to be a usize");
}

async fn handle_directs(ev: &OriginalSyncStateEvent<RoomMemberEventContent>, room: &Room, client: &Client, mtx_token: &String, msn_addr: &String) -> bool {

    let joined_members = room.joined_members().await.unwrap_or(Vec::new());

    let mut notify_ab = false;
    if joined_members.len() >= 0 && joined_members.len() <= 2 {
        //1O1 DM Room
        info!("Room is One on One Direct !!");
        notify_ab = notify_ab || Self::handle_1v1_dm2(ev, room, client, mtx_token, msn_addr, joined_members).await;
    } else {
        info!("Room is Group DM!!");
        //Group DMs

    }

    return notify_ab;
}

async fn handle_1v1_dm2(ev: &OriginalSyncStateEvent<RoomMemberEventContent>, room: &Room, client: &Client, mtx_token: &String, msn_addr: &String, joined_members: Vec<RoomMember>) -> bool {
    let mut notify_ab = false;
    let ab_sender = AB_LOCATOR.get_sender();
    let matrix_token = client.access_token().unwrap();
    let me = client.user_id().unwrap().to_owned();


    let target = Self::get_direct_target_that_isnt_me(&room.direct_targets(), &room, &me).await.unwrap();
    let target_usr = MSNUser::from_matrix_id(target.clone());
    let target_uuid = target_usr.get_uuid();
    let target_msn_addr = target_usr.get_msn_addr();

    log::info!("AB DEBUG - State_key: {}, sender: {}, membership: {}", &ev.state_key, &ev.sender, &ev.content.membership);

    match &room {
        Room::Joined(room) => {
            if &ev.state_key == &target {
                let display_name = ev.content.displayname.as_ref().unwrap_or(&target_msn_addr).to_owned();
                if ev.content.membership == MembershipState::Invite && &ev.sender == &me {
                    //I Invited him, Add to allow list, add to contact pending.
                    log::info!("AB - Send invitation to: {}", &target_msn_addr);
                    let invited_contact = ContactFactory::get_contact(&target_uuid, &target_msn_addr, &display_name, ContactTypeEnum::LivePending, false);
                    let invited_allow_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Allow, false);
                    ab_sender.send(AddressBookEventFactory::get_contact_event(matrix_token.clone(), invited_contact));
                    ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), invited_allow_member, RoleId::Allow));
                    notify_ab = true;
                } else if ev.content.membership == MembershipState::Join {
                    //He accepted my invitation, ADD TO REVERSE LIST, CHANGE CONTACT TO NOT PENDING
                    log::info!("AB - Invitation Accepted from: {}", &target_msn_addr);
                    let invited_contact = ContactFactory::get_contact(&target_uuid, &target_msn_addr, &display_name, ContactTypeEnum::Live, false);
                    let invited_reverse_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Reverse, false);
                    ab_sender.send(AddressBookEventFactory::get_contact_event(matrix_token.clone(), invited_contact));
                    ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), invited_reverse_member, RoleId::Reverse));
                    notify_ab = true;
                } else if ev.content.membership == MembershipState::Leave || ev.content.membership == MembershipState::Ban {
                    log::info!("AB - Contact left: {}", &target_msn_addr);
                    //He left the room, Remove from Reverse List, Set to contact pending
                    let left_contact = ContactFactory::get_contact(&target_uuid, &target_msn_addr, &display_name, ContactTypeEnum::LivePending, false);
                    let left_reverse_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Reverse, true);
                    ab_sender.send(AddressBookEventFactory::get_contact_event(matrix_token.clone(), left_contact));
                    ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), left_reverse_member, RoleId::Reverse));
                    notify_ab = true;
                }
            } else if &ev.state_key == &me {
                if &ev.content.membership == &MembershipState::Invite && &ev.sender == &target {
                    //This is strange, i should get a join membership when i accept an invite, but i get an invite
                             //I Accepted his invitation, REMOVE FROM PENDING LIST, ADD TO ALLOW LIST, ADD TO CONTACT LIST
                    log::info!("AB - I Accepted an invite from: {}", &target_msn_addr);

                    let inviter_contact = ContactFactory::get_contact(&target_uuid, &target_msn_addr, &target_msn_addr, ContactTypeEnum::Live, false);
                    let inviter_allow_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Allow, false);
                    let inviter_pending_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Pending, true); 
                    ab_sender.send(AddressBookEventFactory::get_contact_event(matrix_token.clone(), inviter_contact));
                    ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), inviter_allow_member, RoleId::Allow));
                    ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), inviter_pending_member, RoleId::Pending));
                    notify_ab = true;

                }
            }
        },
        Room::Left(room) => {

            if Self::should_i_really_delete_contact(&client, target.clone()).await {
                log::info!("AB - I Deleted: {}", &target_msn_addr);
                //I Left the room, remove member from PENDING_LIST, ALLOW_LIST, REVERSE_LIST. Remove Contact from Contact List
                let current_contact = ContactFactory::get_contact(&target_uuid, &target_msn_addr, &target_msn_addr, ContactTypeEnum::Live, true);
                let current_allow_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Allow, true);
                let current_reverse_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Reverse, true);
                let current_pending_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Pending, true);
    
                ab_sender.send(AddressBookEventFactory::get_contact_event(matrix_token.clone(), current_contact));
                ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), current_allow_member, RoleId::Allow));
                ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), current_reverse_member, RoleId::Reverse));
                ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), current_pending_member, RoleId::Pending));
                notify_ab = true;
            }
        },
        _=> {

        }
    }

    return notify_ab;
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


//TODO handle the fact that room member events also broadcast name change & psm change
async fn handle_presence_event(ev: PresenceEvent, client: Client, me: MSNUser, ns_sender: Sender<NotificationEvent>) {
                    
    if ev.sender == client.user_id().unwrap() { 

        //TODO handle me changing avatars
        return;
    }

    let user_repo = MSNUserRepository::new(client.clone());


    let event_sender: &OwnedUserId = &ev.sender;
    let sender_msn_addr = event_sender.to_msn_addr();

    if let Ok(mut user) = user_repo.get_msnuser_from_userid(event_sender, false).await{

    let presence_status : PresenceStatus = ev.content.presence.clone().into();

    info!("Received Presence Event: {:?} - ev: {:?}", &presence_status, &ev);

    if PresenceStatus::FLN == presence_status {
        ns_sender.send(NotificationEventFactory::get_disconnect(user));
    } else {

            user.set_status(presence_status);
            if let Some(display_name) = ev.content.displayname {
                user.set_display_name(display_name);
            }

            if let Some(status_msg) = ev.content.status_msg{
                user.set_psm(status_msg);
            }


            if let Some(avatar_mxc) = ev.content.avatar_url.as_ref() {

                match user_repo.get_avatar(avatar_mxc.clone()).await {
                    Ok(avatar) => {
                       user.set_display_picture(Some(user_repo.avatar_to_msn_obj(&avatar, sender_msn_addr.clone(), &avatar_mxc)));
                    },
                    Err(err) => {
                        log::error!("Couldn't download avatar: {} - {}", &avatar_mxc, err);
                    }
                }
            }

            ns_sender.send(NotificationEventFactory::get_presence(user));
        } 
    
    } else {
        warn!("Could not find user in repo (presence) {}", &event_sender);
    }
}

async fn handle_stripped_room_member_event(ev: StrippedRoomMemberEvent, room: Room, client: Client, me: MSNUser, event_sender: Sender<NotificationEvent>) -> bool {
    let mut notify_ab = false;
    let ab_sender = AB_LOCATOR.get_sender();
    let matrix_token = client.access_token().unwrap();


    let me_matrix_id = me.get_matrix_id();

    log::info!("AB DEBUG - STRIPPED: state_key: {}, sender: {}, membership: {}", &ev.state_key, &ev.sender, &ev.content.membership);

    if room.is_direct() || ev.content.is_direct.unwrap_or(false) {
        log::info!("AB DEBUG - STRIPPED: DIRECT");
        //This is a direct

            match &room {
                Room::Joined(joined_room) => {

                    // COMMENTED BECAUSE WE NEVER RECEIVE THIS EVENT, MAYBE A BUG OF SYNAPSE OR RUST SDK
                    // if ev.content.membership == MembershipState::Join && &ev.state_key == &me_matrix_id && &ev.sender == &me_matrix_id {
                    //     //I Accepted his invitation, REMOVE FROM PENDING LIST, ADD TO ALLOW LIST, ADD TO CONTACT LIST
                    //     let target = Self::get_direct_target_that_isnt_me(&room.direct_targets(), &room, &me_matrix_id).await.unwrap();
                    //     let target_usr = MSNUser::from_matrix_id(target.clone());
                    //     let target_uuid = target_usr.get_uuid();
                    //     let target_msn_addr = target_usr.get_msn_addr();

                    //     log::info!("AB - I Accepted an invite from: {}", &target_msn_addr);
                    //     let inviter_contact = ContactFactory::get_contact(&target_uuid, &target_msn_addr, &target_msn_addr, ContactTypeEnum::Live, false);
                    //     let inviter_allow_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Allow, false);
                    //     let inviter_pending_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Pending, true);

                    //     ab_sender.send(AddressBookEventFactory::get_contact_event(matrix_token.clone(), inviter_contact));
                    //     ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), inviter_allow_member, RoleId::Allow));
                    //     ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), inviter_pending_member, RoleId::Pending));
                    //     notify_ab = true;
                    // }
                },
                Room::Invited(room) => {
                    if &ev.content.membership == &MembershipState::Invite && &ev.state_key == &me_matrix_id {
                        //I've been invited ! ADD TO PENDING LIST WITH INVITE MSG, ADD TO REVERSE LIST

                        let usr_repo = MSNUserRepository::new(client.clone());
                        log::info!("AB - I received an invite DEBUG");
                        let target_usr = usr_repo.get_msnuser(room.room_id(), &ev.sender, false).await.unwrap();
                        let target_uuid = target_usr.get_uuid();
                        let target_msn_addr = target_usr.get_msn_addr();
                        let target_display_name = target_usr.get_display_name();
                        log::info!("AB - I received an invite from: {}", &target_msn_addr);

                        let mut current_pending_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Pending, false);
                        current_pending_member.display_name = Some(target_display_name);
                        let annotation = AnnotationFactory::get_invite(ev.content.reason.unwrap_or(String::new()));
                        let mut annotations = Vec::new();
                        annotations.push(annotation);
                        current_pending_member.annotations=Some(ArrayOfAnnotation{ annotation: annotations });
                
                
                        let current_reverse_member = MemberFactory::get_passport_member(&target_uuid, &target_msn_addr, MemberState::Accepted, RoleId::Reverse, false);
                        
                        ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), current_pending_member, RoleId::Pending));
                        ab_sender.send(AddressBookEventFactory::get_membership_event(matrix_token.clone(), current_reverse_member, RoleId::Reverse));
                        notify_ab = true;

                    }   
                },
                Room::Left(room) => {
                    log::info!("AB DEBUG 1o1DM - STRIPPED: LEFT ROOM");
                }
            }
    } else {
        log::info!("AB DEBUG - STRIPPED: NON DIRECT")
    }
    return notify_ab;
}

async fn handle_sync_room_member_event(ev: SyncRoomMemberEvent, room: Room, client: Client, me: MSNUser, event_sender: Sender<NotificationEvent>) {
  let my_user_id = client.user_id().unwrap();
    let mut notify_ab = false;
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
                            notify_ab = notify_ab || Self::handle_directs(ev, &room, &client, &client.access_token().unwrap(), &me.get_msn_addr()).await;
                        } else {
                            info!("Room is not a direct !!");
                        }
                    } else if let SyncRoomMemberEvent::Redacted(ev) = &ev {
                        info!("ABDEBUG: Redacted event !!");
                    }

    if notify_ab {
        event_sender.send(NotificationEventFactory::get_ab_updated(me.clone()));
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
                
                let typing_user = user_repo.get_msnuser(&room.room_id(), &user_id, false).await.unwrap();

                if &typing_user.get_msn_addr() != &me.get_msn_addr() {

                    let typing_user_payload = MsgPayloadFactory::get_typing_user(typing_user.get_msn_addr().clone());

                    found.on_message_received(typing_user_payload, typing_user, None);
                }
            }
        }
}

async fn handle_sync_room_message_event(ev: SyncRoomMessageEvent, room: Room, client: Client, event_sender: Sender<NotificationEvent>) {
    if let SyncRoomMessageEvent::Original(ev) = ev {
    
        let joined_members = room.joined_members().await.unwrap_or(Vec::new());

        let debug = room.is_direct();
        let debug_len = joined_members.len();

       // if room.is_direct() && joined_members.len() > 0 && joined_members.len() <= 2 {
            let me_user_id =  client.user_id().unwrap();

            if let Some(target) = Self::get_direct_target_that_isnt_me(&room.direct_targets(), &room, &me_user_id).await{

                let room_id = room.room_id().to_string();
                let target_msn_user = MSNUser::from_matrix_id(target.clone());

                if let Some(msn_client) = MSN_CLIENT_LOCATOR.get(){
                    if let Some(found) = msn_client.get_switchboards().find(&room_id) {
                        Self::handle_messages(client.clone(), &room.room_id(), &found, &ev).await;
                    } else {
                             //sb not initialized yet
                            let sb_data = Switchboard::new(client.clone(), room.room_id().to_owned(), client.user_id().unwrap().to_owned());
                            {
                                Self::handle_messages(client.clone(), &room.room_id(), &sb_data, &ev).await;
                            }

                            msn_client.get_switchboards().add(room_id.clone(), sb_data);
                             //send RNG command
                             let session_id = identifiers::get_sb_session_id();

                             let ticket = general_purpose::STANDARD.encode(format!("{target_room_id};{token};{target_matrix_id}", target_room_id = &room_id, token = &client.access_token().unwrap(), target_matrix_id = target.to_string()));

                             event_sender.send(NotificationEventFactory::get_switchboard_init(target_msn_user, session_id, ticket));
                          
                    }  
                }
            }
       // }
    }
}


fn try_fetch_in_direct_targets(direct_targets: &HashSet<OwnedUserId>, me: &UserId) -> Option<OwnedUserId> {
    log::info!("TryGetDirectTarget - target count: {}, me: {}", direct_targets.len(), &me);
    for direct_target in direct_targets {
        if(direct_target != me) {
            log::info!("TryGetDirectTarget - found {}", &direct_target);
            return Some(direct_target.clone());
        }
    }
    log::info!("TryGetDirectTarget - found none");
    return None;
}

async fn get_direct_target_that_isnt_me(direct_targets: &HashSet<OwnedUserId>, room: &Room, me: &UserId) -> Option<OwnedUserId> {
   let mut maybe = Self::try_fetch_in_direct_targets(direct_targets, me);
    if maybe.is_none() {
       let members = room.members_no_sync().await.unwrap();
       log::info!("TryGetDirectTarget2 - members count: {}, me: {}", members.len(), &me);
       for member in members {
        if member.user_id() != me {
            maybe = Some(member.user_id().to_owned());
            log::info!("TryGetDirectTarget2 - members found: {}", &member.user_id());
        }
       }
    }
   return maybe;
}

}