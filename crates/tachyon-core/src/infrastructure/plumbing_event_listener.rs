use crate::domain::events::PlumbingEvent;
use crate::infrastructure::app_state::AppState;
use tokio::sync::mpsc::Sender;


pub fn listen_plumbing_events_task(app_state: AppState) -> Sender<PlumbingEvent> {

    let (sender, mut receiver) = tokio::sync::mpsc::channel::<PlumbingEvent>(200);

    tokio::spawn(async move {
        loop {
            if let Some(event) = receiver.recv().await {

                match event {
                    PlumbingEvent::BridgeAnnounce(announce) => {

                    }
                    PlumbingEvent::BridgeGoodbye(bye) => {

                    }
                    PlumbingEvent::BridgeAuth(auth) => {

                    }
                }
            }
        }
    });

    sender
}