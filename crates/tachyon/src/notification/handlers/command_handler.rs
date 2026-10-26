use std::sync::Arc;
use crate::notification::handlers::ready::handle_ready;
use crate::notification::handlers::{auth, negotiation};
use crate::notification::models::connection_phase::ConnectionPhase;
use crate::notification::models::local_client_data::LocalClientData;
use crate::tachyon::config::tachyon_config::TachyonConfig;
use crate::tachyon::global_state::{GlobalState, PendingLogin};
use anyhow::anyhow;
use log::warn;
use msnp::msnp::notification::command::command::{NotificationClientCommand, NotificationServerCommand};
use tokio::sync::mpsc::Sender;

pub(crate) async fn handle_command(command: NotificationClientCommand, command_sender: Sender<NotificationServerCommand>, global_state: &GlobalState, local_client_data: &mut LocalClientData, config: &TachyonConfig) -> Result<(), anyhow::Error> {

    let _command_result = match &local_client_data.phase {
        ConnectionPhase::Negotiating => {
            negotiation::handle_negotiation(command, command_sender, local_client_data).await
        },
        ConnectionPhase::Authenticating  => {
            auth::handle_auth(command, command_sender, &global_state, local_client_data, config).await
        },
        ConnectionPhase::AwaitingLogin => {
            if let Some(tachyon_client) = global_state.get_clients(local_client_data.token.as_str()) {
                local_client_data.matrix_client = Some(tachyon_client.matrix_client());
                local_client_data.tachyon_client = Some(tachyon_client.clone());
                local_client_data.phase = ConnectionPhase::Ready;
                handle_ready(command, command_sender, tachyon_client, local_client_data.matrix_client.clone().expect("just stored"), local_client_data, config).await
            } else {
                let pending_login = global_state.pending_login(local_client_data.token.as_str()).expect("Pending login to be here");
                handle_awaiting_login(command, command_sender, pending_login).await
            }
        },
        ConnectionPhase::Ready => {
            let matrix_client = local_client_data.matrix_client.as_ref().ok_or(anyhow!("Matrix Client should be here by now"))?.clone();
            let tachyon_client = local_client_data.tachyon_client.as_ref().ok_or(anyhow!("Tachyon Client should be here by now"))?.clone();
            handle_ready(command, command_sender, tachyon_client, matrix_client, local_client_data, config).await
        }
    };

    Ok(())

}

