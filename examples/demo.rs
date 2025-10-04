use axum::routing::{on, MethodFilter};
use exum::{*};

#[main]
async fn main() {
    app_chain!(app, {
        route("/", on(MethodFilter::GET, || async { "Hello, World!" })),
        route("/hello", on(MethodFilter::GET, || async { "Hello, World!" }))
    });

}