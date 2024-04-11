use axum::http::StatusCode;
use axum::response::Response;
use axum::http::header::CONTENT_TYPE;
use axum::body::Body;

pub fn build_soap_response(body: String, status_code: StatusCode) -> Response {
     Response::builder().status(status_code)
        .header(CONTENT_TYPE, "application/soap+xml")
        .body(Body::from(body)).expect("RST2 response to be valid")
}
