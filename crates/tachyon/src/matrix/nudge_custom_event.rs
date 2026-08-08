use matrix_sdk::ruma::events::macros::EventContent;
use matrix_sdk::ruma::exports::serde::{Deserialize, Serialize};
use ruma::events::room::message::{MessageType, RoomMessageEventContent};


pub fn create_fireworks_message(content: &str) -> RoomMessageEventContent {

    let msg_type = MessageType::new(
        "nic.custom.fireworks",
        content.to_owned(),
        serde_json::json!({ "mentions": "{}" })
            .as_object()
            .unwrap()
            .clone(),
    ).unwrap();

    RoomMessageEventContent::new(msg_type)
}


pub fn create_buzz_message(sender_name: &str) -> RoomMessageEventContent {

    let msg_type = MessageType::new(
        "chat.tachyon.buzz",
        format!("\u{1fae8} {} sent a buzz!", sender_name).to_owned(),
        serde_json::json!({})
            .as_object()
            .unwrap()
            .clone(),
    ).unwrap();

    RoomMessageEventContent::new(msg_type)
}