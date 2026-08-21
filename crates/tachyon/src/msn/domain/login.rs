use msnp::shared::models::ticket_token::TicketToken;

pub trait AuthService {
    fn login_with_token(&self, token: TicketToken);

}