use std::sync::Mutex;

use tokio::sync::broadcast::{self, Receiver, Sender};

use crate::{generated::msnab_datatypes::types::ContactType, models::abch::events::AddressBookEvent};

pub struct ABLocator {
    contact_receiver: Mutex<Receiver<AddressBookEvent>>,
    member_receiver: Mutex<Receiver<AddressBookEvent>>,
    sender: Sender<AddressBookEvent>
}

impl ABLocator {
    pub fn new() -> Self {
        let (sender, mut receiver) = broadcast::channel::<AddressBookEvent>(10000);
        let contact_receiver = receiver.resubscribe();
        return Self { contact_receiver: Mutex::new(contact_receiver), member_receiver: Mutex::new(receiver), sender};
    }

    pub async fn get_contacts(&self, ticket_token: &str) -> Result<(Vec<ContactType>, bool), ()>{
        let mut out = Vec::new();
        let mut contact_receiver = self.contact_receiver.lock().or(Err(()))?;
        let mut profile_update = false;

        while !contact_receiver.is_empty() {
           match contact_receiver.recv().await {
            Ok(event) => {

                match event {
                    AddressBookEvent::ContactEvent(ev) => {
                        if &ev.token == ticket_token {
                            out.push(ev.contact);
                        }
                    },
                    AddressBookEvent::ExpressionProfileUpdateEvent => {
                        profile_update = true;
                    },
                    _ => {}
                }
            },
            Err(err) => {
                log::error!("An error has occured consuming contact events: {}", err);
            }
           }
        }

        return Ok((out, profile_update));
    }

    pub async fn get_membership_events(&self, ticket_token: &String) -> Result<Vec<AddressBookEvent>, ()>{
        let mut out = Vec::new();
        let mut member_receiver = self.member_receiver.lock().or(Err(()))?;

        while !member_receiver.is_empty() {
           match member_receiver.recv().await {
            Ok(event) => {
                if let AddressBookEvent::MembershipEvent(ev)= &event {
                    if &ev.token == ticket_token {
                        out.push(event);
                    }
                }
            },
            Err(err) => {
                log::error!("An error has occured consuming membership events: {}", err);
            }
           }
        }
        return Ok(out);
    }

    pub fn get_sender(&self) -> Sender<AddressBookEvent> {
        return self.sender.clone();
    }
}