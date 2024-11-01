use std::str::from_utf8;

use actix_web::{get, HttpRequest, HttpResponse, HttpResponseBuilder, post, web};
use http::StatusCode;
use lazy_static::lazy_static;
use log::info;
use matrix_sdk::ruma::api::client::error::{ErrorBody, ErrorKind};
use matrix_sdk::ruma::OwnedUserId;
use msnp::soap::passport::rst2::request::RST2RequestMessageSoapEnvelope;
use msnp::soap::passport::rst2::response::factory::RST2ResponseFactory;
use msnp::soap::traits::xml::TryFromXml;
use regex::Regex;
use urlencoding::decode;
use yaserde::de::from_str;
use yaserde::ser::to_string;

use crate::matrix::matrix_client::get_matrix_client_builder;
use crate::models::msn_user::MSNUser;
use crate::models::owned_user_id_traits::FromMsnAddr;
use crate::SETTINGS_LOCATOR;
use crate::utils::identifiers::get_matrix_device_id;

use super::error::WebError;

lazy_static! {
    pub static ref DEFAULT_CACHE_KEY: String = String::from("12r1:8nBBE6vX1J4uPKajtbem5XBIblimCwAhIziAeEAwYD0AMiaztryWvcZthkN9oX_pl2scBKXfKvRvuWKYdHUNuRkgiyV9rzcDpnDIDiM6vdcEB6d82wjjnL4TAFAjc5X8i-C94mNfQvujUk470P7fz9qbWfK6ANcEtygDb-oWsYVfEBrxl6geTUg9tGT7yCIsls7ECcLyqwsROuAbWCrued_VPKiUgSIvqG8gaA");
    pub static ref SHA1_REGEX: Regex = Regex::new(r"ru=([^&]*)&").unwrap();

}

lazy_static_include_bytes! {
    MSGR_CONFIG_XML => "assets/web/MsgrConfig.xml",
    BANNER => "assets/web/banner.html",
    TEXT_AD => "assets/web/ads/textad.xml",
    PPCRLCONFIG => "assets/web/ppcrlconfig.bin",
    WLIDSVCCONFIG => "assets/web/wlidsvcconfig.xml",
    PPCRLCHECK => "assets/web/ppcrlcheck.srf.html"
}

#[post("/")]
pub async fn firewall_test(request: HttpRequest) -> Result<HttpResponse, WebError> {
    return Ok(HttpResponseBuilder::new(StatusCode::OK).finish());
}

#[post("/RST2.srf")]
pub async fn rst2(body: web::Bytes, request: HttpRequest) -> Result<HttpResponse, WebError> {
    let request_body_str = std::str::from_utf8(&body).unwrap();
    info!("RST2 Request: {}", &request_body_str);

    let request_parsed = RST2RequestMessageSoapEnvelope::try_from_xml(request_body_str).unwrap();
    let username_token = request_parsed.header.security.username_token.unwrap();

    let matrix_id = OwnedUserId::from_msn_addr(&username_token.username);
    let msn_user = MSNUser::from_matrix_id(matrix_id.clone());


    //TODO pass disable ssl through config
    let client = get_matrix_client_builder(matrix_id.server_name(), SETTINGS_LOCATOR.homeserver_url.clone(), true).build().await?;
    
    match client.matrix_auth().login_username(matrix_id.as_str(), username_token.password.as_str()).device_id(get_matrix_device_id().as_str()).initial_device_display_name("WLMatrix").await {
        Ok(result) => {
            let response = RST2ResponseFactory::get_rst2_success_response(
                result.access_token,
                username_token.username,
                msn_user.get_uuid(),
            );
        
            let response_serialized = to_string(&response)?;
            info!("RST2 Response: {}", &response_serialized);
            return Ok(HttpResponseBuilder::new(StatusCode::OK)
                .append_header(("Content-Type", "application/soap+xml"))
                .body(response_serialized));
        },
        Err(error) => {
            log::error!("Unable to login to homeserver: {}", &error);
            if let matrix_sdk::Error::Http(err) = error {
               if let Some(test) = err.as_client_api_error(){
               if let ErrorBody::Standard { kind, message } = &test.body {
                    if &ErrorKind::Forbidden == kind {
                        return Err(WebError { message: Some(RST2ResponseFactory::get_auth_error_response()), status_code: StatusCode::INTERNAL_SERVER_ERROR });
                    }
               }
            }
        }
    }
    
    }
    return Err(WebError { message: None, status_code: StatusCode::INTERNAL_SERVER_ERROR });
}

#[get("/Config/MsgrConfig.asmx")]
pub async fn get_msgr_config() -> HttpResponse {
    let data: &'static [u8] = *MSGR_CONFIG_XML;
    return HttpResponseBuilder::new(StatusCode::OK)
        .append_header(("Content-Type", "application/soap+xml"))
        .body(data);
}

#[get("/ads/banner")]
pub async fn get_banner() -> HttpResponse {
    let data: &'static [u8] = *BANNER;
    return HttpResponseBuilder::new(StatusCode::OK)
        .append_header(("Content-Type", "text/html"))
        .body(data);
}

#[get("/ads/text")]
pub async fn get_text_ad() -> HttpResponse {
    let data: &'static [u8] = *TEXT_AD;
    return HttpResponseBuilder::new(StatusCode::OK)
        .append_header(("Content-Type", "text/xml"))
        .body(data);
}

#[post("/ppsecure/sha1auth.srf")]
pub async fn sha1auth(body: web::Bytes) -> Result<HttpResponse, WebError> {
    let body = decode(from_utf8(&body)?)?.into_owned();
    let captures = SHA1_REGEX.captures(&body).unwrap();
    let redirect_url = decode(&captures[1])?.into_owned();
    info!("Redirect to {}", &redirect_url);
    return Ok(HttpResponseBuilder::new(StatusCode::FOUND).append_header(("Location", redirect_url.as_str())).finish());
}


#[get("/ppcrlconfig.srf")]
pub async fn ppcrlconfigsrf() -> HttpResponse {
    info!("Downloading ppcrlconfig.srf");
    let data: &'static [u8] = *PPCRLCONFIG;
    return HttpResponseBuilder::new(StatusCode::OK)
        .append_header(("Content-Type", "application/octet-stream"))
        .body(data);
}

#[get("/ppcrlconfig.bin")]
pub async fn ppcrlconfig() -> HttpResponse {
    info!("Downloading ppcrlconfig");
    let data: &'static [u8] = *PPCRLCONFIG;
    return HttpResponseBuilder::new(StatusCode::OK)
        .append_header(("Content-Type", "application/octet-stream"))
        .body(data);
}

#[get("/PPCRLconfig.srf")]
pub async fn wlidsvcconfig() -> HttpResponse {
    info!("PPCRLconfig.srf Downloading wlidsvcconfig.xml");
    let data: &'static [u8] = *WLIDSVCCONFIG;
    return HttpResponseBuilder::new(StatusCode::OK)
        .append_header(("Content-Type", "text/xml"))
        .body(data);
}

#[get("/ppcrlcheck.srf")]
pub async fn ppcrlcheck() -> HttpResponse {
    let data: &'static [u8] = *PPCRLCHECK;
    return HttpResponseBuilder::new(StatusCode::OK)
        .append_header(("Content-Type", "text/html"))
        .body(data);
}
