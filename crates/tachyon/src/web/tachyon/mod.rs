use crate::tachyon::tachyon_state::TachyonState;
use axum::body::Body;
use axum::extract::Path;
use axum::http::header::CONTENT_TYPE;
use axum::http::{Response, StatusCode};
use axum::response::Html;
use axum::routing::get;
use axum::Router;
use lazy_static_include::lazy_static_include_bytes;
use maud::{html, Markup, DOCTYPE};

lazy_static_include_bytes! {
    INDEX => "./assets/web/tachyon/index.html",
    FAVICON => "./assets/web/tachyon/favicon.ico",
    STYLE => "./assets/web/tachyon/style.css",
    LOGO => "./assets/web/tachyon/tachyon_logo.png",
    LOGO_2 => "./assets/web/tachyon/tachyon_logo_2.png",
    INTERCOOLER => "./assets/web/tachyon/intercooler-1.2.4.min.js",
    JQUERY => "./assets/web/tachyon/jquery-1.10.0.min.js"
}
pub fn tachyon_router(state: TachyonState) -> Router<TachyonState> {
    Router::new()
        .route("/", get(serve_index))
        .route("/{file}", get(serve_static))
        .with_state(state)
}

pub fn index_page() -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                title { "Tachyon" }
                base href="/tachyon/";
                link rel="icon" type="image/x-icon" href="favicon.ico";
                link rel="stylesheet" href="style.css";
                script type="text/javascript" src="jquery-1.10.0.min.js" {}
                script type="text/javascript" src="intercooler-1.2.4.min.js" {}
            }
            body {
                div class="header" {
                    div class="bg" {
                        div class="bg-content" {
                            img class="logo" src="tachyon_logo_2.png" alt="Tachyon Logo";
                            div class="title" {
                                h1 { "Tachyon" }
                                h2 { "Welcome to Tachyon" }
                            }

                            div class="menu" {
                                ul {
                                    li { "Home" }
                                    li { "Profile" }
                                    li { "People" }
                                }
                            }

                            div class="signin" {
                                h2 { "Log-on" }
                            }
                        }
                    }
                }

                div class="content" {}
            }
        }
    }
}

async fn serve_index() -> Html<String> {
    Html(index_page().into_string())
}

async fn serve_static(Path(file): Path<String>) -> Response<Body> {
    let (data, content_type) = match file.as_str() {
        "favicon.ico" => (*FAVICON, "image/x-icon"),
        "style.css" => (*STYLE, "text/css"),
        "tachyon_logo.png" => (*LOGO, "image/png"),
        "tachyon_logo_2.png" => (*LOGO_2, "image/png"),
        "intercooler-1.2.4.min.js" => (*INTERCOOLER, "text/javascript"),
        "jquery-1.10.0.min.js" => (*JQUERY, "text/javascript"),
        _ => return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .unwrap(),
    };

    Response::builder()
        .header(CONTENT_TYPE, content_type)
        .body(Body::from(data))
        .expect("response to be valid")
}