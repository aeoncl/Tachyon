use crate::notification::models::client_out::ClientOut;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::xfr::XfrClient;
use msnp::msnp::notification::models::ip_address::IpAddress;
use msnp::shared::models::ticket_token::TicketToken;
use std::str::FromStr;

/// Pure decision (§3.1): answer a switchboard transfer request with the
/// local switchboard's address (127.0.0.1 + the configured port) and the
/// session's own ticket token as the CKI auth token. No IO — the port and
/// the token are plain data the shell reads off the config/SessionState;
/// the never-mutated `&mut SessionState` died with the Sender. The `?` on
/// the address parse is kept (it was the pre-D2 error path).
///
/// Dialect: deliberately free — MSNP21 has no switchboard, so a v21 client
/// must earn an XFR refusal and SDG/UUM over the NS writer instead. That
/// fork lands in §7 M1 and is owned by `deliver()` (§3.4: "the fork point
/// is named, owned by one function"), created in D4 with the classic arm
/// only — NOT by a dialect seam here, which would fork one difference
/// across two functions and burn a ≤6-budget seam slot ahead of its
/// consumer (§3.3 tripwire, §6 D). Anticipated, not shipped.
pub(crate) fn decide_xfr(command: XfrClient, token: &TicketToken, switchboard_port: u32) -> Result<ClientOut, anyhow::Error> {
    let xfr_response = command.get_response_for(IpAddress::from_str(format!("127.0.0.1:{}", switchboard_port).as_str())?, token.to_string());

    Ok(ClientOut::Command(NotificationServerCommand::XFR(xfr_response)))
}

#[cfg(test)]
mod tests {
    use msnp::msnp::notification::command::xfr::{ClientRequestType, XfrClient};
    use msnp::shared::models::ticket_token::TicketToken;
    use msnp::shared::traits::IntoBytes;

    use super::decide_xfr;
    use crate::notification::models::client_out::ClientOut;

    #[test]
    fn a_switchboard_request_is_answered_with_the_local_port_and_the_session_token() {
        let table = vec![
            (12u128, TicketToken("4uth_t0k3n".to_string()), 11864u32, "XFR 12 SB 127.0.0.1:11864 CKI t=4uth_t0k3n\r\n"),
            (13, TicketToken(String::new()), 1864, "XFR 13 SB 127.0.0.1:1864 CKI t=\r\n"),
        ];

        for (tr_id, token, port, expected_bytes) in table {
            let command = XfrClient::new(tr_id, ClientRequestType::Switchboard);

            let ClientOut::Command(out) = decide_xfr(command, &token, port).unwrap() else {
                panic!("xfr decides a bare wire command");
            };
            assert_eq!(String::from_utf8(out.into_bytes()).unwrap(), expected_bytes);
        }
    }
}
