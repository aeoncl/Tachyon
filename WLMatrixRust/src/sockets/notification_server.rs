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

use crate::sockets::msnp_command::MSNPCommand;

use super::{
    command_handler::CommandHandler, msnp_command::MSNPCommandParser, notification_command_handler::NotificationCommandHandler, tcpserver::TCPServer
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
            let (tx, mut rx) = mpsc::unbounded_channel::<String>();
            let mut parser = MSNPCommandParser::new();
            let mut command_handler = self.get_command_handler(tx);


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

                            let commands : Vec<MSNPCommand> = parser.parse_message(line);

                            for command in commands {
                                info!("NS <- {}", &command);
                                    match command_handler.handle_command(&command).await {
                                        Ok(response) => {
                                            if !response.is_empty() {
                                                write.write_all(response.as_bytes()).await;
                                                info!("NS -> {}", &response);
                                            }
                                        },
                                        Err(err) => {

                                            let error = format!("{error_code} {tr_id}\r\n", error_code = err.code as i32, tr_id= err.tr_id);
                                            let out = "OUT\r\n";
                                            log::error!("NS -> {}", &error);
                                            log::error!("NS -> {}", &out);
                                            write.write_all(error.as_bytes()).await;
                                            write.write_all(out.as_bytes()).await;
                                        }
                                    }

                            }
                            buffer = [0u8; 2048];
                        },
                        command_to_send = rx.recv() => {
                            let msg = command_to_send.unwrap();
                            info!("NS => {}", &msg);
                            write.write_all(msg.as_bytes()).await;
                        }
                    }
                }
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

    fn get_command_handler(&self, sender: UnboundedSender<String>) -> Box<dyn CommandHandler> {
            return Box::new(NotificationCommandHandler::new(sender));
    }

}