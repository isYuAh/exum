use exum::*;

#[get("/")]
async fn index() {
    "快速开始".to_string()
}
#[get("/开始")]
async fn start() {
    "自动urlencode".to_string()
}

struct DemoService {
    name: String,
    data: Vec<String>,
}
#[service]
impl DemoService {
    async fn new() -> Self {
        Self {
            name: "DemoService 服务".to_string(),
            data: vec![],
        }
    }
    pub fn insert(&mut self, source: String, data: String) {
        self.data.push(format!("{} 来源于 {}", data, source));
    }
    pub fn list(&self) -> &Vec<String> {
        &self.data
    }
}

#[controller("/controller")]
impl DemoController {

    #[get("/list")]
    async fn list(service: DemoService) {
        format!("{} 列表: {:?}", service.name, &service.list())
    }

    #[get("/insert")]
    async fn insert(service: DemoService) {
        service.insert("None".to_string(), "数据".to_string());
    }

    #[get("/insert_q")]
    async fn insert_q(#[q] q: String, service: DemoService) {
        service.insert("Query Source".to_string(), q);
    }

    #[get("/insert/:data")]
    async fn insert_p(data: String, service: DemoService) {
        service.insert("Path Source".to_string(), data);
    }
}

#[main]
async fn main() {
    
}