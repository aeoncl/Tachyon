mod sockets;
mod generated;
use actix_web::App;
use actix_web::HttpServer;
use sockets::tcpserver::*;
use sockets::webserver::*;
use tokio::join;


#[tokio::main]
async fn main() {
    
    let notif_server = tokio::spawn(async {
        TCPServer::new("127.0.0.1".to_string(), 1863).listen().await;
    });

    let switchboard_server = tokio::spawn(async {
        TCPServer::new("127.0.0.1".to_string(), 1864).listen().await;
    });

    HttpServer::new(|| App::new().service(greet).service(rst2))
    .bind(("127.0.0.1", 8080)).unwrap()
    .run().await;

    let _test = join!(notif_server, switchboard_server);
    println!("See you next time 👀!");
}
