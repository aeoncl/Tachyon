use std::{str::from_utf8};

use tokio::{
    io::{AsyncWriteExt, BufReader, AsyncReadExt},
    net::TcpListener,
    sync::broadcast::{self, Sender},
};

use async_trait::async_trait;


use crate::{models::uuid::UUID, sockets::msnp_command::MSNPCommand};

use super::{
    msnp_command::{MSNPCommandParser},
    msnp_command_handlers::{
        CommandHandler, NotificationCommandHandler, SwitchboardCommandHandler,
    }, tcpserver::TCPServer,
};


pub struct NotificationServer {
    url: String,
    port: u32,
}


#[async_trait]
impl TCPServer for NotificationServer {

    async fn listen(&self) {
        let listener = TcpListener::bind(format!("{}:{}", self.url, self.port))
            .await
            .unwrap();
        
        loop {
            let (mut socket, _addr) = listener.accept().await.unwrap();
            let (tx, mut rx) = broadcast::channel::<String>(10);
            let mut command_handler = self.get_command_handler(tx.clone());
            let mut incomplete_command: Option<MSNPCommand> = None;


            let _result = tokio::spawn(async move {
                let (read, mut write) = socket.split();
                let mut reader = BufReader::new(read);
                let mut buffer = [0u8; 2048];
                loop {
                    tokio::select! {
                        bytes_read = reader.read(&mut buffer) => {
                        let mut line = String::from(from_utf8(&buffer).unwrap());
                            //println!("DEBUG: {}", &line);
                             if bytes_read.unwrap_or(0) == 0 {
                                 break;
                             }

                             let mut commands : Vec<MSNPCommand> = Vec::new();

                             if let Some(command_to_fill) = incomplete_command {
                               incomplete_command = None;
                               let (remaining, command) = MSNPCommandParser::parsed_chunked(line.clone(), command_to_fill);
                               line = remaining;
                               commands.push(command);
                             }


                            commands.extend(MSNPCommandParser::parse_message(&line));

                            for command in commands {
                                if command.is_complete() {
                                    println!("NS <= {}", &command);
                                    let response = command_handler.handle_command(&command).await;
                                    if !response.is_empty() {
                                        write.write_all(response.as_bytes()).await;
                                        println!("NS => {}", &response);
                                    }
                                } else {
                                    incomplete_command = Some(command);
                                }
                            }
                            buffer = [0u8; 2048];
                        },
                        command_to_send = rx.recv() => {
                            let msg = command_to_send.unwrap();
                            println!("NS => {}", &msg);
                            write.write_all(msg.as_bytes()).await;
                        }
                    }
                }
                command_handler.cleanup();
            }).await;
        }
    }
}

impl NotificationServer {
    pub fn new(
        url: String,
        port: u32,
    ) -> NotificationServer {
        return NotificationServer {
            url,
            port,
        };
    }

    fn get_command_handler(&self, sender: Sender<String>) -> Box<dyn CommandHandler> {
            return Box::new(NotificationCommandHandler::new(sender));
    }

}