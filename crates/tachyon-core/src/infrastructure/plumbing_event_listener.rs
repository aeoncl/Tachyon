use crate::domain::events::TachyonEvent;
use crate::infrastructure::app_state::AppState;
use tokio::sync::mpsc::Sender;


pub fn listen_plumbing_events_task(app_state: AppState) -> Sender<TachyonEvent> {

    let (sender, mut receiver) = tokio::sync::mpsc::channel::<TachyonEvent>(200);

    tokio::spawn(async move {
        loop {
            if let Some(event) = receiver.recv().await {

                match event {
                    TachyonEvent::BridgeAnnounce(announce) => {

                    }
                    TachyonEvent::BridgeGoodbye(bye) => {

                    }
                    TachyonEvent::BridgeAuth(auth) => {

                    }
                }
            }
        }
    });

    sender
}