mod middleware;
mod layout;
mod matrix_auth;
mod login;
mod confirm_device;
mod verification;
mod client;

use crate::tachyon::global::global_state::GlobalState;
use axum::body::Body;
use axum::extract::Path;
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE, ETAG, IF_NONE_MATCH};
use axum::http::{HeaderMap, Method, Response, StatusCode};
use axum::middleware::{from_fn, from_fn_with_state};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, head, post};
use axum::Router;
use lazy_static_include::lazy_static_include_bytes;
use maud::html;
use std::str::FromStr;
use sha1::{Digest, Sha1};
use crate::tachyon::repository::RepositoryStr;
use crate::web::tachyon::confirm_device::{reset_identity, recover, other_device};

lazy_static_include_bytes! {
    INDEX => "./assets/web/tachyon/index.html",
    FAVICON => "./assets/web/tachyon/favicon.ico",
    STYLE => "./assets/web/tachyon/style.css",
    INTERCOOLER => "./assets/web/tachyon/intercooler-1.2.4.min.js",
    JQUERY => "./assets/web/tachyon/jquery-1.10.0.min.js",
    VERIFY_SCRIPT => "./assets/web/tachyon/verify.js",
    TREMOVE_SCRIPT => "./assets/web/tachyon/tremove.js",
    SHIELD_VERIFY => "./assets/web/tachyon/img/shield_verify.png",
    LOGO => "./assets/web/tachyon/img/tachyon_logo.png",
    LOGO_2 => "./assets/web/tachyon/img/tachyon_logo_2.png",
    SMILEY_SCARED => "./assets/web/tachyon/img/scared-emoticon.gif",
    SMILEY_BOSSEORDI => "./assets/web/tachyon/img/smiley_bosseordi.gif",
    SMILEY_LOOKING => "./assets/web/tachyon/img/text-looking.gif",
    STAR_SPEECH => "./assets/web/tachyon/img/star_speech.gif",
    KEY => "./assets/web/tachyon/img/key.gif",
    TACHYON_BANNER => "./assets/web/tachyon/img/tachyon_banner_bigger.png"
}

