use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// The client's major and minor version, the part WLM spells the same everywhere: `CVR`
/// says `14.0.8117.0416` and the RST2 `User-Agent` says `14.0.8117.416`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientVersion {
    major: u32,
    minor: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ClientVersionError(String);

impl Display for ClientVersionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "not a client version: {}", self.0)
    }
}

impl std::error::Error for ClientVersionError {}

impl ClientVersion {
    /// The `App <exe>, <version>, {<guid>}` tail of the PPCRL `User-Agent`.
    pub fn from_user_agent(user_agent: &str) -> Option<Self> {
        let (_, app) = user_agent.split_once("App ")?;
        let mut fields = app.split(',').map(str::trim);
        let _exe = fields.next()?;
        fields.next()?.parse().ok()
    }
}

impl FromStr for ClientVersion {
    type Err = ClientVersionError;

    fn from_str(version: &str) -> Result<Self, Self::Err> {
        let mut numbers = version.split('.').map(|part| part.parse::<u32>());
        match (numbers.next(), numbers.next()) {
            (Some(Ok(major)), Some(Ok(minor))) => Ok(Self { major, minor }),
            _ => Err(ClientVersionError(version.to_owned())),
        }
    }
}

impl Display for ClientVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const USER_AGENT: &str = "Mozilla/4.0 (compatible; MSIE 6.0; Windows NT 6.2; WOW64; .NET4.0C; .NET4.0E; .NET CLR 2.0.50727; .NET CLR 3.0.30729; .NET CLR 3.5.30729; IDCRL 5.000.819.1; IDCRL-cfg 16.0.27832.0; App msnmsgr.exe, 14.0.8117.416, {7108E71A-9926-4FCB-BCC9-9A9D3F32E423})";

    #[test]
    fn the_cvr_and_the_user_agent_spell_the_same_version() {
        let from_cvr: ClientVersion = "14.0.8117.0416".parse().unwrap();
        let from_user_agent = ClientVersion::from_user_agent(USER_AGENT).unwrap();

        assert_eq!(from_cvr, from_user_agent);
        assert_eq!(from_cvr.to_string(), "14.0");
    }

    #[test]
    fn a_user_agent_without_the_app_tail_names_no_client() {
        assert_eq!(
            ClientVersion::from_user_agent("Mozilla/4.0 (compatible; MSIE 6.0)"),
            None
        );
    }

    #[test]
    fn a_version_needs_a_major_and_a_minor() {
        assert!("14".parse::<ClientVersion>().is_err());
        assert!("fourteen.zero".parse::<ClientVersion>().is_err());
    }
}
