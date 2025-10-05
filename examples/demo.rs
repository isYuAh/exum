use exum::{*};

#[controller("/files")]
impl FileController {
    #[route(path="/123")]
    pub async fn list(&self) -> String {
        format!("file list")
    }

    #[get("/hello")]
    pub async fn hello() {
        "hello"
    }
}

#[main]
async fn main() {
    
}