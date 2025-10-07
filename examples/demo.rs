use std::collections::{HashMap};
use std::fmt::Debug;
use std::sync::{Arc};
use tokio::sync::Mutex;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use axum::Router;
use exum::layers::static_layer::StaticFileServiceBuilder;
use exum::*;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use serde_json::Value;

type RedisServiceImpl = SimpleRedisService;
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Response<T: Serialize> {
    code: u16,
    msg: String,
    data: Option<T>,
}
impl<T: Serialize> IntoResponse for Response<T> {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::to_string(&self).unwrap();
        (body).into_response()
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum RValue {
    String(String),
    Number(u64),
    Boolean(bool),
    HashMap(HashMap<String, RValue>),
    Array(Vec<RValue>),
    Null,
}
impl RValue {
    pub fn from_json(json_value: serde_json::Value) -> Self {
        match json_value {
            serde_json::Value::String(s) => Self::String(s),
            serde_json::Value::Number(n) => Self::Number(n.as_u64().unwrap()),
            serde_json::Value::Bool(b) => Self::Boolean(b),
            serde_json::Value::Object(map) => Self::HashMap(map.into_iter().map(|(k, v)| (k, Self::from_json(v))).collect()),
            serde_json::Value::Array(vec) => Self::Array(vec.into_iter().map(|v| Self::from_json(v)).collect()),
            serde_json::Value::Null => Self::Null,
        }
    }
    pub fn val(&self) -> serde_json::Value {
        match self {
            Self::String(s) => serde_json::Value::String(s.clone()),
            Self::Number(n) => serde_json::Value::Number(serde_json::Number::from(*n)),
            Self::Boolean(b) => serde_json::Value::Bool(*b),
            Self::HashMap(map) => serde_json::Value::Object(map.iter().map(|(k, v)| (k.clone(), v.val().clone())).collect()),
            Self::Array(vec) => serde_json::Value::Array(vec.iter().map(|v| v.val().clone()).collect()),
            Self::Null => serde_json::Value::Null,
        }
    }
}

pub trait RedisService: Debug + Send + Sync {
    async fn set(&self, topic: String, key: String, value: &RValue);
    async fn get(&self, topic: String, key: String) -> Option<RValue>;
    async fn list_by_topic(&self, topic: String) -> HashMap<String, RValue>;
    async fn del(&self, topic: String, key: String);
    async fn clear(&self, topic: String);
    async fn destroy(&self);
}

#[derive(Clone, Debug)]
struct SimpleRedisService {
    data: Arc<Mutex<HashMap<String, HashMap<String, RValue>>>>,
}
impl SimpleRedisService {
    async fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
#[service]
impl RedisService for SimpleRedisService {
    async fn set(&self, topic: String, key: String, value: &RValue) {
        self.data.lock().await.entry(topic).or_insert(HashMap::new()).insert(key, value.clone());
    }
    async fn get(&self, topic: String, key: String) -> Option<RValue> {
        self.data.lock().await.get(&topic)?.get(&key).cloned()
    }
    async fn list_by_topic(&self, topic: String) -> HashMap<String, RValue> {
        self.data.lock().await.entry(topic).or_insert(HashMap::new()).clone()
    }
    async fn del(&self, topic: String, key: String) {
        self.data.lock().await.entry(topic).or_insert(HashMap::new()).remove(&key);
    }
    async fn clear(&self, topic: String) {
        self.data.lock().await.entry(topic).or_insert(HashMap::new()).clear();
    }

    async fn destroy(&self) {
        self.data.lock().await.clear();
    }
}

#[controller("/redis")]
impl SimpleRedisController {
    #[get("/list")]
    async fn list(
        service: dyn RedisService,
        #[q] topic: String) {
        let list = service.list_by_topic(topic).await;
        Json(Response {
            code: 200,
            msg: "success".to_string(),
            data: Some(list.clone().iter()
                    .map(|v| (v.0.clone(), v.1.val().clone()))
                    .collect::<HashMap<String, _>>()),
        })
    }

