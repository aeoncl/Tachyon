use dashmap::DashMap;
use std::sync::Arc;
use thiserror::Error;
use thiserror::__private::AsDynError;
use crate::notification::models::client_data::ClientData;
//     pub async fn send_with_retry(&self, payload: NotificationServerCommand) -> Result<(), TachyonError> {
//
//         let retry_strategy = ExponentialBackoff::from_millis(10)
//             .factor(2)
//             .max_delay_millis(1000)
//             .max_interval(1000)
//             .take(3);
//
//         let sender = self.notification_sender.clone();
//
//         Retry::spawn(retry_strategy,move || {
//
//             let sender_clone = sender.clone();
//             let payload = payload.clone();
//             async move {
//                 if let Err(err) = sender_clone.try_send(payload) {
//                     match err {
//                         tokio::sync::mpsc::error::TrySendError::Closed(_) => {
//                             Err(RetryError::Permanent(err))
//                         }
//                         tokio::sync::mpsc::error::TrySendError::Full(_) => {
//                             Err(RetryError::Transient{
//                                 err,
//                                 retry_after: None,
//                             })
//                         }
//                     }
//                 } else {
//                     Ok(())
//                 }
//             }
//
//         }).await.map_err(|err|  TachyonError::NotificationChannelError)?;
//
//         Ok(())
//     }
// }

#[derive(Error, Debug)]
pub enum ClientStoreError {
    #[error("Mutex lock was poisonned: {}", .name)]
    PoisonnedLockError { name: String, source: anyhow::Error },
}

#[derive(Clone, Default)]
pub struct ClientStoreFacade {
    data: Arc<DashMap<String, ClientData>>,
}

impl ClientStoreFacade {
    pub fn get_single_client_data(&self) -> Option<ClientData> {
        if self.data.len() > 1 {
            return None;
        }

        self.data.iter().next().map(|x| x.value().clone())
    }

    pub fn get_client_data(&self, key: &str) -> Option<ClientData> {
        match self.data.get(key) {
            None => None,
            Some(found) => Some(found.value().clone()),
        }
    }

    pub fn insert_client_data(&self, key: String, client_data: ClientData) {
        self.data.insert(key, client_data);
    }

    pub fn remove_client_data(&self, key: &str) -> Option<(String, ClientData)> {
        self.data.remove(key)
    }
}
