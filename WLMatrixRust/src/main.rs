mod sockets;
use sockets::tcpserver::*;
use tokio::join;

#[tokio::main]
async fn main() {

    let notif_server = tokio::spawn(async {
        TCPServer::new("localhost".to_string(), 7777).listen().await;
    });

    let switchboard_server = tokio::spawn(async {
        TCPServer::new("localhost".to_string(), 7778).listen().await;
    });
    
    let _test = join!(notif_server, switchboard_server);
    println!("Goodbye!");
}
