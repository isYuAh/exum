use exum::{*};

// #[state]
// async fn str_dep() -> String {
//     "hello".to_string()
// }

// #[get("/hello")]
// async fn hello() -> String {
//     format!("hello")
// }

#[controller("/prefix")]
impl HelloController {
    #[get("/hello")]
    async fn hello(#[q] q: String) -> String {
        format!("hello: {}", q)
    }

    #[route(path = "/docs", method = "GET")]
    async fn docs() {
        format!("docs")
    }
}

// #[derive(Debug)]
// struct MyService {
//     str_dep: Arc<String>,
// }
// #[service]
// impl MyService {
//     async fn new(depend: Arc<Dependency>) -> Self {
//         Self {
//             str_dep: Arc::new(format!("CONCAT: {:?}", depend.depend)),
//         }
//     }
// }

// struct Dependency {
//     depend: String
// }
// #[service]
// impl Dependency {
//     async fn new() -> Self {
//         Self {
//             depend: "This is Depend".to_string(),
//         }
//     }
// }

#[main]
async fn main() {
    
}