use crate::notification::models::client_out::ClientOut;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::url::{UrlClient, UrlServer};

/// Pure decision (§3.1): answer a URL request with the Tachyon endpoints.
/// Passthrough — no real decision: the requested url_type is still ignored
/// (the pre-D2 handler's TODO), so every request earns the same response
/// and only the tr_id travels. No IO; the pre-D2 handler's SessionState and
/// TachyonClient parameters were never read and are gone.
pub(crate) fn decide_url(command: UrlClient) -> ClientOut {

    //TODO: handle url types

    ClientOut::Command(NotificationServerCommand::URL(
        UrlServer::new(command.tr_id, "https://hotmail.live.com/tachyon".to_string(), "https://login.live.com/RST2.srf".to_string(),0)
    ))
}

#[cfg(test)]
mod tests {
    use msnp::msnp::notification::command::command::NotificationServerCommand;
    use msnp::msnp::notification::command::url::{UrlClient, UrlType};
    use msnp::shared::traits::IntoBytes;

    use super::decide_url;
    use crate::notification::models::client_out::ClientOut;

    #[test]
    fn every_url_request_is_answered_with_the_tachyon_endpoints_regardless_of_type() {
        let table = vec![
            (14, UrlType::Inbox),
            (15, UrlType::Compose),
            (16, UrlType::Mobile),
        ];

        for (tr_id, url_type) in table {
            let ClientOut::Command(command) = decide_url(UrlClient { tr_id, url_type }) else {
                panic!("url decides a bare wire command");
            };
            let NotificationServerCommand::URL(url) = command else {
                panic!("expected the URL response");
            };

            assert_eq!(
                String::from_utf8(url.into_bytes()).unwrap(),
                format!("URL {tr_id} https://hotmail.live.com/tachyon https://login.live.com/RST2.srf 0\r\n")
            );
        }
    }
}