lazy_static_include_bytes! {
    SAS_0 => "./assets/web/tachyon/img/sas_v1/0_dog.gif",
    SAS_1 => "./assets/web/tachyon/img/sas_v1/1_cat.gif",
    SAS_2 => "./assets/web/tachyon/img/sas_v1/2_lion.gif",
    SAS_3 => "./assets/web/tachyon/img/sas_v1/3_horse.gif",
    SAS_4 => "./assets/web/tachyon/img/sas_v1/4_unicorn.gif",
    SAS_5 => "./assets/web/tachyon/img/sas_v1/5_pig.gif",
    SAS_6 => "./assets/web/tachyon/img/sas_v1/6_elephant.gif",
    SAS_7 => "./assets/web/tachyon/img/sas_v1/7_rabbit.gif",
    SAS_8 => "./assets/web/tachyon/img/sas_v1/8_panda.gif",
    SAS_9 => "./assets/web/tachyon/img/sas_v1/9_rooster.gif",
    SAS_10 => "./assets/web/tachyon/img/sas_v1/10_penguin.gif",
    SAS_11 => "./assets/web/tachyon/img/sas_v1/11_turtle.gif",
    SAS_12 => "./assets/web/tachyon/img/sas_v1/12_fish.gif",
    SAS_13 => "./assets/web/tachyon/img/sas_v1/13_octopus.gif",
    SAS_14 => "./assets/web/tachyon/img/sas_v1/14_butterfly.gif",
    SAS_15 => "./assets/web/tachyon/img/sas_v1/15_flower.gif",
    SAS_16 => "./assets/web/tachyon/img/sas_v1/16_tree.gif",
    SAS_17 => "./assets/web/tachyon/img/sas_v1/17_cactus.gif",
    SAS_18 => "./assets/web/tachyon/img/sas_v1/18_mushroom.gif",
    SAS_19 => "./assets/web/tachyon/img/sas_v1/19_globe.gif",
    SAS_20 => "./assets/web/tachyon/img/sas_v1/20_moon.gif",
    SAS_21 => "./assets/web/tachyon/img/sas_v1/21_cloud.gif",
    SAS_22 => "./assets/web/tachyon/img/sas_v1/22_fire.gif",
    SAS_23 => "./assets/web/tachyon/img/sas_v1/23_banana.gif",
    SAS_24 => "./assets/web/tachyon/img/sas_v1/24_apple.gif",
    SAS_25 => "./assets/web/tachyon/img/sas_v1/25_strawberry.gif",
    SAS_26 => "./assets/web/tachyon/img/sas_v1/26_corn.gif",
    SAS_27 => "./assets/web/tachyon/img/sas_v1/27_pizza.gif",
    SAS_28 => "./assets/web/tachyon/img/sas_v1/28_cake.gif",
    SAS_29 => "./assets/web/tachyon/img/sas_v1/29_heart.gif",
    SAS_30 => "./assets/web/tachyon/img/sas_v1/30_smiley.gif",
    SAS_31 => "./assets/web/tachyon/img/sas_v1/31_robot.gif",
    SAS_32 => "./assets/web/tachyon/img/sas_v1/32_hat.gif",
    SAS_33 => "./assets/web/tachyon/img/sas_v1/33_glasses.gif",
    SAS_34 => "./assets/web/tachyon/img/sas_v1/34_spanner.gif",
    SAS_35 => "./assets/web/tachyon/img/sas_v1/35_santa.gif",
    SAS_36 => "./assets/web/tachyon/img/sas_v1/36_thumbs_up.gif",
    SAS_37 => "./assets/web/tachyon/img/sas_v1/37_umbrella.gif",
    SAS_38 => "./assets/web/tachyon/img/sas_v1/38_hourglass.gif",
    SAS_39 => "./assets/web/tachyon/img/sas_v1/39_clock.gif",
    SAS_40 => "./assets/web/tachyon/img/sas_v1/40_gift.gif",
    SAS_41 => "./assets/web/tachyon/img/sas_v1/41_light_bulb.gif",
    SAS_42 => "./assets/web/tachyon/img/sas_v1/42_book.gif",
    SAS_43 => "./assets/web/tachyon/img/sas_v1/43_pencil.gif",
    SAS_44 => "./assets/web/tachyon/img/sas_v1/44_paperclip.gif",
    SAS_45 => "./assets/web/tachyon/img/sas_v1/45_scissors.gif",
    sas_46 => "./assets/web/tachyon/img/sas_v1/46_lock.gif",
    SAS_47 => "./assets/web/tachyon/img/sas_v1/47_key.gif",
    SAS_48 => "./assets/web/tachyon/img/sas_v1/48_hammer.gif",
    SAS_49 => "./assets/web/tachyon/img/sas_v1/49_telephone.gif",
    SAS_50 => "./assets/web/tachyon/img/sas_v1/50_flag.gif",
    SAS_51 => "./assets/web/tachyon/img/sas_v1/51_train.gif",
    SAS_52 => "./assets/web/tachyon/img/sas_v1/52_bicycle.gif",
    SAS_53 => "./assets/web/tachyon/img/sas_v1/53_aeroplane.gif",
    SAS_54 => "./assets/web/tachyon/img/sas_v1/54_rocket.gif",
    SAS_55 => "./assets/web/tachyon/img/sas_v1/55_trophy.gif",
    SAS_56 => "./assets/web/tachyon/img/sas_v1/56_ball.gif",
    SAS_57 => "./assets/web/tachyon/img/sas_v1/57_guitar.gif",
    SAS_58 => "./assets/web/tachyon/img/sas_v1/58_trumpet.gif",
    SAS_59 => "./assets/web/tachyon/img/sas_v1/59_bell.gif",
    SAS_60 => "./assets/web/tachyon/img/sas_v1/60_anchor.gif",
    SAS_61 => "./assets/web/tachyon/img/sas_v1/61_headphones.gif",
    SAS_62 => "./assets/web/tachyon/img/sas_v1/62_folder.gif",
    SAS_63 => "./assets/web/tachyon/img/sas_v1/63_pin.gif",
}


