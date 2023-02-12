use super::content::presence_event_content::PresenceEventContent;


#[derive(Clone, Debug)]
pub enum NotificationEvents {

    PresenceEvent(PresenceEventContent),
    SwitchboardInitEvent,
    

}