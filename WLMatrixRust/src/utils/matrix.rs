use std::{collections::HashSet, path::Path};

use matrix_sdk::{Client, ruma::{UserId, DeviceId, device_id, user_id, events::{AnySyncMessageLikeEvent, SyncMessageLikeEvent, OriginalSyncStateEvent, room::member::{RoomMemberEventContent, MembershipState}}, OwnedUserId}, Session, Error};
use reqwest::Url;

use super::identifiers::get_matrix_device_id;

pub fn save_mtx_timestamp(msn_addr: &String, mtx_timestamp: String) {


}

pub fn load_mtx_timestamp(msn_addr: &String) -> String {
    return String::new();
}

