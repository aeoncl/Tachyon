use tokio::{join, select, signal, sync::broadcast::{self, Sender}};

use crate::notification::notification_server::NotificationServer;

mod notification;

#[tokio::main]
async fn main() {
    let (master_kill_signal,  kill_recv) = broadcast::channel::<()>(1);

    



    let notification_server = NotificationServer::new();
    let finished_notification_server = notification_server.listen("127.0.0.1", 1843, kill_recv);
      
    join!(finished_notification_server, listen_for_stop_signal(master_kill_signal));

    println!("Byebye, world!");
}

async fn listen_for_stop_signal(master_kill_signal: Sender<()>) {

    match signal::ctrl_c().await {
        Ok(()) => {},
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
            // we also shut down in case of error
        },
    }
    println!("Sending kill signals");
    master_kill_signal.send(());

}

