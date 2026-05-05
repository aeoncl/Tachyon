use std::sync::{Arc, Mutex};

use matrix_sdk::{async_trait, Client};
use msnp::shared::models::email_address::EmailAddress;
use msnp::soap::abch::msnab_datatypes::{BaseMember, ContactType};

use crate::notification::models::soap_holder::AddressBookContact;
use crate::tachyon::services::session::user_service::UserService;

#[async_trait]
pub trait ContactService: Send + Sync {
    fn push_delta(&self, contact: AddressBookContact);

    fn drain_contact_deltas(&self) -> Vec<ContactType>;

    fn push_membership_delta(&self, member: BaseMember);

    fn push_membership_deltas(&self, members: Vec<BaseMember>);

    fn drain_membership_deltas(&self) -> Vec<BaseMember>;

    fn has_pending_deltas(&self) -> bool;

    async fn compute_all_contacts(&self) -> Vec<AddressBookContact>;

    async fn add_contact(
        &self,
        email: &EmailAddress,
        invite_msg: Option<&str>,
    ) -> anyhow::Result<()>;

    async fn remove_contact(&self, email: &EmailAddress) -> anyhow::Result<()>;

    async fn block_contact(&self, email: &EmailAddress) -> anyhow::Result<()>;

    async fn unblock_contact(&self, email: &EmailAddress) -> anyhow::Result<()>;
}

pub struct ContactServiceImpl {
    delta_buffer: Arc<Mutex<Vec<AddressBookContact>>>,
    membership_delta_buffer: Arc<Mutex<Vec<BaseMember>>>,

    matrix_client: Client,

    user_service: Arc<dyn UserService>,
}

impl ContactServiceImpl {
    pub fn new(
        delta_buffer: Arc<Mutex<Vec<AddressBookContact>>>,
        membership_delta_buffer: Arc<Mutex<Vec<BaseMember>>>,
        matrix_client: Client,
        user_service: Arc<dyn UserService>,
    ) -> Self {
        Self {
            delta_buffer,
            membership_delta_buffer,
            matrix_client,
            user_service,
        }
    }
}

#[async_trait]
impl ContactService for ContactServiceImpl {
    fn push_delta(&self, contact: AddressBookContact) {
        let mut buf = self.delta_buffer.lock().expect("delta_buffer lock");
        buf.push(contact);
    }

    fn drain_contact_deltas(&self) -> Vec<ContactType> {
        let mut buf = self.delta_buffer.lock().expect("delta_buffer lock");
        buf.drain(..)
            .filter_map(|c| match c {
                AddressBookContact::Contact(ct) => Some(ct),
                AddressBookContact::Circle(_) => None,
            })
            .collect()
    }

    fn push_membership_delta(&self, member: BaseMember) {
        let mut buf = self
            .membership_delta_buffer
            .lock()
            .expect("membership_delta_buffer lock");
        buf.push(member);
    }

    fn push_membership_deltas(&self, members: Vec<BaseMember>) {
        let mut buf = self
            .membership_delta_buffer
            .lock()
            .expect("membership_delta_buffer lock");
        buf.extend(members);
    }

    fn drain_membership_deltas(&self) -> Vec<BaseMember> {
        let mut buf = self
            .membership_delta_buffer
            .lock()
            .expect("membership_delta_buffer lock");
        buf.drain(..).collect()
    }

    fn has_pending_deltas(&self) -> bool {
        let contact_buf = self.delta_buffer.lock().expect("delta_buffer lock");
        let member_buf = self
            .membership_delta_buffer
            .lock()
            .expect("membership_delta_buffer lock");
        !contact_buf.is_empty() || !member_buf.is_empty()
    }

    async fn compute_all_contacts(&self) -> Vec<AddressBookContact> {
        crate::matrix::handlers::contact_handlers::compute_all_contacts(
            self.matrix_client.clone(),
            self.user_service.clone(),
        )
        .await
    }

    async fn add_contact(
        &self,
        email: &EmailAddress,
        invite_msg: Option<&str>,
    ) -> anyhow::Result<()> {
        let user_id = crate::tachyon::mappers::user_id::MatrixIdCompatible::to_owned_user_id(email);

        match self.matrix_client.get_dm_room(&user_id) {
            Some(room) => {
                use crate::matrix::extensions::direct::DirectRoom;
                if !room.is_valid_one_to_one_direct() {
                    let dm = self.matrix_client.create_dm(&user_id).await?;
                    if let Some(msg) = invite_msg {
                        use crate::matrix::extensions::message_dedup::SendWithDedup;
                        let content = matrix_sdk::ruma::events::room::message::RoomMessageEventContent::text_plain(msg);
                        let _ = dm.send_with_dedup(content).await;
                    }
                } else {
                    // Valid 1:1 room exists — re-invite if needed
                    if let Some(member) = room.get_member(&user_id).await? {
                        use matrix_sdk::ruma::events::room::member::MembershipState;
                        match member.membership() {
                            MembershipState::Invite | MembershipState::Join => {}
                            _ => {
                                room.invite_user_by_id(&user_id).await?;
                            }
                        }
                    }
                }
            }
            None => {
                let dm = self.matrix_client.create_dm(&user_id).await?;
                if let Some(msg) = invite_msg {
                    use crate::matrix::extensions::message_dedup::SendWithDedup;
                    let content = matrix_sdk::ruma::events::room::message::RoomMessageEventContent::text_plain(msg);
                    let _ = dm.send_with_dedup(content).await;
                }
            }
        }

        Ok(())
    }

    async fn remove_contact(&self, email: &EmailAddress) -> anyhow::Result<()> {
        match self.user_service.find_room_from_email(email)? {
            Some(room) => {
                room.leave().await?;
                Ok(())
            }
            None => Err(anyhow::anyhow!("No room found for contact {}", email)),
        }
    }

    async fn block_contact(&self, email: &EmailAddress) -> anyhow::Result<()> {
        // Blocking is equivalent to leaving the room
        self.remove_contact(email).await
    }

    async fn unblock_contact(&self, email: &EmailAddress) -> anyhow::Result<()> {
        // Unblocking is equivalent to re-adding
        self.add_contact(email, None).await
    }
}
