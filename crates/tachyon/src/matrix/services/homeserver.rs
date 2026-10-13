use std::time::Duration;

use matrix_sdk::reqwest::Client as HttpClient;
use matrix_sdk::Client;
use tachyon_sqlite_store::HomeserverUrl;

#[derive(Clone, Debug)]
pub struct ResolvedHomeserver {
    pub url: HomeserverUrl,
    pub supports_oauth: bool,
}

#[derive(Clone, Debug)]
pub struct ResolutionError {
    pub message: String,
    pub hint: Option<String>,
}

impl ResolutionError {
    pub fn unreachable(input: &str) -> Self {
        Self { message: format!("No homeserver answered at \"{input}\"."), hint: None }
    }

    pub fn plain_http_disabled(url: &str) -> Self {
        Self {
            message: "Plain-http homeservers are disabled.".to_string(),
            hint: Some(format!(
                "{url} only answers over plain http. To allow it, set \
                 allow_plain_http = true under [matrix] in config.ini and \
                 restart Tachyon."
            )),
        }
    }
}

impl std::fmt::Display for ResolutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

#[derive(Clone)]
pub struct HomeserverResolver {
    disable_ssl: bool,
}

impl HomeserverResolver {
    pub fn new(disable_ssl: bool) -> Self {
        Self { disable_ssl }
    }

    fn http_client(&self) -> Result<HttpClient, String> {
        let mut builder = HttpClient::builder().timeout(Duration::from_secs(5));
        if self.disable_ssl {
            builder = builder.danger_accept_invalid_certs(true);
        }
        builder.build().map_err(|error| format!("the probe client failed to build: {error}"))
    }

    async fn answers_versions(&self, http: &HttpClient, base: &HomeserverUrl) -> bool {
        let url = format!("{}/_matrix/client/versions", base.as_str());
        matches!(http.get(url).send().await, Ok(response) if response.status().is_success())
    }

    async fn well_known_base(&self, http: &HttpClient, domain: &str) -> Option<HomeserverUrl> {
        let url = format!("https://{domain}/.well-known/matrix/client");
        let body = http.get(url).send().await.ok()?.text().await.ok()?;
        well_known_base_from_json(&body)
    }

    async fn supports_oauth(&self, base: &HomeserverUrl) -> bool {
        let mut builder = Client::builder().homeserver_url(base.as_str());
        if self.disable_ssl {
            builder = builder.disable_ssl_verification();
        }
        match builder.build().await {
            Ok(client) => match client.oauth().server_metadata().await {
                Ok(_) => true,
                Err(error) if error.is_not_supported() => false,
                Err(error) => {
                    log::warn!("OAuth discovery failed for {base}, assuming it is unsupported: {error}");
                    false
                }
            },
            Err(error) => {
                log::warn!("probe client for {base} failed to build: {error}");
                false
            }
        }
    }

    pub async fn resolve(&self, input: &str, allow_plain_http: bool) -> Result<ResolvedHomeserver, ResolutionError> {
        let http = self.http_client().map_err(|message| ResolutionError { message, hint: None })?;
        let trimmed = input.trim().trim_end_matches('/');

        enum Candidate {
            WellKnown(String),
            Url(String),
        }

        let mut candidates: Vec<Candidate> = Vec::new();
        let mut skipped_by_policy: Option<HomeserverUrl> = None;
        if let Ok(url) = HomeserverUrl::parse(trimmed) {
            if url.as_str().starts_with("http://") && !allow_plain_http {
                skipped_by_policy = Some(url);
            } else {
                candidates.push(Candidate::Url(url.as_str().to_string()));
            }
        } else {
            candidates.push(Candidate::WellKnown(trimmed.to_string()));
            candidates.push(Candidate::Url(format!("https://{trimmed}")));
            if allow_plain_http {
                candidates.push(Candidate::Url(format!("http://{trimmed}")));
            } else {
                skipped_by_policy = HomeserverUrl::parse(&format!("http://{trimmed}")).ok();
            }
        }

        for candidate in candidates {
            let base = match candidate {
                Candidate::WellKnown(domain) => self.well_known_base(&http, &domain).await,
                Candidate::Url(url) => HomeserverUrl::parse(&url).ok(),
            };
            let Some(base) = base else { continue };
            if base.as_str().starts_with("http://") && !allow_plain_http {
                skipped_by_policy = Some(base);
                continue;
            }
            if self.answers_versions(&http, &base).await {
                let supports_oauth = self.supports_oauth(&base).await;
                return Ok(ResolvedHomeserver { url: base, supports_oauth });
            }
        }

        // A policy-skipped candidate is probed anyway, so the honest error is the policy one, not "unreachable".
        if let Some(base) = skipped_by_policy {
            if self.answers_versions(&http, &base).await {
                return Err(ResolutionError::plain_http_disabled(base.as_str()));
            }
        }

        Err(ResolutionError::unreachable(input))
    }
}

