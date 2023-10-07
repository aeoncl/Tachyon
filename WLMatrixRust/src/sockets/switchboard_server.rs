use std::str::from_utf8_unchecked;

use async_trait::async_trait;
use log::info;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast::{self, Sender},
};
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedSender;

use crate::{models::uuid::UUID, sockets::msnp_command::MSNPCommand};

use super::{
    command_handler::CommandHandler, events::socket_event::SocketEvent, msnp_command::MSNPCommandParser, switchboard_command_handler::SwitchboardCommandHandler, tcpserver::TCPServer,
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

    fn get_command_handler(&self, sender: UnboundedSender<SocketEvent>) -> Box<dyn CommandHandler> {
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
            let (tx, mut rx) = mpsc::unbounded_channel::<SocketEvent>();
            let mut command_handler = self.get_command_handler(tx);
            let mut parser = MSNPCommandParser::new();


            let _result = tokio::spawn(async move {
                let (read, mut write) = socket.split();
                let mut reader = BufReader::new(read);
                let mut buffer = [0u8; 2048];

                let uuid = UUID::new();
                loop {
                    tokio::select! {
                        bytes_read = reader.read(&mut buffer) => {

                        let bytes_read = bytes_read.unwrap_or(0);
                           //  info!("DEBUG BUFFER\r\n{:?}", &buffer);
                            //info!("DEBUG: {}, length: {}", &line, line.len());
                            if bytes_read == 0 {
                                break;
                            }

                            
                        //  I'm forced to use the unchecked variant of the from_utf8 function because of P2P packets.
                        //  They contain a binary header which is not UTF8
                        let line = unsafe {from_utf8_unchecked(&buffer[..bytes_read])};


                            let commands : Vec<MSNPCommand> = parser.parse_message(line);

                           for command in commands {
                                info!("SW {} <- {}", &uuid.to_string(), &command);
                                   let response = command_handler.handle_command(&command).await.unwrap();
                                   if !response.is_empty() {
                                       write.write_all(response.as_bytes()).await;
                                       info!("SW {} -> {}",&uuid.to_string(), &response);
                                   }
                           }
                            buffer = [0u8; 2048];
                        },
                        command_to_send = rx.recv() => {
                            let msg = command_to_send.unwrap();
                            match msg {
                                SocketEvent::Single(content) => {
                                    info!("SW {} -> {}",&uuid.to_string(), &content);
                                    write.write_all(content.as_bytes()).await;
                                },
                                SocketEvent::Multiple(content) => {
                                    for current in content {
                                        info!("SW {} -> {}",&uuid.to_string(), &current);
                                        write.write_all(current.as_bytes()).await;
                                    }
                                }
                            }
                        
                        }
                    }
                }
            });



        }
    }
}

/*

  match msg {
                                SocketEvent::Single(msg) => {
                                    info!("SW {} -> {}",&uuid.to_string(), &msg);
                                    write.write_all(msg.as_bytes()).await;
                                },
                                SocketEvent::Multiple(msgs) => {
                                    for msg in msgs {
                                        info!("SW {} -> {}",&uuid.to_string(), &msg);
                                        write.write_all(msg.as_bytes()).await;
                                    }
                                }
                            }

 */