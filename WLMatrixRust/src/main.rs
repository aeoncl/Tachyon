mod sockets;
mod generated;

use sockets::tcpserver::*;
use tokio::join;
use generated::msnab_sharingservice::messages::Abheader;

#[tokio::main]
async fn main() {
    


   // let out_dir = "./src/generated".to_string();
   // println!("{}", out_dir);
   // let _s = savon::gen::gen_write("../assets/wsdl/absharingservice/msnab_sharingservice.wsdl", &out_dir).unwrap();


    let notif_server = tokio::spawn(async {
        TCPServer::new("127.0.0.1".to_string(), 1863).listen().await;
    });

    let switchboard_server = tokio::spawn(async {
        TCPServer::new("127.0.0.1".to_string(), 1864).listen().await;
    });
    
    let _test = join!(notif_server, switchboard_server);
    println!("Goodbye!");
}
