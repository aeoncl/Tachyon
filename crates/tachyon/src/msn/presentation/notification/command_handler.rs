use crate::msn::notification::presentation::ready_handler::handle_ready;
use crate::msn::notification::presentation::{authentication_handler, negotiation_handler};
use crate::msn::notification::models::connection_phase::ConnectionPhase;
use crate::msn::notification::models::local_client_data::LocalClientData;
use crate::tachyon::config::tachyon_config::TachyonConfig;
use crate::app_state::AppState;
use anyhow::anyhow;
use msnp::msnp::notification::command::command::{NotificationClientCommand, NotificationServerCommand};
use tokio::sync::mpsc::Sender;
use crate::msn::domain::msn_bridge::{MsnpBridge, MsnpBridgeStatus};

pub(crate) async fn handle_command(command: NotificationClientCommand, command_sender: Sender<NotificationServerCommand>, global_state: &AppState, local_client_data: &mut LocalClientData, config: &TachyonConfig, bridge: MsnpBridge) -> Result<(), anyhow::Error> {

    let _command_result = match &bridge.status()? {
        MsnpBridgeStatus::Negotiating => {
            negotiation_handler::handle_negotiation(command, command_sender, local_client_data, bridge).await
        },
        MsnpBridgeStatus::Authenticating  => {
            authentication_handler::handle_auth(command, command_sender, &global_state, local_client_data, config, global_state.matrix_login_service()).await
        },
        MsnpBridgeStatus::Ready => {
            let matrix_client = local_client_data.matrix_client.as_ref().ok_or(anyhow!("Matrix Client should be here by now"))?.clone();
            let tachyon_client = local_client_data.tachyon_client.as_ref().ok_or(anyhow!("Tachyon Client should be here by now"))?.clone();
            handle_ready(command, command_sender, tachyon_client, matrix_client, local_client_data, config).await
        }
    };

    Ok(())

}

#[cfg(test)]
mod tests {
    use crate::msn::notification::presentation::command_handler::handle_command;
    use crate::msn::notification::models::connection_phase::ConnectionPhase;
    use crate::msn::notification::models::local_client_data::LocalClientData;
    use crate::tachyon::config::secret_encryptor::SecretEncryptor;
    use crate::app_state::AppState;
    use msnp::msnp::notification::command::command::{NotificationClientCommand, NotificationServerCommand};
    use msnp::msnp::notification::command::cvr::CvrClient;
    use msnp::msnp::notification::command::usr::{AuthOperationTypeClient, AuthPolicy, OperationTypeServer, SsoPhaseClient, SsoPhaseServer, UsrClient};
    use msnp::msnp::notification::command::ver::VerClient;
    use msnp::msnp::notification::models::msnp_version::MsnpVersion::{MSNP17, MSNP18};
    use msnp::msnp::raw_command_parser::RawCommand;
    use msnp::shared::models::email_address::EmailAddress;
    use msnp::shared::traits::TryFromRawCommand;
    use std::str::FromStr;
    use matrix_sdk::{async_trait, Client};
    use matrix_sdk::ruma::UserId;
    use matrix_sdk::test_utils::mocks::{MatrixMock, MatrixMockServer};
    use crate::matrix::application::login::{AccessToken, MatrixLoginService, MatrixLoginServiceImpl};
    use crate::tachyon::error::TachyonError;

    const TEST_SECRET: [u8; 32] = [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32];

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
        async fn login_with_token(&self, user_id: &UserId, token: &str, disable_ssl: bool) -> Result<Client, TachyonError> {
            self.client.matrix_auth().login_token(token).await;
            Ok(self.client.clone())
        }

        async fn login_with_password(&self, matrix_id: &UserId, password: &str, disable_ssl: bool) -> Result<(AccessToken, Client), TachyonError> {
            let test = self.client.matrix_auth().login_username(matrix_id.to_string(), password).await.unwrap();
            Ok((test.access_token, self.client.clone()))
        }
    }


    #[tokio::test]
    async fn handshake_happy_flow_test() {

        let (snd, mut rcv) = tokio::sync::mpsc::channel::<NotificationServerCommand>(10);

        let state = AppState::new(Default::default(), SecretEncryptor::new(&TEST_SECRET).unwrap(), Box::new(MatrixLoginServiceImpl::new()));

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

        assert!(matches!(local_client_data.phase, ConnectionPhase::Authenticating));
    }


    #[tokio::test]
    async fn auth_i_test() {

        let matrix_mock_server = MatrixMockServer::new().await;

        let client = matrix_mock_server.client_builder().build().await;
        let matrix_login_service = TestMatrixLoginService::new(client);


        let (snd, mut rcv) = tokio::sync::mpsc::channel::<NotificationServerCommand>(10);

        let state = AppState::new(Default::default(), SecretEncryptor::new(&TEST_SECRET).unwrap(), Box::new(matrix_login_service));

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