use crate::matrix::extensions::direct::DirectRoom;
use crate::matrix::MatrixClient;
use crate::tachyon::state::session::room_proxy_repository::RoomProxyRepository;
use anyhow::Error;
use base64::engine::general_purpose;
use base64::Engine;
use matrix_sdk::ruma::RoomId;
use matrix_sdk::{async_trait, Room};
use msnp::shared::models::email_address::EmailAddress;
use msnp::shared::models::msn_object::{FriendlyName, MSNObjectFactory};
use msnp::shared::models::msn_user::MsnUser;
use sha1::{Digest, Sha1};
use std::str::FromStr;
use std::sync::{Arc, RwLock};

#[async_trait]
pub trait UserService: Send + Sync {

    fn own_user(&self) -> MsnUser;

    async fn resolve_room_proxy_user(&self, room_id: &RoomId) -> Option<MsnUser>;

    async fn resolve_room_proxy_user_from_email(&self, email: &EmailAddress) -> Option<MsnUser>;

    fn resolve_room_proxy_email(&self, room_id: &RoomId) -> Option<EmailAddress>;

    fn find_room_from_email(&self, email: &EmailAddress) -> Result<Option<Room>, Error>;
}

struct UserServiceImpl {
    matrix_client: MatrixClient,
    room_proxies: Arc<dyn RoomProxyRepository>,
    own_user: Arc<RwLock<MsnUser>>
}

impl UserServiceImpl {
    pub fn new(matrix_client: MatrixClient, room_proxies: Arc<dyn RoomProxyRepository>, own_user: Arc<RwLock<MsnUser>>) -> Self {
        Self {
            matrix_client,
            room_proxies,
            own_user
        }
    }
}

#[async_trait]
impl UserService for UserServiceImpl {

    fn own_user(&self) -> MsnUser {
        self.own_user.read().expect("to not be poisonned").clone()
    }

    async fn resolve_room_proxy_user(&self, room_id: &RoomId) -> Option<MsnUser> {
        let email = self.resolve_room_proxy_email(room_id)?;
        let room = &self.matrix_client.get_room(room_id)?;
        room_to_msn_user(email, true, room).await.ok()
    }

    async fn resolve_room_proxy_user_from_email(&self, email: &EmailAddress) -> Option<MsnUser> {
        let room = self.find_room_from_email(email).ok()??;
        room_to_msn_user(email.clone(), true, &room).await.ok()
    }

    fn resolve_room_proxy_email(&self, room_id: &RoomId) -> Option<EmailAddress> {
        let room = self.matrix_client.get_room(room_id)?;
        let proxy_email = map_room_to_proxy_email(&room).ok()?;
        self.room_proxies
            .insert(&proxy_email, room.room_id());
        Some(proxy_email)
    }

    fn find_room_from_email(&self, email: &EmailAddress) -> Result<Option<Room>, Error> {
        let out = if let Some(entry) = self.room_proxies
            .get_room_for_email(email)
        {
            let room_id = entry.value();
            self.matrix_client.get_room(room_id.as_ref())
        } else {
            let mut found = None;

            for current_room in self.matrix_client.rooms() {
                let current_room_proxy_email = map_room_to_proxy_email(&current_room)?;
                if current_room_proxy_email == *email {
                    self.room_proxies
                        .insert(&current_room_proxy_email, current_room.room_id());
                    found = Some(current_room);
                    break;
                }
            }
            found
        };

        Ok(out)
    }
}

fn map_room_to_proxy_email(room: &matrix_sdk::Room) -> anyhow::Result<EmailAddress> {
    let room_info = room.clone_info();

    let room_id_format = room_info.room_version_rules_or_default().room_id_format;

    let room_id = room.room_id();

    let room_id_hashed = hash_room_id(room_id);

    match room_id_format {
        matrix_sdk::ruma::room_version_rules::RoomIdFormatVersion::V1 => {
            let server_name = room_id
                .server_name()
                .expect("RoomIdV1 to contain it's server name");

            let domain = if server_name.as_str().len() > 64 - room_id_hashed.len() - 1 {
                "t.local"
            } else {
                server_name.as_str()
            };

            let email_str = format!("{}@{}", room_id_hashed, &domain);
            Ok(EmailAddress::from_str(email_str.as_str()).expect("Room Email to be valid"))
        }
        matrix_sdk::ruma::room_version_rules::RoomIdFormatVersion::V2 => {
            let server_name = room_info
                .create()
                .expect("RoomCreateEvent to be present")
                .creator
                .server_name();

            let domain = if server_name.as_str().len() > 64 - room_id_hashed.len() - 1 {
                "t.local"
            } else {
                server_name.as_str()
            };

            let email_str = format!("{}@{}", room_id_hashed, &domain);
            Ok(EmailAddress::from_str(email_str.as_str()).expect("Room Email to be valid"))
        }
        _ => Err(anyhow::anyhow!("unhandled room_id_format: {}", room_id)),
    }
}

fn hash_room_id(room_id: &RoomId) -> String {
    let mut hasher = Sha1::new();
    Digest::update(&mut hasher, room_id.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

pub async fn room_to_msn_user(
    email_address: EmailAddress,
    lazy_resolve: bool,
    room: &Room,
) -> Result<MsnUser, Error> {
    let email = email_address;
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

    if let Ok(display_name) = room.display_name().await {
        user.display_name = Some(display_name.to_string());
    } else {
        user.display_name = Some(room.room_id().to_string());
    }

    //Todo chek if direct_target for avatar.
    // Check if we call call endpoint with OPTION or HEAD request to fetch image size and avoid downloading the image at this time

    if let Some(avatar_info) = room.avatar_info() {
        //FIXME: Avatar MSNObject requires to compute the SHA1 of the bytes, let's see if we can avoid that.
        // What happens if no SHA1 is set in the MSNObject?

        let avatar_mxc = room.avatar_url().unwrap();
        let base64_mxc = general_purpose::STANDARD.encode(avatar_mxc.to_string());

        let size = usize::try_from(avatar_info.size.unwrap()).unwrap();
        let display_picture = MSNObjectFactory::get_display_picture_no_bytes(
            size,
            user.get_email_address(),
            format!("{}.tmp", base64_mxc).to_string(),
            FriendlyName::default(),
        );
        user.display_picture = Some(display_picture)
    }

    Ok(user)
}
