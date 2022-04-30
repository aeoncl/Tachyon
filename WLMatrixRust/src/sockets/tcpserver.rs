use std::{str::from_utf8};

use tokio::{
    io::{AsyncWriteExt, BufReader, AsyncReadExt},
    net::TcpListener,
    sync::broadcast::{self, Sender},
};

use super::{
    msnp_command::{MSNPCommandParser},
    msnp_command_handlers::{
        CommandHandler, NotificationCommandHandler, SwitchboardCommandHandler,
    },
};

pub enum ServerType {
    Notification,
    Switchboard,
}

pub struct TCPServer {
    url: String,
    port: u32,
    server_type: ServerType,
}

impl TCPServer {
    pub fn new(
        url: String,
        port: u32,
        server_type: ServerType,
    ) -> TCPServer {
        return TCPServer {
            url,
            port,
            server_type: server_type,
        };
    }

    fn get_command_handler(&self, sender: Sender<String>) -> Box<dyn CommandHandler> {
        if let ServerType::Notification = self.server_type {
            return Box::new(NotificationCommandHandler::new(sender));
        } else {
            return Box::new(SwitchboardCommandHandler::new());
        }
    }

    pub async fn listen(&self) {
        let listener = TcpListener::bind(format!("{}:{}", self.url, self.port))
            .await
            .unwrap();

        let (tx, _rx) = broadcast::channel::<String>(10);
        loop {
            let (mut socket, _addr) = listener.accept().await.unwrap();

            let tx = tx.clone();
            let mut rx = tx.subscribe();
            let mut command_handler = self.get_command_handler(tx.clone());

            tokio::spawn(async move {
                let (read, mut write) = socket.split();

                let mut reader = BufReader::new(read);
                let mut buffer = [0u8; 2048];
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
                                println!("NS <= {}", &command);
                                let response = command_handler.handle_command(&command).await;
                                if !response.is_empty() {
                                    write.write_all(response.as_bytes()).await;
                                    println!("NS => {}", &response);
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
            });
        }
    }
}
