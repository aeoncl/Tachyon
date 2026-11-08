use crate::msnp::notification::models::msnp_version::MsnpVersion;

/// What Tachyon can *serve* — the capability side of the wire/capability seam.
/// `MsnpVersion` is what clients may *say* on the wire; `Dialect` is negotiated
/// once per connection (VER), stored once in the session state, and matched
/// exhaustively at every seam: no `_` arms, no `#[non_exhaustive]` — adding a
/// variant here makes the compiler print the work list at every site that
/// needs a decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Dialect {
    #[default]
    Msnp18,
}

/// Per-dialect wire data, one record per `Dialect` variant. Fields move in
/// with their consumers, never ahead of them: negotiation consumes the CVR
/// strings and the switchboard chunker consumes the message-size cap (D5);
/// auth scheme / verb sets join only with a real second dialect.
pub struct DialectProfile {
    pub dialect: Dialect,
    pub cvr_version: &'static str,
    pub cvr_host: &'static str,
    /// The MSG body byte cap the switchboard chunker splits on — consumed
    /// by `SwitchboardHandle::receive_msg`, which stores it at construction.
    /// This record is its single source of truth: the pre-D5 file-local
    /// `MAX_MSG_BODY_SIZE` const in `switchboard_handle.rs` is retired.
    pub max_msg_body_size: usize,
}

impl Dialect {
    /// The profile record for this dialect.
    ///
    /// Seam site (negotiation / profile lookup, reached from the VER and CVR
    /// arms through `dialect_for`): MSNP21 serves SDG/UUM over the NS and has
    /// no switchboard (§7 M1); pre-MSNP18 clients need the browser-auth
    /// trigger (§7 M2). Both land as new arms with their own records.
    pub fn profile(self) -> DialectProfile {
        match self {
            Dialect::Msnp18 => DialectProfile {
                dialect: Dialect::Msnp18,
                cvr_version: "14.0.8117.0416",
                cvr_host: "localhost",
                max_msg_body_size: 1311,
            },
        }
    }
}

/// The single mapping from wire vocabulary to serve capability: one
/// exhaustive match, no registry. Versions without a profile earn a clean
/// OUT from negotiation instead of a parse error.
pub fn dialect_for(v: MsnpVersion) -> Option<Dialect> {
    match v {
        MsnpVersion::MSNP17 => None,
        MsnpVersion::MSNP18 => Some(Dialect::Msnp18),
        MsnpVersion::MSNP21 => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{dialect_for, Dialect};
    use crate::msnp::notification::models::msnp_version::MsnpVersion;

    /// The Msnp18 profile record is wire-contract data, pinned value-exact
    /// (D5): negotiation renders the CVR strings (the negotiation.rs fixture
    /// pins the rendered bytes) and the switchboard chunker splits on the
    /// cap — the e2e chunked flows and the msg_handler chunk table tests
    /// anchor the behavior. Changing any value here is a deliberate protocol
    /// change, named in the same commit (§2's fixture rule).
    #[test]
    fn the_msnp18_profile_pins_the_cvr_strings_and_the_chunker_cap() {
        let profile = Dialect::Msnp18.profile();
        assert_eq!(profile.dialect, Dialect::Msnp18);
        assert_eq!(profile.cvr_version, "14.0.8117.0416");
        assert_eq!(profile.cvr_host, "localhost");
        assert_eq!(profile.max_msg_body_size, 1311);
    }

    /// The wire-vocabulary -> serve-capability mapping is exhaustive over
    /// MsnpVersion: MSNP18 is the only served dialect; MSNP17 (WLM 2011's
    /// second candidate) and MSNP21 (parseable, not served — §7 M1) earn a
    /// clean None, which negotiation turns into OUT, never a parse error.
    #[test]
    fn dialect_for_maps_only_the_served_version() {
        assert_eq!(dialect_for(MsnpVersion::MSNP18), Some(Dialect::Msnp18));
        assert_eq!(dialect_for(MsnpVersion::MSNP17), None);
        assert_eq!(dialect_for(MsnpVersion::MSNP21), None);
    }
}
