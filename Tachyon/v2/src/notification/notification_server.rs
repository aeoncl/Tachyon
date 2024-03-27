use std::future::Future;

use matrix_sdk::ruma::events::key::verification::start;
use msnp::{msnp::{notification::command::command::NotificationCommand, raw_command_parser::{RawCommand, RawCommandParser}}, shared::command::command::SerializeMsnp};
use tokio::{io::{AsyncReadExt, AsyncWriteExt, BufReader}, net::{tcp::{OwnedWriteHalf, WriteHalf}, TcpListener, TcpStream}, signal, sync::{broadcast::{self, Receiver}, mpsc::{self, Sender}}};
use anyhow::anyhow;
use std::mem;
pub struct NotificationServer;


impl NotificationServer {

    pub fn new() -> Self {
        Self
    }

    pub async fn listen(&self, ip_addr: &str, port: u32, global_kill_recv: broadcast::Receiver<()>) -> Result<(), anyhow::Error>{
        println!("TCP Server started...");

        let listener = TcpListener::bind(format!("{}:{}", ip_addr, port))
            .await.map_err(|e| anyhow!(e))?;

            loop {
                let mut global_kill_recv = global_kill_recv.resubscribe();

                tokio::select! {
                    accepted = listener.accept() => {
                        let (socket, _addr)  = accepted.map_err(|e| anyhow!(e))?;
                        let _result = tokio::spawn(async move {handle_client(socket, global_kill_recv.resubscribe()).await}).await;
                    }
                    global_kill = global_kill_recv.recv() => {
                        if let Err(err) = global_kill {
                            println!("Unable to listen for global kill: {}", err);
                        }
                        break;
                    }
                }               
            }   

            println!("TCP Server gracefull shtudown...");
            Ok(())
    }

    
}

async fn handle_client(socket: TcpStream, mut global_kill_recv : broadcast::Receiver<()>) -> Result<(), anyhow::Error> {    

    println!("Client connected...");

    let (read, write) = socket.into_split();
    
    let (client_kill_snd, client_kill_recv) = broadcast::channel::<()>(1);

    let command_sender = start_write_task(write, client_kill_recv.resubscribe());


    let mut parser = RawCommandParser::new();
    let mut reader = BufReader::new(read);
    let mut buffer= [0u8; 2048];

    loop {

        tokio::select! {
            bytes_read = reader.read(&mut buffer) => {
                match bytes_read {
                    Err(e) => {
                        //TODO add log
                        eprintln!("SOCKET ERROR");
                        break;
                    },
                    Ok(bytes_read) => {

                        if bytes_read == 0 {
                            break;
                        }

                        let data = &buffer[..bytes_read];
                        

                        let commands = parser.parse_message(data);
                        if let Err(e) = commands {
                            println!("Unable to parse commands: {}", e);
                        }
                    }
                }
            },
            global_kill = global_kill_recv.recv() => {
                if let Err(err) = global_kill {
                    println!("Unable to listen for global kill: {}", err);
                }
                break;
            }
        }
    }

    client_kill_snd.send(())?;

    println!("Client gracefully shutdown...");
    Ok(())

}

fn handle_command(raw_command: RawCommand) {
    
}

fn start_write_task(mut write: OwnedWriteHalf, mut kill_recv: Receiver<()>) -> Sender<NotificationCommand> {

    println!("Socket write task started...");


    let (sender, mut receiver) = mpsc::channel::<NotificationCommand>(300);
    let _result = tokio::spawn(async move {
        loop {

            tokio::select! {
                command = receiver.recv() => {
                    if let Some(command) = command {
                        let result = write.write_all(&command.serialize_msnp()).await;
                    }
                },
                _kill_signal = kill_recv.recv() => {
                    break;
                }
            }
        }

        println!("Socket write task gracefully shutdown...");
    } );



    sender

}
