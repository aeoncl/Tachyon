//! Room/member → MSN identity resolution (§3.5, H7). Pre-D3 the room-hash
//! tables were process-global lazily-initialized maps — unowned, never
//! reset, shared across accounts (§1.4 H7). They are now an owned
//! `RoomHashCache` field of `TachyonClientInner`: lifetime = the signed-in
//! account's lifetime, fixed by ownership, not by a compound key. The traits
//! on SDK types (`ToMsnUser for Room`, `ToEmailAddress for Room`,
//! `FindRoomFromEmail for Client`) could not reach owned state without a
//! global, so they are retired in favor of explicit functions that take the
//! cache — every call site already holds the `TachyonClient` that owns it.
use crate::matrix::extensions::direct::DirectRoom;
use anyhow::Error;
use base64::engine::general_purpose;
use base64::Engine;
use dashmap::DashMap;
use matrix_sdk::room::RoomMember;
use matrix_sdk::ruma::{OwnedRoomId, RoomId};
use matrix_sdk::{Client, Room};
use msnp::shared::models::msn_object::{FriendlyName, MSNObjectFactory};
use msnp::shared::models::{email_address::EmailAddress, msn_user::MsnUser};
use sha1::digest::DynDigest;
use sha1::{Digest, Sha1};
use std::str::FromStr;
use crate::tachyon::mappers::user_id::MatrixIdCompatible;

/// Owned room-id hash cache (§3.5): the pre-D3 global hash-table +
/// hash-cache map pair, one instance per signed-in account. The forward
/// map memoizes sha1(room_id); the reverse map turns a hashed room email
/// back into a room id without scanning every room.
#[derive(Default)]
pub struct RoomHashCache {
    by_hash: DashMap<String, OwnedRoomId>,
    by_room: DashMap<OwnedRoomId, String>,
}

impl RoomHashCache {
    /// sha1-hex of the room id, memoized in both directions (the pre-D3
    /// `hash_room_id_cached` semantics, owned).
    pub fn hash_room_id(&self, room_id: &RoomId) -> String {
        match self.by_room.get(room_id) {
            None => {
                let hash = hash_room_id(room_id);
                self.by_hash.insert(hash.clone(), room_id.to_owned());
                self.by_room.insert(room_id.to_owned(), hash.clone());
                hash
            }
            Some(hash) => { hash.value().clone() }
        }
    }

    /// The room id a hashed room email stands for, if this account ever
    /// hashed it.
    pub fn room_id_for_hash(&self, hash: &str) -> Option<OwnedRoomId> {
        self.by_hash.get(hash).map(|entry| entry.value().clone())
    }
}

/// Room → its synthetic MSN address: `sha1(room_id)@server`, capped to the
/// 64-char local-part budget with the `t.local` fallback (pre-D3 semantics
/// verbatim). §1.4: the pre-D3 `.expect(...)`s panicked the calling task on
/// a malformed room; they now return Err (failure-path-only delta — callers
/// warn and skip).
pub fn room_email_address(room: &Room, hashes: &RoomHashCache) -> Result<EmailAddress, Error> {
    let room_info = room.clone_info();

    let room_id_format = room_info.room_version_rules_or_default().room_id_format;

    let room_id = room.room_id();

    let room_id_hashed = hashes.hash_room_id(room_id);

    match room_id_format {
        matrix_sdk::ruma::room_version_rules::RoomIdFormatVersion::V1 => {
            let server_name = room_id
                .server_name()
                .ok_or_else(|| anyhow::anyhow!("RoomIdV1 without a server name: {}", room_id))?;

            let domain = if server_name.as_str().len() > 64 - room_id_hashed.len() - 1 {
                "t.local"
            } else {
                server_name.as_str()
            };

            let email_str = format!("{}@{}", room_id_hashed, &domain);
            EmailAddress::from_str(email_str.as_str())
                .map_err(|e| anyhow::anyhow!("room email {} is not a valid address: {}", email_str, e))
        }
        matrix_sdk::ruma::room_version_rules::RoomIdFormatVersion::V2 => {

            let server_name = room_info
                .create()
                .ok_or_else(|| anyhow::anyhow!("V2 room {} has no create event", room_id))?
                .creator
                .server_name();

            let domain = if server_name.as_str().len() > 64 - room_id_hashed.len() - 1 {
                "t.local"
            } else {
                server_name.as_str()
            };

            let email_str = format!("{}@{}", room_id_hashed, &domain);
            EmailAddress::from_str(email_str.as_str())
                .map_err(|e| anyhow::anyhow!("room email {} is not a valid address: {}", email_str, e))
        }
        _ => {
            Err(anyhow::anyhow!("unhandled room_id_format: {}", room_id))
        }
    }
}

