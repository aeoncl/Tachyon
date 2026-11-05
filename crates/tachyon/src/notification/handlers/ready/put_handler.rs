use crate::notification::models::client_out::ClientOut;
use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::command::nfy::{NfyOperation, NfyServer};
use msnp::msnp::notification::command::put::PutClient;

/// Pure decision (§3.1): ack the PUT, then echo the payload back as an NFY
/// with the envelope sides swapped and its flags cleared. Both outputs, in
/// this exact order — the pre-D2 handler sent the ack first and the NFY
/// second, and the dispatcher sends in Vec order. No IO anywhere: the
/// payload mutation is plain data, so the whole verb is decide.
/// Passthrough — no real decision beyond the mechanical envelope swap.
pub(crate) fn decide_put(command: PutClient) -> Vec<ClientOut> {
    let ok = command.get_ok_command();

    let mut payload = command.payload;
    payload.envelope.swap_sides();
    payload.envelope.flags = None;

    vec![
        ClientOut::Command(NotificationServerCommand::PUT(ok)),
        ClientOut::Command(NotificationServerCommand::NFY(NfyServer {
            operation: NfyOperation::Put,
            payload
        })),
    ]
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use msnp::msnp::notification::command::command::NotificationServerCommand;
    use msnp::msnp::notification::command::nfy::NfyOperation;
    use msnp::msnp::notification::command::put::PutClient;
    use msnp::shared::models::network_id_email::NetworkIdEmail;
    use msnp::shared::payload::nfy::nfy_put_payload::RawNfyPayload;
    use msnp::shared::traits::IntoBytes;

    use super::decide_put;
    use crate::notification::models::client_out::ClientOut;

    #[test]
    fn a_put_is_acked_first_then_echoed_with_the_envelope_swapped_and_flags_cleared() {
        let from = NetworkIdEmail::from_str("1:aeon@test.com").unwrap();
        let to = NetworkIdEmail::from_str("1:peer@test.com").unwrap();

        let mut payload = RawNfyPayload::new_circle(from.clone(), to.clone());
        payload.envelope.flags = Some("0x2".to_string());

        let mut outs = decide_put(PutClient { tr_id: 3, payload }).into_iter();

        let ClientOut::Command(ack) = outs.next().expect("the ack first") else {
            panic!("expected a wire command ack");
        };
        assert!(matches!(ack, NotificationServerCommand::PUT(_)));
        assert_eq!(String::from_utf8(ack.into_bytes()).unwrap(), "PUT 3 OK 0\r\n");

        let ClientOut::Command(nfy) = outs.next().expect("the echo second") else {
            panic!("expected a wire command echo");
        };
        let NotificationServerCommand::NFY(nfy) = nfy else {
            panic!("expected the NFY second");
        };
        assert!(matches!(nfy.operation, NfyOperation::Put));
        assert_eq!(nfy.payload.envelope.from.email, to.email);
        assert_eq!(nfy.payload.envelope.to.email, from.email);
        assert!(nfy.payload.envelope.flags.is_none());

        assert!(outs.next().is_none());
    }
}
