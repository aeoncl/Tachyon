use std::str::from_utf8_unchecked;

use async_trait::async_trait;
use log::info;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
};
use tokio::sync::mpsc;

use crate::sockets::msnp_command::MSNPCommand;

use super::{msnp_command::MSNPCommandParser, tcpserver::TCPServer};

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
            let (socket_sender, mut socket_receiver) = mpsc::unbounded_channel::<String>();

            let (command_sender, mut command_receiver) = mpsc::unbounded_channel::<MSNPCommand>();

            crate::sockets::notification_command_handler2::handle_commands(command_receiver, socket_sender);


            let mut parser = MSNPCommandParser::new();

            let _result = tokio::spawn(async move {

                let (read, mut write) = socket.split();
                let mut reader = BufReader::new(read);
                let mut buffer = [0u8; 2048];
                loop {
                    tokio::select! {
                        bytes_read = reader.read(&mut buffer) => {

                            let bytes_read = bytes_read.unwrap_or(0);

                            //println!("DEBUG: {}", &line);
                            if bytes_read == 0 {
                                break;
                            }

                        //  I'm forced to use the unchecked variant of the from_utf8 function because of P2P packets.
                        //  They contain a binary header which is not UTF8
                        let line = unsafe {from_utf8_unchecked(&buffer[..bytes_read])};

                            let mut commands : Vec<MSNPCommand> = parser.parse_message(line);
                            commands.drain(..).inspect(|command| info!("NS <- {}", &command)).for_each(|command| {command_sender.send(command); });
                            buffer = [0u8; 2048];
                        },
                        command_to_send = socket_receiver.recv() => {
                            let msg = command_to_send.unwrap();
                            info!("NS => {}", &msg);
                            write.write_all(msg.as_bytes()).await;
                            if &msg == "OUT\r\n" {
                                break;
                            }

                        }
                    }
                }
            }).await;

            info!("Client Disconnected");
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
}