use crate::notification::models::client_out::ClientOut;
use msnp::msnp::notification::command::command::NotificationServerCommand;

/// Pure decision (§3.1): the Ready-phase ping is answered with the QNG
/// keep-alive delay the client expects. Passthrough — no real decision.
/// The awaiting-login PNG path in command_handler.rs is deliberately a
/// separate, byte-frozen site (§8 D2) and does not share this function.
pub(crate) fn decide_png() -> ClientOut {
    ClientOut::Command(NotificationServerCommand::QNG(120))
}

#[cfg(test)]
mod tests {
    use msnp::msnp::notification::command::command::NotificationServerCommand;
    use msnp::shared::traits::IntoBytes;

    use super::decide_png;
    use crate::notification::models::client_out::ClientOut;

    #[test]
    fn a_ready_phase_ping_is_answered_with_the_qng_keep_alive_delay() {
        let ClientOut::Command(command) = decide_png() else {
            panic!("png decides a bare wire command");
        };

        let NotificationServerCommand::QNG(timeout) = &command else {
            panic!("expected QNG, got {}", &command);
        };
        assert_eq!(*timeout, 120);
        assert_eq!(String::from_utf8(command.into_bytes()).unwrap(), "QNG 120\r\n");
    }
}
