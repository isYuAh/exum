use exum::{layers::static_layer::StaticFileService, *};

#[get("/hello/素材/*key")]
async fn hello(key: String) {
    format!("hello {:?}", key)
}

#[main]
async fn main() {
    app.merge(StaticFileService::builder("./素材")
        .with_cors(fast_builder::cors_any())
        .with_spa_fallback(true)
        .build_router("/素材"));
}