// the backend is still logging in: keep the client alive with pings and list acknowledgements
async fn handle_awaiting_login(command: NotificationClientCommand, command_sender: Sender<NotificationServerCommand>, pending_login: Arc<PendingLogin>) -> Result<(), anyhow::Error> {
    match command {
        NotificationClientCommand::PNG => {
            command_sender.send(NotificationServerCommand::QNG(120)).await?;
            Ok(())
        }
        NotificationClientCommand::ADL(command) => {

            let contacts = command.payload.get_contacts()?;

            {
                let mut contact_list = pending_login.contact_list.lock().unwrap();
                contact_list.add_contacts(contacts.clone(), command.payload.is_initial());
            }

            command_sender.send(NotificationServerCommand::OK(command.get_ok_response("ADL"))).await?;
            Ok(())
        }
        NotificationClientCommand::RML(_) | NotificationClientCommand::UUX(_) | NotificationClientCommand::PRP(_) | NotificationClientCommand::OUT | NotificationClientCommand::VER(_) | NotificationClientCommand::CVR(_) => Ok(()),
        other => {
            warn!("MSNP|NS: command {} needs the backend; still authenticating", other);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::notification::handlers::command_handler::handle_command;
    use crate::notification::models::connection_phase::ConnectionPhase;
    use crate::notification::models::local_client_data::LocalClientData;
    use crate::tachyon::config::secret_encryptor::SecretEncryptor;
    use crate::tachyon::global_state::GlobalState;
    use msnp::msnp::notification::command::command::{NotificationClientCommand, NotificationServerCommand};
    use msnp::msnp::notification::command::cvr::CvrClient;
    use msnp::msnp::notification::command::usr::{AuthOperationTypeClient, AuthPolicy, OperationTypeServer, SsoPhaseClient, SsoPhaseServer, UsrClient};
    use msnp::msnp::notification::command::ver::VerClient;
    use msnp::msnp::notification::models::msnp_version::MsnpVersion::{MSNP17, MSNP18};
    use msnp::msnp::raw_command_parser::RawCommand;
    use msnp::shared::models::email_address::EmailAddress;
    use msnp::shared::traits::TryFromRawCommand;
    use std::str::FromStr;
    use std::sync::Arc;
    use matrix_sdk::{async_trait, Client};
    use matrix_sdk::ruma::UserId;
    use matrix_sdk::test_utils::mocks::{MatrixMock, MatrixMockServer};
    use crate::matrix::services::login::{MatrixLoginService, MatrixLoginServiceImpl};
    use crate::tachyon::error::TachyonError;
    use tachyon_sqlite_store::SqliteCredentialsStore;

    const TEST_SECRET: [u8; 32] = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32];

    fn test_credentials_store() -> Arc<SqliteCredentialsStore> {
        let dir = std::env::temp_dir().join(format!("tachyon-cmdh-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        Arc::new(SqliteCredentialsStore::open(&dir.join("credentials.db")).expect("store opens"))
    }

    struct TestMatrixLoginService {
        client: Client,
    }

    impl TestMatrixLoginService {
        pub fn new(client: Client) -> Self {
            Self { client }
        }
    }

    #[async_trait]
    impl MatrixLoginService for TestMatrixLoginService {
        async fn restore_saved_session(&self, _credentials: &tachyon_sqlite_store::Credentials, _homeserver_url: Option<&tachyon_sqlite_store::HomeserverUrl>, _disable_ssl: bool) -> Result<Client, TachyonError> {
            Ok(self.client.clone())
        }

        async fn login_with_password(&self, _matrix_id: &UserId, _password: &str, _device_id: Option<&tachyon_sqlite_store::DeviceId>, _homeserver_url: Option<&tachyon_sqlite_store::HomeserverUrl>, _disable_ssl: bool) -> Result<Client, TachyonError> {
            Ok(self.client.clone())
        }
    }


    #[tokio::test]
    async fn handshake_happy_flow_test() {

        let (snd, mut rcv) = tokio::sync::mpsc::channel::<NotificationServerCommand>(10);

        let state = GlobalState::new(Default::default(), SecretEncryptor::new(&TEST_SECRET).unwrap(), Box::new(MatrixLoginServiceImpl::new()), test_credentials_store());

        let (kill_snd, kill_recv) = tokio::sync::broadcast::channel::<()>(1);
        let mut local_client_data = LocalClientData::new(kill_snd.clone(), kill_recv.resubscribe());

        let ver = NotificationClientCommand::VER(VerClient::new(1, MSNP18, MSNP17));
        handle_command(ver, snd.clone(), &state, &mut local_client_data, state.get_config()).await.unwrap();

        let cvr = NotificationClientCommand::CVR(CvrClient::try_from_raw(RawCommand::from_str("CVR 2 0x0409 winnt 6.2.0 i386 MSNMSGR 14.0.8117.0416 msmsgs aeontest3@shlasouf.local").unwrap()).unwrap());
        handle_command(cvr, snd, &state, &mut local_client_data, state.get_config()).await.unwrap();

        let ver_respo = rcv.recv().await.unwrap();

        assert!(matches!(ver_respo, NotificationServerCommand::VER(_)));
        let NotificationServerCommand::VER(ver_resp) = ver_respo else {
            panic!("Expected VER response, got {}", &ver_respo);
        };

        assert_eq!(ver_resp.agreed_version, MSNP18);

        let cvr_respo = rcv.recv().await.unwrap();

        assert!(matches!(cvr_respo, NotificationServerCommand::CVR(_)));
        let NotificationServerCommand::CVR(cvr_resp) = cvr_respo else {
            panic!("Expected CVR response, got {}", &cvr_respo);
        };

        assert_eq!(local_client_data.phase, ConnectionPhase::Authenticating);
    }


    #[tokio::test]
    async fn auth_i_test() {

        let matrix_mock_server = MatrixMockServer::new().await;

        let client = matrix_mock_server.client_builder().build().await;
        let matrix_login_service = TestMatrixLoginService::new(client);


        let (snd, mut rcv) = tokio::sync::mpsc::channel::<NotificationServerCommand>(10);

        let state = GlobalState::new(Default::default(), SecretEncryptor::new(&TEST_SECRET).unwrap(), Box::new(matrix_login_service), test_credentials_store());

        let (kill_snd, kill_recv) = tokio::sync::broadcast::channel::<()>(1);
        let mut local_client_data = LocalClientData::new(kill_snd.clone(), kill_recv.resubscribe());
        local_client_data.phase = ConnectionPhase::Authenticating;

        let usr_i = NotificationClientCommand::USR(UsrClient {
            tr_id: 1,
            auth_type: AuthOperationTypeClient::Sso(SsoPhaseClient::I { email_addr: EmailAddress::from_str("aeon@test.com").unwrap() }),
        });

        handle_command(usr_i, snd, &state, &mut local_client_data, state.get_config()).await.unwrap();

        let usr_respo = rcv.recv().await.unwrap();
        assert!(matches!(usr_respo, NotificationServerCommand::USR(_)));
        let NotificationServerCommand::USR(usr_resp) = usr_respo else {
            panic!("Expected USR response, got {}", &usr_respo);
        };

        assert_eq!(usr_resp.tr_id, 1);
        assert!(matches!(usr_resp.auth_type, OperationTypeServer::Sso(SsoPhaseServer::S { .. })));

        let OperationTypeServer::Sso(SsoPhaseServer::S { policy, nonce }) = usr_resp.auth_type else {
            panic!("Expected Sso Phase Server, got {}", &usr_resp.auth_type);
        };

        assert!(matches!(policy, AuthPolicy::MbiKeyOld));

        let gcf_respo = rcv.recv().await.unwrap();
        assert!(matches!(gcf_respo, NotificationServerCommand::RAW(_)));

        let NotificationServerCommand::RAW(raw_resp) = gcf_respo else {
            panic!("Expected RAW response, got {}", &gcf_respo);
        };

        assert_eq!(raw_resp.get_operand(), "GCF");
    }


}
