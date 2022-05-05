use std::{str::from_utf8};

use tokio::{
    io::{AsyncWriteExt, BufReader, AsyncReadExt},
    net::TcpListener,
    sync::broadcast::{self, Sender},
};

use async_trait::async_trait;


use crate::models::uuid::UUID;

use super::{
    msnp_command::{MSNPCommandParser},
    msnp_command_handlers::{
        CommandHandler, NotificationCommandHandler, SwitchboardCommandHandler,
    }, tcpserver::TCPServer,
};

pub struct SwitchboardServer {
    url: String,
    port: u32,
}

impl SwitchboardServer {
    pub fn new(
        url: String,
        port: u32,
    ) -> SwitchboardServer {
        return SwitchboardServer {
            url,
            port,
        };
    }

    fn get_command_handler(&self, sender: Sender<String>) -> Box<dyn CommandHandler> {
        return Box::new(SwitchboardCommandHandler::new(sender));
    }

}

#[async_trait]
impl TCPServer for SwitchboardServer {

    async fn listen(&self) {
        let listener = TcpListener::bind(format!("{}:{}", self.url, self.port))
            .await
            .unwrap();

        loop {
            let (mut socket, _addr) = listener.accept().await.unwrap();
            let (tx, mut rx) = broadcast::channel::<String>(10);
            let mut command_handler = self.get_command_handler(tx.clone());
            

            let _result = tokio::spawn(async move {
                let (read, mut write) = socket.split();
                let mut reader = BufReader::new(read);
                let mut buffer = [0u8; 2048];

                let uuid = UUID::new();
                loop {
                    tokio::select! {
                        bytes_read = reader.read(&mut buffer) => {
                        let line = String::from(from_utf8(&buffer).unwrap());
                            //println!("DEBUG: {}", &line);
                             if bytes_read.unwrap_or(0) == 0 {
                                 break;
                             }
                             let commands = MSNPCommandParser::parse_message(&line);

                            for command in commands {
                                println!("SW {}<= {}", &uuid.to_string(), &command);
                                let response = command_handler.handle_command(&command).await;
                                if !response.is_empty() {
                                    write.write_all(response.as_bytes()).await;
                                    println!("SW {}=> {}",&uuid.to_string(), &response);
                                }
                            }
                            buffer = [0u8; 2048];
                        },
                        command_to_send = rx.recv() => {
                            let msg = command_to_send.unwrap();
                            println!("SW {}=> {}",&uuid.to_string(), &msg);
                            write.write_all(msg.as_bytes()).await;
                        }
                    }
                }
                //Cleanup
                command_handler.cleanup();
            });
        }
    }
}