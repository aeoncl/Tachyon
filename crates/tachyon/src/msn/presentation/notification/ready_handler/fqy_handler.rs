use crate::tachyon::client::tachyon_client::TachyonClient;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::fqy::{FqyClient, FqyServer};
use tokio::sync::mpsc::Sender;

pub async fn handle_fqy(command: FqyClient, tachyon_client: TachyonClient, command_sender: Sender<NotificationServerCommand>) -> Result<(), anyhow::Error> {

    let mut payload = command.payload;

//    let matrix_client = tachyon_client.matrix_client();


/*    for domain in payload.domains.as_mut_slice() {
        for contact in domain.contacts.as_mut_slice() {
            let email_address = EmailAddress::from_str(format!("{}@{}", &contact.email_part, &domain.domain).as_str()).unwrap();
            let user_id = email_address.to_owned_user_id();

            //Doesnt work because we don't have the room in joined yet. :(
            if let Some(found) = matrix_client.get_dm_room(&user_id) {
                contact.actual = Some(found.to_email_address().unwrap().to_string());
            }
        }
    }*/

    command_sender.send(NotificationServerCommand::FQY(FqyServer {
        tr_id: command.tr_id,
        payload,
    })).await?;

    Ok(())

}