/// The hashed room email → room lookup (pre-D3 `FindRoomFromEmail`): the
/// owned reverse map first, a linear hash-compare scan over the account's
/// rooms as the fallback.
pub fn find_room_from_email(client: &Client, hashes: &RoomHashCache, email: &EmailAddress) -> Result<Option<Room>, Error> {

    let (room_id_hashed, _server_name) = email.crack();

    let out = if let Some(room_id) = hashes.room_id_for_hash(room_id_hashed) {
        client.get_room(room_id.as_ref())
    } else {

        let mut found = None;

        for curent_room in client.rooms() {
            let current_room_hash = hashes.hash_room_id(curent_room.room_id());
            if current_room_hash == room_id_hashed {
                found = Some(curent_room);
                break;
            }
        }
        found
    };

    Ok(out)
}

/// Room → MsnUser, resolving the direct target through a server sync when
/// needed (pre-D3 `to_msn_user`).
pub async fn room_to_msn_user(room: &Room, hashes: &RoomHashCache) -> Result<MsnUser, Error> {
    to_msn_user_internal(room, hashes, false).await
}

/// Room → MsnUser from the local store only (pre-D3 `to_msn_user_lazy`).
pub async fn room_to_msn_user_lazy(room: &Room, hashes: &RoomHashCache) -> Result<MsnUser, Error> {
    to_msn_user_internal(room, hashes, true).await
}

async fn to_msn_user_internal(room: &Room, hashes: &RoomHashCache, lazy_resolve: bool) -> Result<MsnUser, Error> {
    let email = room_email_address(room, hashes)?;
    let mut user = MsnUser::with_email_addr(email);

    let maybe_direct_target = if room.is_valid_one_to_one_direct() {
         if lazy_resolve {
             room.get_single_direct_target_member_lazy().await
        } else {
             room.get_single_direct_target_member().await
         }
    } else {
        Ok(None)
    };
    // Pre-D3 this resolved member was never read either — the lookup (and
    // its store side effects) is preserved verbatim, not "cleaned up" in a
    // migration commit.
    let _ = maybe_direct_target;



        if let Ok(display_name) = room.display_name().await {
            user.display_name = Some(display_name.to_string());
        } else {
            user.display_name = Some(room.room_id().to_string());
        }


    //Todo chek if direct_target for avatar.
    // Check if we call call endpoint with OPTION or HEAD request to fetch image size and avoid downloading the image at this time

    // §1.4: pre-D3 `room.avatar_url().unwrap()` and `avatar_info.size
    // .unwrap()` panicked the calling task whenever an avatar event carried
    // info without a url or without a size — the SDK allows both
    // independently. Now: the user resolves WITHOUT a display picture
    // instead (failure-path-only delta).
    if let Some(avatar_info) = room.avatar_info() {
        //FIXME: Avatar MSNObject requires to compute the SHA1 of the bytes, let's see if we can avoid that.
        // What happens if no SHA1 is set in the MSNObject?

        if let Some(avatar_mxc) = room.avatar_url() {
            if let Some(size) = avatar_info.size.and_then(|size| usize::try_from(size).ok()) {
                let base64_mxc =  general_purpose::STANDARD.encode(avatar_mxc.to_string());

                let display_picture = MSNObjectFactory::get_display_picture_no_bytes(size, user.get_email_address(), format!("{}.tmp", base64_mxc).to_string(), FriendlyName::default());
                user.display_picture = Some(display_picture)
            }
        }
    }

    Ok(user)
}

/// RoomMember → MsnUser (display name only; the avatar TODO is pre-D3).
/// Infallible and sync — the pre-D3 trait returned a `Result` that could
/// never fail and an `async fn` that never awaited.
pub fn member_to_msn_user(member: &RoomMember) -> MsnUser {
    let mut msn_user = MsnUser::from_user_id(member.user_id());
    msn_user.display_name = member.display_name().map(|name| name.to_string());

    //TODO Display Picture

    msn_user
}

fn hash_room_id(room_id: &RoomId) -> String {
    let mut hasher = Sha1::new();
    Digest::update(&mut hasher, room_id.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use matrix_sdk::ruma::room_id;

    use super::RoomHashCache;

    #[test]
    fn the_same_room_always_hashes_to_the_same_sha1_hex_and_maps_back() {
        let cache = RoomHashCache::default();
        let room = room_id!("!abc123:test.local");

        let first = cache.hash_room_id(room);
        // served from the owned cache the second time — same value by
        // construction, and the reverse map resolves it without a scan.
        let second = cache.hash_room_id(room);

        assert_eq!(first, second);
        assert_eq!(first.len(), 40, "sha1 hex is 40 chars");
        assert_eq!(cache.room_id_for_hash(&first).as_deref(), Some(room));
    }

    #[test]
    fn an_unknown_hash_resolves_to_no_room() {
        let cache = RoomHashCache::default();

        assert!(cache.room_id_for_hash("0000000000000000000000000000000000000000").is_none());
    }

    #[test]
    fn two_caches_do_not_share_their_rooms() {
        // H7's point: the state is owned per account — a second cache starts
        // empty even for a room the first one already hashed, while the hash
        // itself stays the pure sha1 it always was.
        let first = RoomHashCache::default();
        let room = room_id!("!abc123:test.local");
        let hash = first.hash_room_id(room);

        let second = RoomHashCache::default();

        assert!(second.room_id_for_hash(&hash).is_none());
        assert_eq!(second.hash_room_id(room), hash);
    }
}
