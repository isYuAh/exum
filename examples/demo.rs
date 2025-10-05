use exum::{*};

#[controller("/files")]
impl FileController {
    #[route(path="/123")]
    async fn list(&self) -> String {
        format!("file list")
    }

    #[get("/hello")]
    async fn hello() {
        "hello"
    }
}

#[main]
async fn main() {
}