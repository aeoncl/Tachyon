use std::fmt::{Display, Formatter};
use anyhow::anyhow;
use configparser::ini::Ini;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct TachyonConfig {

    pub notification_port: u32,
    pub switchboard_port: u32,
    pub http_port: u32,
    pub strict_ssl: bool,
    pub logs_enabled: bool,
    pub allow_plain_http: bool,
    pub homeserver_url: Option<String>,

}

impl Default for TachyonConfig {
    fn default() -> Self {

        Self {
            notification_port: 11863,
            switchboard_port: 11864,
            http_port: 11866,
            strict_ssl: true,
            logs_enabled: false,
            allow_plain_http: false,
            homeserver_url: None,
        }
    }
}

impl Display for TachyonConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut ini = Ini::new();
        ini.set("server", "notification_port", Some(self.notification_port.to_string()));
        ini.set("server", "switchboard_port", Some(self.switchboard_port.to_string()));
        ini.set("server", "http_port", Some(self.http_port.to_string()));
        ini.set("matrix", "strict_ssl", Some(self.strict_ssl.to_string()));
        ini.set("matrix", "allow_plain_http", Some(self.allow_plain_http.to_string()));
        if let Some(homeserver_url) = &self.homeserver_url {
            ini.set("matrix", "homeserver_url", Some(homeserver_url.clone()));
        }
        ini.set("tachyon_logs", "enabled", Some(self.logs_enabled.to_string()));
        write!(f, "{}", ini.writes())
    }
}

impl FromStr for TachyonConfig {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {

        let mut config = Ini::new();
        config.read(s.into()).map_err(|e| anyhow!("Couldn't parse config: {}", e))?;

        let notification_port : u32 = config.getuint("server", "notification_port").map_err(|e| anyhow!("Couldn't parse notification_port: {}", e))?.ok_or(anyhow!("notification_port is mandatory"))?.try_into().map_err(|e| anyhow!("notification_port is not a valid port: {}", e))?;
        let switchboard_port: u32 = config.getuint("server", "switchboard_port").map_err(|e| anyhow!("Couldn't parse switchboard_port: {}", e))?.ok_or(anyhow!("switchboard_port is mandatory"))?.try_into().map_err(|e| anyhow!("switchboard_port is not a valid port: {}", e))?;
        let http_port: u32 = config.getuint("server", "http_port").map_err(|e| anyhow!("Couldn't parse http_port: {}", e))?.ok_or(anyhow!("http_port is mandatory"))?.try_into().map_err(|e| anyhow!("http_port is not a valid port: {}", e))?;

        let strict_ssl = config.getbool("matrix", "strict_ssl").map_err(|e| anyhow!("Couldn't parse strict_ssl: {}", e))?.unwrap_or(true);

        let allow_plain_http = config.getbool("matrix", "allow_plain_http").map_err(|e| anyhow!("Couldn't parse allow_plain_http: {}", e))?.unwrap_or(false);

        let homeserver_url = config.get("matrix", "homeserver_url").map(|url| url.trim().to_string()).filter(|url| !url.is_empty());

        let logs_enabled = config.getbool("tachyon_logs", "enabled").map_err(|e| anyhow!("Couldn't parse strict_ssl: {}", e))?.unwrap_or(false);


        Ok(Self {
            notification_port,
            switchboard_port,
            http_port,
            strict_ssl,
            logs_enabled,
            allow_plain_http,
            homeserver_url,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use crate::tachyon::config::tachyon_config::TachyonConfig;

    #[test]
    fn deserialize_config() {
        let config = r#"[server]
notification_port = 1863
switchboard_port = 1864
http_port = 8080


[matrix]
strict_ssl = true


[zathras_logs]
enabled = true

[tachyon_logs]
enabled = true


"#;

    let config = TachyonConfig::from_str(config).expect("config to be valid");

        assert_eq!(config.notification_port, 1863);
        assert_eq!(config.switchboard_port, 1864);
        assert_eq!(config.http_port, 8080);
        assert_eq!(config.strict_ssl, true);
        assert_eq!(config.logs_enabled, true);
        assert_eq!(config.allow_plain_http, false);
        assert_eq!(config.homeserver_url, None);
    }

    #[test]
    fn deserialize_homeserver_url() {
        let config = r#"[server]
notification_port = 1863
switchboard_port = 1864
http_port = 8080

[matrix]
strict_ssl = false
allow_plain_http = true
homeserver_url = http://127.0.0.1:8008

[tachyon_logs]
enabled = false
"#;

        let config = TachyonConfig::from_str(config).expect("config to be valid");

        assert_eq!(config.homeserver_url.as_deref(), Some("http://127.0.0.1:8008"));
        assert!(config.allow_plain_http);
        assert!(!config.strict_ssl);
    }

    #[test]
    fn serialize_config() {

        let config = TachyonConfig {
            notification_port: 1863,
            switchboard_port: 1864,
            http_port: 8080,
            strict_ssl: false,
            logs_enabled: true,
            allow_plain_http: true,
            homeserver_url: Some("http://127.0.0.1:8008".to_string()),
        };

        let ser = config.to_string();
        let round_trip = TachyonConfig::from_str(&ser).expect("config to be valid");

        assert_eq!(round_trip.homeserver_url.as_deref(), Some("http://127.0.0.1:8008"));
        assert!(round_trip.allow_plain_http);

        assert!(ser.contains("[server]"));
        assert!(ser.contains("[matrix]"));
        assert!(ser.contains("[tachyon_logs]"));
        assert!(ser.contains("notification_port=1863"));
        assert!(ser.contains("switchboard_port=1864"));
        assert!(ser.contains("http_port=8080"));
        assert!(ser.contains("strict_ssl=false"));
        assert!(ser.contains("allow_plain_http=true"));
        assert!(ser.contains("homeserver_url=http://127.0.0.1:8008"));
        assert!(ser.contains("enabled=true"));

    }
}