use std::{str::{from_utf8, from_utf8_unchecked}};

use byteorder::{BigEndian, ByteOrder, LittleEndian};
use log::info;
use tokio::{
    io::{AsyncWriteExt, BufReader, AsyncReadExt},
    net::TcpListener,
    sync::broadcast::{self, Sender},
};

use async_trait::async_trait;


use crate::{models::{uuid::UUID, p2p::{pending_packet::PendingPacket, p2p_session::P2PSession}, msn_user::MSNUser}, sockets::{msnp_command::MSNPCommand, msnp2p_command::{P2PCommandParser, P2PCommand}}, P2P_REPO};

use super::{
    msnp_command::{MSNPCommandParser}, tcpserver::TCPServer
};


pub struct P2PServer {
    url: String,
    port: u32
}


#[async_trait]
impl TCPServer for P2PServer {

    async fn listen(&self) {
        let listener = TcpListener::bind(format!("{}:{}", self.url, self.port))
            .await
            .unwrap();
        
        loop {
            let (mut socket, _addr) = listener.accept().await.unwrap();
            let (tx, mut rx) = broadcast::channel::<PendingPacket>(10);
            let mut p2p_session = P2PSession::new(tx.clone());

            let seq_number=  P2P_REPO.get_seq_number();
            info!("fetched seq_num: {:x}", seq_number);
            p2p_session.set_seq_number(seq_number);
            p2p_session.set_initialized(true);
            // p2p_session.listen_for_raks();

            let mut real_user: Option<MSNUser> = None;
            let mut proxy_user: Option<MSNUser> = None;
            let mut do_nonce: bool = true;
            let mut parser = P2PCommandParser::new();

            let _result = tokio::spawn(async move {
                let (read, mut write) = socket.split();
                let mut reader = BufReader::new(read);
                let mut buffer = [0u8; 4096];
                loop {
                    tokio::select! {
                        bytes_read = reader.read(&mut buffer) => {
                           // info!("P2P << BUFFER: {:?}", &buffer);

                            let line = unsafe {from_utf8_unchecked(&buffer)};

                        //    info!("P2P << STRING: {}", &line);
                            let bytes_read = bytes_read.unwrap_or(0);
                            if bytes_read == 0 {
                                break;
                            }

                            let commands: Vec<P2PCommand> = parser.parse_message(&buffer[0..bytes_read], do_nonce);
                            
                            for command in commands {
                            //info!("P2P << msg");

                            //info!("P2P << {:?}", &command);

                                    if command.is_nonce() && do_nonce {
                                        //respond with our nonce
                                        let mut buff = [16,0,0,0,0x37,0x29,0x2d,0x12,0x86,0x5c,0x7b,0x4c,0x81,0xf5,0xe,0x5,0x1,0x78,0x80,0xc2];
                                        //LittleEndian::write_u32(&mut buff, 16);
    
                                        info!("P2P >> Nonce");
                                        write.write(&buff).await;
                                        do_nonce = false;
    
                                    } else if command.is_data() {
                                        let p2p_data = command.data.unwrap();
    
                                        if real_user.is_none() {
                                            if let Some(payload) = p2p_data.get_payload() {
                                                if let Ok(slp) = payload.get_payload_as_slp() {
                                                    real_user = slp.get_sender();
                                                    proxy_user = slp.get_receiver();
                                                }
                                            }
                                        }
                                        p2p_session.on_message_received(PendingPacket::new(p2p_data, real_user.as_ref().unwrap_or(&MSNUser::default()).clone(), proxy_user.as_ref().unwrap_or(&MSNUser::default()).clone()));
                                        
                                    } else {
                                        info!("unkown command {:?}", &command);
                                    }
                            }              
                            buffer = [0u8; 4096];
                        },
                        command_to_send = rx.recv() => {
                            if let Ok(msg) = command_to_send {
                                info!("P2P >> {:?}", &msg);
                            
                                let bytes_to_send = msg.as_direct_p2p();
                                let line = unsafe {from_utf8_unchecked(&bytes_to_send.as_slice())};
                            //    info!("P2P >> STRING: {}", &line);
    
                                write.write_all(bytes_to_send.as_slice()).await;
                            } else {
                                info!("P2P >> BAD COMMAND {:?}", &command_to_send);

                            }
                            
                           
                        }
                    }
                }
                //cleanup
            }).await;
        }
    }



}

impl P2PServer {
    pub fn new(
        url: String,
        port: u32,
    ) -> P2PServer {
        return P2PServer {
            url,
            port,
        };
    }
}