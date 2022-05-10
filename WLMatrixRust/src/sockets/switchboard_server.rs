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
            let mut incomplete_command: Option<MSNPCommand> = None;


            let _result = tokio::spawn(async move {
                let (read, mut write) = socket.split();
                let mut reader = BufReader::new(read);
                let mut buffer = [0u8; 2048]; //TODO shit workaround, support message chunking in the parser.

                let uuid = UUID::new();
                loop {
                    tokio::select! {
                        bytes_read = reader.read(&mut buffer) => {
                        let mut line = String::from(from_utf8(&buffer).unwrap());

                             println!("DEBUG: {}, length: {}", &line, line.len());
                             if bytes_read.unwrap_or(0) == 0 {
                                 break;
                             }

                             let mut commands : Vec<MSNPCommand> = Vec::new();

                             if let Some(command_to_fill) = incomplete_command {
                                incomplete_command = None;
                                println!("SOME INCOMPLETE STUFF: {}", &line);
                                let (remaining, command) = MSNPCommandParser::parsed_chunked(line.clone(), command_to_fill);
                               line = remaining;
                               commands.push(command);
                             } else {
                                println!("NO INCOMPLETE STUFF");

                             }


                            commands.extend(MSNPCommandParser::parse_message(&line));
                            println!("PARSING WORKED, {} commands found in msg", commands.len());

                            for command in commands {
                                if command.is_complete() {

                                    println!("command passed complete check");

                                    println!("SW {}<= {}", &uuid.to_string(), &command);
                                    let response = command_handler.handle_command(&command).await;
                                    if !response.is_empty() {
                                        write.write_all(response.as_bytes()).await;
                                        println!("SW {}=> {}",&uuid.to_string(), &response);
                                    }
                                } else {
                                    println!("command failed complete check");
                                    incomplete_command = Some(command);
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