fn well_known_base_from_json(body: &str) -> Option<HomeserverUrl> {
    let well_known: serde_json::Value = serde_json::from_str(body).ok()?;
    let base = well_known.get("m.homeserver")?.get("base_url")?.as_str()?;
    HomeserverUrl::parse(base).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn well_known_json_yields_the_advertised_base_url() {
        let base = well_known_base_from_json(r#"{"m.homeserver": {"base_url": "https://matrix.example.org"}}"#);
        assert_eq!(base.map(|b| b.as_str().to_string()), Some("https://matrix.example.org".to_string()));
        assert!(well_known_base_from_json("not json").is_none());
        assert!(well_known_base_from_json(r#"{"m.homeserver": {}}"#).is_none());
    }

    fn resolver() -> HomeserverResolver {
        HomeserverResolver::new(false)
    }

    fn versions_mock() -> Mock {
        Mock::given(method("GET"))
            .and(path("/_matrix/client/versions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "versions": ["v1.11"] })))
    }

    #[tokio::test]
    async fn resolves_a_full_url_and_reports_password_only_without_oauth() {
        let server = MockServer::start().await;
        versions_mock().mount(&server).await;

        let error = resolver().resolve(server.uri().as_str(), false).await.expect_err("plain http is disallowed");
        assert_eq!(error.message, "Plain-http homeservers are disabled.");
        let hint = error.hint.expect("the policy error carries the knob hint");
        assert!(hint.contains("allow_plain_http"));
        assert!(hint.contains(server.uri().as_str()));

        let resolved = resolver().resolve(server.uri().as_str(), true).await.expect("resolves");
        assert_eq!(resolved.url.as_str(), server.uri());
        assert!(!resolved.supports_oauth);
    }

    #[tokio::test]
    async fn https_domain_fallback_when_well_known_is_missing() {
        let server = MockServer::start().await;
        versions_mock().mount(&server).await;

        let domain = server.uri().trim_start_matches("http://").to_string();
        let error = resolver().resolve(&domain, false).await.expect_err("the http candidate is policy-blocked");
        assert_eq!(
            error.message, "Plain-http homeservers are disabled.",
            "a server that answers over http is a policy case, not unreachable"
        );

        let resolved = resolver().resolve(&domain, true).await.expect("resolves");
        assert_eq!(resolved.url.as_str(), server.uri());
    }

    #[tokio::test]
    async fn unreachable_input_is_an_error() {
        let result = resolver().resolve("https://127.0.0.1:1", false).await;
        let error = result.expect_err("nothing answers on port 1");
        assert_eq!(error.message, "No homeserver answered at \"https://127.0.0.1:1\".");
        assert!(error.hint.is_none());
    }

    #[tokio::test]
    async fn a_dead_http_candidate_stays_unreachable_not_policy() {
        let error = resolver().resolve("http://127.0.0.1:1", false).await.expect_err("nothing answers on port 1");
        assert_eq!(error.message, "No homeserver answered at \"http://127.0.0.1:1\".");
    }
}
