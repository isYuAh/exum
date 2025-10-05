use std::sync::{atomic::AtomicUsize};

use exum::{layers::static_layer::StaticFileService, *};

#[get("/hello")]
async fn hello() {
    format!("hello")
}

#[get("/")]
async fn index(#[dep] cfg: TestConfig) -> String {
    format!("index {:?}", cfg)
}

#[derive(Debug)]
pub struct TestConfig {
    pub version: String,
}

#[derive(Debug)]
pub struct Counter {
    pub value: AtomicUsize,
}

#[state(prewarm)]
async fn config() -> TestConfig {
    println!("[config] init called");
    TestConfig { version: "v1.0".into() }
}

#[state]
async fn counter() -> Counter {
    println!("[counter] init called");
    Counter { value: AtomicUsize::new(0) }
}

#[main]
async fn main() {
    app.merge(StaticFileService::builder("./素材")
        .with_cors(fast_builder::cors_any())
        .with_spa_fallback(true)
        .build_router("/素材"));
}