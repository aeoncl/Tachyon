use tokio::{net::TcpListener, io::{AsyncBufReadExt, AsyncWriteExt, BufReader}};
use super::msnp_command::{MSNPCommandParser, MSNPCommand};

pub struct TCPServer {
    url: String,
    port: u32
}

impl TCPServer {
    pub fn new(url: String, port: u32) -> TCPServer {
        return TCPServer{url: url, port: port};
    }

    pub async fn listen(&self){
        let listener = TcpListener::bind( format!("{}:{}", self.url, self.port)).await.unwrap();
        loop {
            let (mut socket, _addr) = listener.accept().await.unwrap();

            tokio::spawn(async move {
                let (read, mut write) = socket.split();
        
                let mut reader = BufReader::new(read);
                let mut line = String::new();
                let mut empty_payload : Option<MSNPCommand> = None;

                loop {
        
                    let bytes_read = reader.read_line(&mut line).await.unwrap();
                    if bytes_read == 0 {
                        break;
                    }

                    let mut messages;

                    match empty_payload {
                        Some(p) => {
                            messages = MSNPCommandParser::parse_payload_message(line.clone(), p);
                            empty_payload = None;
                        },
                        None => {
                            messages = MSNPCommandParser::parse_message(line.clone());
                            if !messages.is_empty() && !messages.last().unwrap().is_complete() {
                                empty_payload = Some(messages.pop().unwrap());
                            }
                        }
                    }

                   for message in messages {
                        println!("message: {}", message);
                   }
                    
                    write.write_all(line.as_bytes()).await.unwrap();
                    line.clear();
                }
            });
        }
    }
}