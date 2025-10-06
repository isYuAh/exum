use std::sync::Arc;

use exum::{*};

#[state]
async fn str_dep() -> String {
    "hello".to_string()
}

#[get("/hello")]
async fn hello(#[dep] str_dep: Arc<MyService>) -> String {
    format!("{:?}", str_dep)
}

#[derive(Debug)]
struct MyService {
    str_dep: Arc<String>,
}
#[service]
impl MyService {
    async fn new(depend: Arc<Dependency>) -> Self {
        Self {
            str_dep: Arc::new(format!("CONCAT: {:?}", depend.depend)),
        }
    }
}

struct Dependency {
    depend: String
}
#[service]
impl Dependency {
    async fn new() -> Self {
        Self {
            depend: "This is Depend".to_string(),
        }
    }
}

#[main]
async fn main() {
    
}