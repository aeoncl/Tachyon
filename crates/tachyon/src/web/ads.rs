use axum::body::Body;
use axum::extract::Path;
use axum::http::header::CONTENT_TYPE;
use axum::http::Response;
use axum::Router;
use axum::routing::{get, post};
use lazy_static::lazy_static;
use lazy_static_include::lazy_static_include_bytes;
use reqwest::StatusCode;
use yaserde::ser;
use yaserde_derive::YaSerialize;
use crate::tachyon::global_state::GlobalState;
use crate::web::matrix_today::get_msn_today;

lazy_static_include_bytes! {
    AVATAR => "./assets/img/avatar_48x48.jpg",
    BANNER => "./assets/web/banner.html",
    TEXT_AD => "./assets/web/ads/textad.xml",
    MATRIX_ICON => "./assets/img/matrix-icon.png",
    SPONGEBOB_ICON => "./assets/img/spongebob.png",
    ALERT_BACKGROUND => "./assets/web/ads/alert-background.png"
}

lazy_static! {
        static ref TAB_ADS: Vec<Ads> = vec![
        Ads{
            tab_ads: vec![
                TabAd {
                    image: "http://127.0.0.1:8080/ads/matrix-icon.png".to_string(),
                    name: "Matrix Today".to_string(),
                    tab_type: "matrix".to_string(),
                    tooltip: "Find out what's up in the Matrix ecosystem".to_string(),
                    content_url: "http://127.0.0.1:8080/ads/msn-today".to_string(),
                    hit_url: "http://127.0.0.1:8080/".to_string(),
                    site_id: 0,
                    notification_id: 0,
                }
            ]
        }
        ];
}

pub fn ads_router(state: GlobalState) -> Router<GlobalState> {
    Router::new()
        .route("/banner", get(get_banner_ads))
        .route("/avatar.jpg", get(get_avatar_jpg))
        .route("/text", get(get_text_ad))
        .route("/msn-today", get(get_msn_today))
        .route("/tabad/{tab_index}", get(get_tab_ad))
        .route("/matrix-icon.png", get(get_matrix_icon))
        .route("/matrix-icon_32x32.png", get(get_matrix_icon))
        .route("/spongebob-icon_48x32.png", get(get_spongebob_icon))
        .route("/spongebob-icon.png", get(get_spongebob_icon))
        .route("/alert-background.png", get(get_alert_background))
        .with_state(state)
}

#[derive(Debug, Default, YaSerialize)]
#[yaserde(rename = "tabad")]
struct TabAd {
    image: String,
    name: String,
    #[yaserde(rename = "type")]
    tab_type: String,
    tooltip: String,
    #[yaserde(rename = "contenturl")]
    content_url: String,
    #[yaserde(rename = "hiturl")]
    hit_url: String,
    #[yaserde(rename = "siteid")]
    site_id: u32,
    #[yaserde(rename = "notificationid")]
    notification_id: u32
}

#[derive(Debug, YaSerialize)]
#[yaserde(rename = "ads")]
struct Ads {
    #[yaserde(rename = "tabad")]
    tab_ads: Vec<TabAd>
}


pub async fn get_tab_ad(Path(tab_index): Path<u32>) -> Response<Body> {

    match TAB_ADS.get(tab_index as usize) {
        None => {
            Response::builder().status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap()
        }
        Some(tab_ad) => {
            Response::builder()
                .status(StatusCode::OK)
                .header(CONTENT_TYPE, "text/xml")
                .body(Body::from(ser::to_string(tab_ad).unwrap())).unwrap()
        }
    }
}

pub async fn get_banner_ads() -> Response<Body> {
    let data: &'static [u8] = *BANNER;

    axum::response::Response::builder()
        .header(CONTENT_TYPE, "text/html")
        .body(Body::from(data)).expect("banner ads response to be valid")
}

pub async fn get_matrix_icon() -> Response<Body> {
    let data: &'static [u8] = *MATRIX_ICON;

    axum::response::Response::builder()
        .header(CONTENT_TYPE, "image/png")
        .body(Body::from(data)).expect("banner ads response to be valid")

}

pub async fn get_avatar_jpg() -> Response<Body> {
    let data: &'static [u8] = *AVATAR;

    axum::response::Response::builder()
        .header(CONTENT_TYPE, "image/jpg")
        .body(Body::from(data)).expect("banner ads response to be valid")

}

pub async fn get_spongebob_icon() -> Response<Body> {
    let data: &'static [u8] = *SPONGEBOB_ICON;

    axum::response::Response::builder()
        .header(CONTENT_TYPE, "image/png")
        .body(Body::from(data)).expect("banner ads response to be valid")

}

pub async fn get_alert_background() -> Response<Body> {
    let data: &'static [u8] = *ALERT_BACKGROUND;

    axum::response::Response::builder()
        .header(CONTENT_TYPE, "image/png")
        .body(Body::from(data)).expect("banner ads response to be valid")

}

pub async fn get_text_ad() -> Response<Body> {
    let data: &'static [u8] = *TEXT_AD;

    axum::response::Response::builder()
        .header(CONTENT_TYPE, "text/html")
        .body(Body::from(data)).expect("Text ad response to be valid")

}

#[cfg(test)]
mod tests {
    use yaserde::ser;
    use crate::web::ads::{Ads, TabAd};

    #[test]
    fn tab_ad_serialization_test() {
        let tab = Ads{
            tab_ads: vec![TabAd {
                image: "http://img.local".to_string(),
                name: "Matrix Today".to_string(),
                tab_type: Default::default(),
                tooltip: "Whats happening in the matrix ecosystem".to_string(),
                content_url: "http://127.0.0.1/ads/msn-today".to_string(),
                hit_url: "http://127.0.0.1/".to_string(),
                site_id: 0,
                notification_id: 0,
            }]
        };
        


        let test = ser::to_string(&tab).unwrap();

        println!("{}", test);

    }
}