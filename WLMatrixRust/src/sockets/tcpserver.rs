use tokio::{net::TcpListener, io::{AsyncBufReadExt, AsyncWriteExt, BufReader}};
use super::msnp_command::{MSNPCommandParser};

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
                loop {
        
                    let bytes_read = reader.read_line(&mut line).await.unwrap();
                    if bytes_read == 0 {
                        break;
                    }

                    let messages = MSNPCommandParser::parse_message(line.clone());
                    
                    println!("bytes read: {}, line: {}", bytes_read, line);
                    write.write_all(line.as_bytes()).await.unwrap();
                    line.clear();
                }
            });
        }
    }
}