pub fn tachyon_router(state: GlobalState) -> Router<GlobalState> {
    Router::new()
        //Secured v
        .route("/test", get(serve_index))
        .route("/confirm_device", get(confirm_device::get_confirm))
        .route("/confirm_device/reset_identity", post(reset_identity::post_reset_identity))
        .route("/confirm_device/reset_identity", get(reset_identity::get_reset_identity))
        .route("/confirm_device/recover", get(recover::get_recover))
        .route("/confirm_device/recover", post(recover::post_recover))
        .route("/confirm_device/other_device", get(other_device::get_other_device))
        .route("/confirm_device/other_device", post(other_device::post_other_device))
        .route("/verification", get(verification::get_verification_poll))
        .route("/verification/sas_v1/{action}", post(verification::sas_v1_actions::post_sas_v1_action))
        .route("/login/nfy", get(login::get_login_nfy))
        .layer(from_fn_with_state(state.clone(), middleware::is_authenticated))
        .layer(from_fn(middleware::intercooler_layout_wrapper))
        //Unsecured v
        .route("/", get(serve_index))
        .route("/login", get(login::get_login_page))
        .route("/login/request", get(login::get_login_request))
        .route("/login/request", post(login::post_login_request))
        .route("/auth", get(matrix_auth::get_auth))
        .route("/auth", post(matrix_auth::post_auth))
        .route("/img/sas_v1/{file}", get(serve_static))
        .route("/img/sas_v1/{file}", head(serve_static))
        .route("/img/{file}", get(serve_static))
        .route("/img/{file}", head(serve_static))
        .route("/{file}", get(serve_static))
        .route("/{file}", head(serve_static))
        .layer(from_fn(middleware::extract_token))
        .with_state(state)
}

type Params = std::collections::HashMap<String, String>;

async fn serve_index() -> Html<String> {
    Html(
        layout::tachyon_page(html! {
            h2 { "Tachyon is running..." }
        })
            .into_string(),
    )
}

