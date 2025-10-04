use exum::{config::ApplicationConfig, *};


#[get("/hello/:id")]
async fn hello(
    id: String, 
    #[q] q: String, 
    #[q] (a, b): (i32, i32)
) {
    format!("id: {}, q: {}, a: {}, b: {}", id, q, a, b)
}
#[get("/hover")]
async fn hello2(id: String) -> String {
    format!("YOU!, {}", id)
}


#[tokio::main]
async fn main() {
    let app = Application::build(ApplicationConfig::default());
    app.run().await;
}