use std::str::from_utf8_unchecked;

use async_trait::async_trait;
use log::info;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
};

use crate::models::msn_user::MSNUser;

use super::tcpserver::TCPServer;

pub struct EchoServer {
    url: String,
    port: u32
}


#[async_trait]
impl TCPServer for EchoServer {

    async fn listen(&self) {
        let listener = TcpListener::bind(format!("{}:{}", self.url, self.port))
            .await
            .unwrap();
        
        loop {
            let (mut socket, _addr) = listener.accept().await.unwrap();
            let mut real_user: Option<MSNUser> = None;
            let mut proxy_user: Option<MSNUser> = None;

            let _result = tokio::spawn(async move {
                let (read, mut write) = socket.split();
                let mut reader = BufReader::new(read);
                let mut buffer = [0u8; 2048];
                loop {
                    tokio::select! {
                        bytes_read = reader.read(&mut buffer) => {
                            info!("echo << BUFFER: {:?}", &buffer);

                            let line = unsafe {from_utf8_unchecked(&buffer)};
                            info!("echo << STRING: {}", &line);

                            if bytes_read.unwrap_or(0) == 0 {
                                break;
                            }
                            write.write(&buffer).await;
                            buffer = [0u8; 2048];
                        }
                    }
                }
                //cleanup
            });
        }
    }



}

impl EchoServer {
    pub fn new(
        url: String,
        port: u32,
    ) -> EchoServer {
        return EchoServer {
            url,
            port,
        };
    }
}