use std::{str::{from_utf8, from_utf8_unchecked}, ffi::{CString, OsString}};

use log::{warn, info};
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

    fn sanitize_buffer(buffer: &[u8]) -> Vec<u8> {
        let mut out : Vec<u8> = Vec::new();
        let zero : u8 = 0x0;
        for current in buffer {
            if(current != &zero) {
                out.push(current.clone());
            }
        }
        out.push(0x0);
        return out;
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
                let mut buffer = [0u8; 2048];

                let uuid = UUID::new();
                loop {
                    tokio::select! {
                        bytes_read = reader.read(&mut buffer) => {

                        //  I'm forced to use the unchecked variant of the from_utf8 function because of P2P packets.
                        //  They contain a binary header which is not UTF8
                        let line = unsafe {from_utf8_unchecked(&buffer)};

                           //  info!("DEBUG BUFFER\r\n{:?}", &buffer);

                            let mut line = String::from(line);
                            //This is potentially not an UTF8 String
                          //  info!("DEBUG: {}, length: {}", &line, line.len());
                            if bytes_read.unwrap_or(0) == 0 {
                                break;
                            }

                            let mut commands : Vec<MSNPCommand> = Vec::new();

                            if let Some(command_to_fill) = incomplete_command {
                               incomplete_command = None;
                          //     info!("SOME INCOMPLETE STUFF: {}", &line);
                               let (remaining, command) = MSNPCommandParser::parsed_chunked(line.clone(), command_to_fill);
                              line = remaining;
                              commands.push(command);
                            } else {
                         //      info!("NO INCOMPLETE STUFF");

                            }


                           commands.extend(MSNPCommandParser::parse_message(&line));
                        //   info!("PARSING WORKED, {} commands found in msg", commands.len());

                           for command in commands {
                               if command.is_complete() {

                            //    info!("command passed complete check");

                             //   info!("SW {}<= {}", &uuid.to_string(), &command);
                                   let response = command_handler.handle_command(&command).await;
                                   if !response.is_empty() {
                                       write.write_all(response.as_bytes()).await;
                                       info!("SW {}=> {}",&uuid.to_string(), &response);
                                   }
                               } else {
                      //          info!("command failed complete check");
                                   incomplete_command = Some(command);
                               }

                           }
                        
                            buffer = [0u8; 2048];
                        },
                        command_to_send = rx.recv() => {
                            let msg = command_to_send.unwrap();
                            info!("SW {}=> {}",&uuid.to_string(), &msg);
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