    #[get("/get")]
    async fn get(service: dyn RedisService, #[q] topic: String, #[q] key: String) {
        let value = service.get(topic, key).await;
        Json(match value {
            Some(v) => Response {
                code: 200,
                msg: "success".to_string(),
                data: Some(v.val()),

            },
            None => Response {
                code: 404,
                msg: "key not found".to_string(),
                data: None,
            }
        })
    }

    #[get("/set")]
    async fn set(service: dyn RedisService, #[q] topic: String, #[q] key: String, #[q] value: String) {
        service.set(topic, key, &RValue::String(value));
        Json(Response::<Option<RValue>> {
            code: 200,
            msg: "success".to_string(),
            data: None,
        })
    }
    #[post("/set")]
    async fn post_set(
        service: SimpleRedisService,
        #[q] topic: Option<String>,
        #[q] key: Option<String>,
        body: String,
    ) {
        let value = from_str::<serde_json::Value>(&body).unwrap();
        let final_topic = if let Some(t) = value.get("topic").and_then(|v| v.as_str()) {
            t.to_string()
        } else if let Some(t) = topic {
            t
        } else {
            return Json(Response {
                code: 400,
                msg: "topic is required".to_string(),
                data: None,
            });
        };
        let final_key = if let Some(k) = value.get("key").and_then(|v| v.as_str()) {
            k.to_string()
        } else if let Some(k) = key {
            k
        } else {
            return Json(Response {
                code: 400,
                msg: "key is required".to_string(),
                data: None,
            });
        };
        let final_value = if let Some(v) = value.get("value") {
            RValue::from_json(v.clone())
        } else {
            return Json(Response {
                code: 400,
                msg: "value is required".to_string(),
                data: None,
            });
        };
        service.set(final_topic.to_string(), final_key.to_string(), &final_value);
        Json(Response::<Value> {
            code: 200,
            msg: "success".to_string(),
            data: Some(final_value.val().clone()),
        })
    }

    #[get("/del")]
    async fn del(service: dyn RedisService, #[q] topic: String, #[q] key: String) {
        service.del(topic, key);
        Json(Response::<Option<RValue>> {
            code: 200,
            msg: "success".to_string(),
            data: None,
        })
    }

    #[get("/clear")]
    async fn clear(service: SimpleRedisService, #[q] topic: String) {
        service.clear(topic);
        Json(Response::<Option<RValue>> {
            code: 200,
            msg: "success".to_string(),
            data: None,
        })
    }

    #[delete("/destroy")]
    async fn destroy(service: SimpleRedisService) {
        service.destroy();
        Json(Response::<Option<RValue>> {
            code: 200,
            msg: "success".to_string(),
            data: None,
        })
    }

    #[get("/")]
    async fn index() {
        r"REDIS
        使用：
        1. /redis/list?topic=topic_name 列出topic下所有key
        2. /redis/get?topic=topic_name&key=key_name 获取key对应的值
        3. /redis/set?topic=topic_name&key=key_name&value=value_name 设置key对应的值
        4. /redis/del?topic=topic_name&key=key_name 删除key
        5. /redis/clear?topic=topic_name 清空topic下所有key
        6. /redis/destroy 销毁所有topic
        ".to_string()
    }

    #[get("/status")]
    async fn status(service: SimpleRedisService, #[q] topic: Option<String>) {
        match topic {
            Some(topic) => {
                let list = service.list_by_topic(topic.clone()).await;
                Response {
                    code: 200,
                    msg: format!("Summary for Topic {}", topic),
                    data: Some(serde_json::json!({
                        "topic": topic,
                        "count": list.len(),
                        "keys": list.keys().cloned().collect::<Vec<_>>(),
                    })),
                }
            },
            None => Response {
                code: 200,
                msg: "Summary For Redis".to_string(),
                data: Some(serde_json::json!({
                    "count": service.data.lock().await.len(),
                    "keys": service.data.lock().await.keys().cloned().collect::<Vec<_>>(),
                })),
            }
        }
    }
}

#[get("/r")]
async fn test_fn(service: dyn RedisService) -> String {
    format!(":{service:?}")
}

#[main]
async fn main() {
    app.merge(StaticFileServiceBuilder::new("./ig_static").cors_any().build_router("/static"));
    app.merge(Router::new().route("/r", axum::routing::get(test_fn)).with_state(Arc::new(SimpleRedisService::new().await)));
}