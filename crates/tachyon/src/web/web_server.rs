use std::str::from_utf8;
use std::sync::Arc;

use anyhow::anyhow;
use axum::{middleware, Router};
use axum::body::Body;
use axum::extract::{FromRequest, Request};
use axum::http::{StatusCode, Uri};
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::{get, post};
use log::{debug, error, info, Level, log_enabled, warn};
use tokio::net::TcpListener;
use tokio::sync::broadcast::Receiver;
use http_body_util::BodyExt;

use crate::notification::client_store::ClientStoreFacade;
use crate::web::soap::ab_service::ab_service::address_book_service;
use crate::web::soap::rsi::rsi::rsi;
use crate::web::soap::sharing_service::sharing_service::sharing_service;

use crate::web::soap::rst2::rst2_handler;
use crate::web::soap::storage_service::storage_service::storage_service;
use crate::web::web_endpoints::{firewall_test, get_banner_ads, get_msgr_config, get_text_ad, ppcrlcheck, ppcrlconfigsrf, sha1auth, wlidsvcconfig};

pub struct WebServer;


impl WebServer {
    pub async fn listen(ip_addr: &str, port: u32, global_kill_recv: Receiver<()>, client_store_facade: ClientStoreFacade) -> Result<(), anyhow::Error> {
        info!("Web Server started...");


        let state = client_store_facade;

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
            //SOAP
            .route("/abservice/abservice.asmx", post(address_book_service))
            .route("/abservice/SharingService.asmx", post(sharing_service))
            .route("/storageservice/SchematizedStore.asmx", post(storage_service))
            .route("/rsi/rsi.asmx", post(rsi))
            .with_state(state)
            .layer(middleware::from_fn(my_middleware))
            .fallback(fallback);

        let listener = TcpListener::bind(format!("{}:{}", ip_addr, port))
            .await.map_err(|e| anyhow!(e))?;

        axum::serve(listener, app).with_graceful_shutdown(shutdown_signals(global_kill_recv)).await.map_err(|e| e.into())
    }
}


async fn my_middleware(
    request: Request,
    next: Next,
) -> Response {

    let mut request = request;

    info!("WEB << {} - SOAPAction: {:?}", request.uri(), request.headers().get("SOAPAction"));

    if log_enabled!(Level::Debug) {
        let (parts, body) = request.into_parts();
        let bytes = body.collect().await.unwrap().to_bytes();
        debug!("{:?}", from_utf8(&bytes).unwrap());
        
        request = Request::from_parts(parts, Body::from(bytes))
    }

    let mut response = next.run(request).await;

    info!("WEB >> {}", &response.status());

    if log_enabled!(Level::Debug) {
        let (parts, body) = response.into_parts();
        let bytes = body.collect().await.unwrap().to_bytes();

        debug!("{:?}", from_utf8(&bytes).unwrap());

        response = Response::from_parts(parts, Body::from(bytes))
    }


    response
}

async fn fallback(request: Request) -> (StatusCode, String) {

    let uri = request.uri().to_string();
    warn!("WEB << Unknown url called: {} {}", request.method(), &uri);
    debug!("Body: {}", String::from_request(request, &()).await.unwrap());
    (StatusCode::NOT_FOUND, format!("No route for {}", &uri))
}

async fn shutdown_signals(mut global_kill_recv: Receiver<()>) {
    let _result = global_kill_recv.recv().await;
    info!("Web Server gracefully shutdown...")
}