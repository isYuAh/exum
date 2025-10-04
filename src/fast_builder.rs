use axum::{body::Body, http::{Response, StatusCode}};
use tower_http::cors::{Any, CorsLayer};

pub fn reponse_not_found() -> Response<Body> {
    Response::builder().status(StatusCode::NOT_FOUND)
        .body(Body::from("404 Not Found"))
        .unwrap()
}

pub fn response_bad_request() -> Response<Body> {
    Response::builder().status(StatusCode::BAD_REQUEST)
        .body(Body::from("400 Bad Request"))
        .unwrap()
}

pub fn response_ok<T: Into<Body>>(body: T) -> Response<Body> {
    Response::builder().status(StatusCode::OK)
        .body(body.into())
        .unwrap()
}

pub fn internal_server_error() -> Response<Body> {
    Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from("500 Internal Server Error"))
        .unwrap()
}

pub fn response_method_not_allowed() -> Response<Body> {
    Response::builder().status(StatusCode::METHOD_NOT_ALLOWED)
        .body(Body::from("405 Method Not Allowed"))
        .unwrap()
}

pub fn cors_any() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any)
}