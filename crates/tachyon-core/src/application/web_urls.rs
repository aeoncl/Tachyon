use crate::domain::auth::TachyonToken;

/// The bridge's web pages as the user's browser reaches them. Every URL a backend or a user
/// is sent to is built from this one base, so the pages and the links agree by construction.
pub struct WebUrls {
    base: String,
}

impl WebUrls {
    /// `base` is the pages' root, for instance `http://127.0.0.1:11866/tachyon`.
    pub fn new(base: impl Into<String>) -> Self {
        let base: String = base.into();
        Self {
            base: base.trim_end_matches('/').to_owned(),
        }
    }

    /// Where the authorization server sends the browser back after OAuth.
    pub(crate) fn oauth_callback(&self) -> String {
        format!("{}/login/callback", self.base)
    }

    pub(crate) fn login_start(&self, flow_id: &str) -> String {
        format!(
            "{}/login/start?flow={}",
            self.base,
            urlencoding::encode(flow_id)
        )
    }

    pub(crate) fn confirm_device(&self, token: &TachyonToken) -> String {
        format!("{}/confirm_device?t={}", self.base, token.as_str())
    }
}
