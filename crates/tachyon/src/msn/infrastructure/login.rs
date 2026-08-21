use msnp::shared::models::ticket_token::TicketToken;
use crate::msn::domain::login::AuthService;
use crate::tachyon::application::login::TachyonLoginService;

pub struct AuthServiceImpl {
    tachyon_login_service: Box<dyn TachyonLoginService>
}

impl AuthServiceImpl {

    pub fn new(tachyon_login_service: Box<dyn TachyonLoginService>) -> Self {
        Self {
            tachyon_login_service,
        }
    }

}

impl AuthService for AuthServiceImpl {
    fn login_with_token(&self, token: TicketToken) {
        self.tachyon_login_service.restore_session();
    }
}