async fn serve_static(
    method: Method,
    Path(file): Path<String>,
    headers: HeaderMap,
) -> Response<Body> {
    let (data, content_type) = match file.as_str() {
        "favicon.ico" => (*FAVICON, "image/x-icon"),
        "style.css" => (*STYLE, "text/css"),
        "intercooler-1.2.4.min.js" => (*INTERCOOLER, "text/javascript"),
        "jquery-1.10.0.min.js" => (*JQUERY, "text/javascript"),
        "verify.js" => (*VERIFY_SCRIPT, "text/javascript"),
        "tremove.js" => (*TREMOVE_SCRIPT, "text/javascript"),
        "tachyon_logo.png" => (*LOGO, "image/png"),
        "tachyon_logo_2.png" => (*LOGO_2, "image/png"),
        "shield_verify.png" => (*SHIELD_VERIFY, "image/png"),
        "scared-emoticon.gif" => (*SMILEY_SCARED, "image/gif"),
        "smiley_bosseordi.gif" => (*SMILEY_BOSSEORDI, "image/gif"),
        "text-looking.gif" => (*SMILEY_LOOKING, "image/gif"),
        "star_speech.gif" => (*STAR_SPEECH, "image/gif"),
        "key-icon.gif" => (*KEY, "image/gif"),
        "tachyon_banner_bigger.png" => (*TACHYON_BANNER, "image/png"),
        "dog.gif" => (*SAS_0, "image/gif"),
        "cat.gif" => (*SAS_1, "image/gif"),
        "lion.gif" => (*SAS_2, "image/gif"),
        "horse.gif" => (*SAS_3, "image/gif"),
        "unicorn.gif" => (*SAS_4, "image/gif"),
        "pig.gif" => (*SAS_5, "image/gif"),
        "elephant.gif" => (*SAS_6, "image/gif"),
        "rabbit.gif" => (*SAS_7, "image/gif"),
        "panda.gif" => (*SAS_8, "image/gif"),
        "rooster.gif" => (*SAS_9, "image/gif"),
        "penguin.gif" => (*SAS_10, "image/gif"),
        "turtle.gif" => (*SAS_11, "image/gif"),
        "fish.gif" => (*SAS_12, "image/gif"),
        "octopus.gif" => (*SAS_13, "image/gif"),
        "butterfly.gif" => (*SAS_14, "image/gif"),
        "flower.gif" => (*SAS_15, "image/gif"),
        "tree.gif" => (*SAS_16, "image/gif"),
        "cactus.gif" => (*SAS_17, "image/gif"),
        "mushroom.gif" => (*SAS_18, "image/gif"),
        "globe.gif" => (*SAS_19, "image/gif"),
        "moon.gif" => (*SAS_20, "image/gif"),
        "cloud.gif" => (*SAS_21, "image/gif"),
        "fire.gif" => (*SAS_22, "image/gif"),
        "banana.gif" => (*SAS_23, "image/gif"),
        "apple.gif" => (*SAS_24, "image/gif"),
        "strawberry.gif" => (*SAS_25, "image/gif"),
        "corn.gif" => (*SAS_26, "image/gif"),
        "pizza.gif" => (*SAS_27, "image/gif"),
        "cake.gif" => (*SAS_28, "image/gif"),
        "heart.gif" => (*SAS_29, "image/gif"),
        "smiley.gif" => (*SAS_30, "image/gif"),
        "robot.gif" => (*SAS_31, "image/gif"),
        "hat.gif" => (*SAS_32, "image/gif"),
        "glasses.gif" => (*SAS_33, "image/gif"),
        "spanner.gif" => (*SAS_34, "image/gif"),
        "santa.gif" => (*SAS_35, "image/gif"),
        "thumbs_up.gif" => (*SAS_36, "image/gif"),
        "umbrella.gif" => (*SAS_37, "image/gif"),
        "hourglass.gif" => (*SAS_38, "image/gif"),
        "clock.gif" => (*SAS_39, "image/gif"),
        "gift.gif" => (*SAS_40, "image/gif"),
        "light_bulb.gif" => (*SAS_41, "image/gif"),
        "book.gif" => (*SAS_42, "image/gif"),
        "pencil.gif" => (*SAS_43, "image/gif"),
        "paperclip.gif" => (*SAS_44, "image/gif"),
        "scissors.gif" => (*SAS_45, "image/gif"),
        "lock.gif" => (*sas_46, "image/gif"),
        "key.gif" => (*SAS_47, "image/gif"),
        "hammer.gif" => (*SAS_48, "image/gif"),
        "telephone.gif" => (*SAS_49, "image/gif"),
        "flag.gif" => (*SAS_50, "image/gif"),
        "train.gif" => (*SAS_51, "image/gif"),
        "bicycle.gif" => (*SAS_52, "image/gif"),
        "aeroplane.gif" => (*SAS_53, "image/gif"),
        "rocket.gif" => (*SAS_54, "image/gif"),
        "trophy.gif" => (*SAS_55, "image/gif"),
        "ball.gif" => (*SAS_56, "image/gif"),
        "guitar.gif" => (*SAS_57, "image/gif"),
        "trumpet.gif" => (*SAS_58, "image/gif"),
        "bell.gif" => (*SAS_59, "image/gif"),
        "anchor.gif" => (*SAS_60, "image/gif"),
        "headphones.gif" => (*SAS_61, "image/gif"),
        "folder.gif" => (*SAS_62, "image/gif"),
        "pin.gif" => (*SAS_63, "image/gif"),
        _ => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Not found"))
                .unwrap()
        }
    };

    if let Some(if_none_match) = headers.get(IF_NONE_MATCH) {
        if let Ok(if_none_match) = if_none_match.to_str() {
            if sha_1_encode(&data) == if_none_match {
                return Response::builder()
                    .status(StatusCode::NOT_MODIFIED)
                    .body(Body::empty())
                    .unwrap();
            }
        }
    }


    let response_builder = Response::builder()
        .header(CONTENT_TYPE, content_type)
        .header(CACHE_CONTROL, "public, max-age=604800, must-revalidate")
        .header(ETAG, sha_1_encode(&data))
        .status(
            StatusCode::OK,
        );

    if method == Method::GET {
        response_builder.body(Body::from(data)).unwrap()
    } else {
        response_builder.body(Body::empty()).unwrap()
    }
}

fn sha_1_encode(input: &[u8]) -> String {

    let mut hasher = Sha1::new();
    Digest::update(&mut hasher, input);
    let result = hasher.finalize();
    hex::encode(result)

}
