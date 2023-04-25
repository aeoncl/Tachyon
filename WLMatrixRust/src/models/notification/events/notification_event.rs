use crate::{models::{msn_user::MSNUser, msn_object::MSNObject}, generated::payloads::factories::NotificationFactory};

use super::content::{presence_event_content::PresenceEventContent, switchboard_init_event_content::SwitchboardInitEventContent, hotmail_notification_event_content::HotmailNotificationEventContent, disconnect_event_content::DisconnectEventContent};


#[derive(Clone, Debug)]
pub enum NotificationEvent {

    PresenceEvent(PresenceEventContent),
    DisconnectEvent(DisconnectEventContent),
    SwitchboardInitEvent(SwitchboardInitEventContent),
    HotmailNotificationEvent(HotmailNotificationEventContent),
    AddressBookUpdateEvent(HotmailNotificationEventContent)

}

pub struct NotificationEventFactory;

impl NotificationEventFactory {

    pub fn get_ab_updated(sender: MSNUser) -> NotificationEvent {

        let payload = NotificationFactory::test(&sender.get_uuid(), sender.get_msn_addr());

        let content = HotmailNotificationEventContent{
            payload: payload,
        };
        return NotificationEvent::AddressBookUpdateEvent(content);
    }

    pub fn get_disconnect(msn_user: MSNUser) -> NotificationEvent {
        return NotificationEvent::DisconnectEvent(DisconnectEventContent{
            msn_user
        });
    }

    pub fn get_presence(msn_user: MSNUser) -> NotificationEvent {
        return NotificationEvent::PresenceEvent(PresenceEventContent{user: msn_user});
    }

    pub fn get_switchboard_init(inviter: MSNUser, session_id: String, ticket: String) -> NotificationEvent {
        return NotificationEvent::SwitchboardInitEvent(SwitchboardInitEventContent { 
            ip_address: String::from("127.0.0.1"), 
            port: 1864, 
            invite_passport: inviter.get_msn_addr(), 
            invite_name: inviter.get_display_name(), 
            session_id, 
            ticket });
    }

}