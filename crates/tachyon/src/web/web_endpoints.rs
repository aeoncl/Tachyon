use std::str::from_utf8;
use axum::body::Body;
use axum::extract::Path;
use axum::http::{HeaderMap, HeaderName, Response, StatusCode};
use axum::http::header::{CONTENT_TYPE, LOCATION};
use base64::Engine;
use base64::engine::general_purpose;
use lazy_static::lazy_static;
use lazy_static_include::lazy_static_include_bytes;
use matrix_sdk::Client;
use matrix_sdk::media::{MediaFormat, MediaRequest, MediaThumbnailSize};
use matrix_sdk::ruma::api::client::media::get_content_thumbnail::v3::Method;
use matrix_sdk::ruma::events::room::MediaSource;
use matrix_sdk::ruma::{mxc_uri, MxcUri, OwnedMxcUri, UInt};
use regex::Regex;
use crate::web::soap::shared::build_soap_response;

lazy_static! {
    pub static ref DEFAULT_CACHE_KEY: String = String::from("12r1:8nBBE6vX1J4uPKajtbem5XBIblimCwAhIziAeEAwYD0AMiaztryWvcZthkN9oX_pl2scBKXfKvRvuWKYdHUNuRkgiyV9rzcDpnDIDiM6vdcEB6d82wjjnL4TAFAjc5X8i-C94mNfQvujUk470P7fz9qbWfK6ANcEtygDb-oWsYVfEBrxl6geTUg9tGT7yCIsls7ECcLyqwsROuAbWCrued_VPKiUgSIvqG8gaA");
    pub static ref SHA1_REGEX: Regex = Regex::new(r"ru=([^&]*)&").unwrap();

}

lazy_static_include_bytes! {
    MSGR_CONFIG_XML => "./assets/web/MsgrConfig.xml",
    BANNER => "./assets/web/banner.html",
    TEXT_AD => "./assets/web/ads/textad.xml",
    PPCRLCONFIG => "./assets/web/ppcrlconfig.bin",
    WLIDSVCCONFIG => "./assets/web/wlidsvcconfig.xml",
    PPCRLCHECK => "./assets/web/ppcrlcheck.srf.html"
}

pub async fn firewall_test() -> StatusCode {
    StatusCode::OK
}

pub async fn get_msgr_config() -> Response<Body> {
    let data: &'static [u8] = *MSGR_CONFIG_XML;
    build_soap_response(from_utf8(data).expect("MsgrConfig to be valid").to_string(), StatusCode::OK)
}

pub async fn get_banner_ads() -> Response<Body> {
    let data: &'static [u8] = *BANNER;

    axum::response::Response::builder()
        .header(CONTENT_TYPE, "text/html")
        .body(Body::from(data)).expect("banner ads response to be valid")

}

pub async fn get_text_ad() -> Response<Body> {
    let data: &'static [u8] = *TEXT_AD;

    axum::response::Response::builder()
        .header(CONTENT_TYPE, "text/html")
        .body(Body::from(data)).expect("Text ad response to be valid")

}

pub async fn sha1auth(body: String) -> (StatusCode, HeaderMap ){

    let bodyDecoded = urlencoding::decode(&body).unwrap().to_string();
    let captures = SHA1_REGEX.captures(&bodyDecoded).unwrap();
    let redirect_url = urlencoding::decode(&captures[1]).expect("Url to be correct").into_owned();

    let mut headers = HeaderMap::new();
    headers.insert(LOCATION, redirect_url.parse().expect("Redirect Url to be valid"));
    (StatusCode::FOUND, headers)
}

pub async fn ppcrlconfigsrf() -> Vec<u8> {
    let data: &'static [u8] = *PPCRLCONFIG;
    data.to_vec()

}

pub async fn wlidsvcconfig() -> Response<Body> {

    let data: &'static [u8] = *WLIDSVCCONFIG;
    Response::builder()
        .header(CONTENT_TYPE, "text/xml")
        .body(Body::from(data)).expect("wlid config to be valid")

}

pub async fn ppcrlcheck() -> Response<Body> {

    let data: &'static [u8] = *PPCRLCHECK;
    Response::builder()
        .header(CONTENT_TYPE, "text/xml")
        .body(Body::from(data)).expect("wlid config to be valid")

}

pub async fn get_profile_pic(Path((image_mxid, _image_type)): Path<(String, String)>) -> Response<Body> {

    //Todo handle errors
    let image_mxid = String::from_utf8(general_purpose::STANDARD.decode(image_mxid.as_bytes()).unwrap()).unwrap();
    let parsed_mxc = OwnedMxcUri::from(image_mxid);

    let client= Client::builder().disable_ssl_verification().server_name(parsed_mxc.server_name().unwrap()).build().await.unwrap();

    let media_request = MediaRequest{ source: MediaSource::Plain(parsed_mxc), format: MediaFormat::Thumbnail(MediaThumbnailSize{ method: Method::Scale, width: UInt::new(200).unwrap(), height: UInt::new(200).unwrap() })};
    let image = client.media().get_media_content(&media_request, false).await.unwrap();

    Response::builder()
        .header(CONTENT_TYPE, "image/jpeg")
        .body(Body::from(image)).expect("Image to be valid")
}