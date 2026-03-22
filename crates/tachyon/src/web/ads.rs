use axum::body::Body;
use axum::extract::Path;
use axum::http::header::CONTENT_TYPE;
use axum::http::Response;
use lazy_static_include::lazy_static_include_bytes;
use maud::{html, Markup};

lazy_static_include_bytes! {
    BANNER => "./assets/web/banner.html",
    TEXT_AD => "./assets/web/ads/textad.xml",
}

pub async fn get_tab_add(Path(tab_index): Path<u32>) -> Response<Body> {

    let str = format!(r#"<?xml version="1.0" encoding="UTF-8" ?>
<ads>
    <tabad>
        <image>http://127.0.0.1:8080/ads/tab/image/{tab_index}</image>
        <name>{name}</name>
        <type>page</type>
        <tooltip>Whats new in Matrix</tooltip>
        <contenturl>http://127.0.0.1:8080/ads/matrix-today</contenturl>
        <hiturl>http://127.0.0.1:8080</hiturl>
        <siteid>0</siteid>
        <notificationid>0</notificationid>
    </tabad>
</ads>
    "#, tab_index = tab_index, name = "name");

    todo!()

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