use exum::{config::ApplicationConfig, *};
use serde::Deserialize;

#[derive(Deserialize)]
struct AQ{
    q: String,
}

#[get("/hello/:id")]
async fn hello(id: String, #[q] q: AQ) {
    format!("id: {}, q: {}", id, q.q)
}
#[get("/hover")]
async fn hello2(id: String) {
    format!("YOU!, {}", id)
}

#[tokio::main]
async fn main() {
    let app = Application::build(ApplicationConfig::default());
    app.run().await;
}