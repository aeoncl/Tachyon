use async_trait::async_trait;

#[async_trait]
pub trait TCPServer {
    async fn listen(&self);
}
