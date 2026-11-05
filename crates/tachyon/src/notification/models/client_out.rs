use msnp::msnp::notification::command::command::NotificationServerCommand;
use msnp::msnp::notification::models::endpoint_data::EndpointData;
use msnp::shared::models::display_name::DisplayName;
use msnp::shared::models::msn_object::MsnObject;
use msnp::shared::models::network_id_email::NetworkIdEmail;
use msnp::shared::models::presence_status::PresenceStatus;

/// Toward-MSN vocabulary: plain data produced by pure decide functions and
/// rendered per dialect by the shell (§3.2). A variant is promoted to
/// semantic only on demonstrated difference (§3.2 escape-hatch rule);
/// everything else rides `Command(..)`.
pub(crate) enum ClientOut {
    /// A contact presence notification: exactly what today's ILN + UBX pair
    /// carries, no more.
    ContactPresence {
        user: NetworkIdEmail,
        status: PresenceStatus,
        display_name: DisplayName,
        avatar: Option<MsnObject>,
        endpoint_data: EndpointData,
    },
    /// Escape hatch: an already-built wire command, passed through untouched.
    // No D1 producer yet: the escape hatch ships as part of the vocabulary
    // (§3.2) and its first construction site arrives with the D2 migration.
    #[allow(dead_code)]
    Command(NotificationServerCommand),
}
