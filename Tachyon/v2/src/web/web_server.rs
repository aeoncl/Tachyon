use anyhow::anyhow;
use axum::extract::{FromRequest, Request};
use axum::http::{StatusCode, Uri};
use axum::Router;
use axum::routing::{post, get};
use log::{debug, info, warn};
use tokio::net::TcpListener;
use tokio::sync::broadcast::Receiver;
use crate::notification::client_store::ClientStoreFacade;
use crate::web::soap::rst2::{rst2_handler};
use crate::web::web_endpoints::{firewall_test, get_banner_ads, get_msgr_config, get_text_ad, ppcrlcheck, ppcrlconfigsrf, sha1auth, wlidsvcconfig};
pub struct WebServer;


impl WebServer {
    pub async fn listen(ip_addr: &str, port: u32, global_kill_recv: Receiver<()>, client_store_facade: ClientStoreFacade) -> Result<(), anyhow::Error> {
        info!("Web Server started...");



        let app = Router::new()
            .route("/", post(firewall_test))
            .route("/Config/MsgrConfig.asmx", get(get_msgr_config))
            .route("/ads/banner", get(get_banner_ads))
            .route("/ads/text", get(get_text_ad))
            .route("/ppsecure/sha1auth.srf", post(sha1auth))
            .route("/ppcrlconfig.srf", get(ppcrlconfigsrf))
            .route("/ppcrlconfig.bin", get(ppcrlconfigsrf))
            .route("/PPCRLconfig.srf", get(wlidsvcconfig))
            .route("/ppcrlcheck.srf", get(ppcrlcheck))
            .route("/RST2.srf", post(rst2_handler))
            .fallback(fallback);

        let listener = TcpListener::bind(format!("{}:{}", ip_addr, port))
            .await.map_err(|e| anyhow!(e))?;

        axum::serve(listener, app).with_graceful_shutdown(shutdown_signals(global_kill_recv)).await.map_err(|e| e.into())
    }
}

async fn fallback(request: Request) -> (StatusCode, String) {

    let uri = request.uri().to_string();
    warn!("WEB - Unknown url called: {} {}", request.method(), &uri);
    
    debug!("Body: {}", String::from_request(request, &()).await.unwrap());
    (StatusCode::NOT_FOUND, format!("No route for {}", &uri))
}

async fn shutdown_signals(mut global_kill_recv: Receiver<()>) {
    let _result = global_kill_recv.recv().await;
    info!("Web Server gracefully shutdown...")
}