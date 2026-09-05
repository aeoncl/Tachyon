use async_trait::async_trait;
use crate::domain::error::TachyonResult;
use crate::domain::events::BridgeEvent;

#[async_trait]
pub trait BridgeHandle: Send + Sync {
    async fn send(event: BridgeEvent) -> TachyonResult<()>;

}