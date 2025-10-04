use exum::{config::ApplicationConfig, *};


#[get("/hello/:id")]
async fn hello(id: String, #[q] q: String) {
    format!("id: {}, q: {}", id, q)
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