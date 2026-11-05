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
/// strings today; the message-size cap joins when its consumer migrates, and
/// auth scheme / verb sets join only with a real second dialect.
pub struct DialectProfile {
    pub dialect: Dialect,
    pub cvr_version: &'static str,
    pub cvr_host: &'static str,
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
