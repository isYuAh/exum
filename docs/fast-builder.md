# 快速响应构建器

Exum 提供了 `fast_builder` 模块，包含一系列便捷的HTTP响应构建函数，简化常见的HTTP响应创建过程。

## 功能特性

`fast_builder` 模块提供了以下响应构建函数：

### 标准HTTP响应
- **`response_ok(body)`** - 创建200 OK响应
- **`reponse_not_found()`** - 创建404 Not Found响应
- **`response_bad_request()`** - 创建400 Bad Request响应
- **`internal_server_error()`** - 创建500 Internal Server Error响应
- **`response_method_not_allowed()`** - 创建405 Method Not Allowed响应

### CORS支持
- **`cors_any()`** - 创建全允许的CORS配置层

## 使用示例

### 基础响应构建

```rust
use exum::fast_builder::*;

#[get("/api/data")]
async fn get_data() -> impl IntoResponse {
    response_ok("数据获取成功")  // 返回200 OK
}

#[get("/api/not-found")]
async fn not_found() -> impl IntoResponse {
    reponse_not_found()  // 返回404 Not Found
}

#[post("/api/bad-request")]
async fn bad_request() -> impl IntoResponse {
    response_bad_request()  // 返回400 Bad Request
}

#[get("/api/error")]
async fn server_error() -> impl IntoResponse {
    internal_server_error()  // 返回500 Internal Server Error
}

#[post("/api/method-not-allowed")]
async fn method_not_allowed() -> impl IntoResponse {
    response_method_not_allowed()  // 返回405 Method Not Allowed
}
```

### 自定义响应体

```rust
use exum::fast_builder::*;

#[get("/api/json")]
async fn get_json() -> impl IntoResponse {
    let data = serde_json::json!({ "message": "成功", "code": 200 });
    response_ok(data.to_string())  // 返回JSON数据
}

#[get("/api/html")]
async fn get_html() -> impl IntoResponse {
    let html = r#"<!DOCTYPE html><html><body><h1>Hello World</h1></body></html>"#;
    response_ok(html)  // 返回HTML内容
}
```

### 结合错误处理

```rust
use exum::fast_builder::*;

#[get("/api/user/:id")]
async fn get_user(id: String) -> impl IntoResponse {
    match find_user_by_id(&id).await {
        Some(user) => response_ok(serde_json::to_string(&user).unwrap()),
        None => reponse_not_found(),
    }
}

#[post("/api/upload")]
async fn upload_file() -> impl IntoResponse {
    match process_upload().await {
        Ok(_) => response_ok("文件上传成功"),
        Err(e) => {
            tracing::error!("上传失败: {}", e);
            internal_server_error()
        }
    }
}
```

## CORS配置

### 使用预置的CORS配置

```rust
use exum::fast_builder::*;
use exum::layers::StaticFileServiceBuilder;

#[tokio::main]
async fn main() {
    let static_service = StaticFileServiceBuilder::new("./public")
        .with_cors(cors_any())  // 使用预置的全允许CORS
        .build_router("/static");
    
    let mut app = Application::build(ApplicationConfig::default());
    app.merge(static_service);
    
    app.run().await;
}
```

### 自定义CORS配置

```rust
use exum::fast_builder::*;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(["http://localhost:3000".parse().unwrap()])
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_headers([axum::http::header::CONTENT_TYPE]);
    
    let mut app = Application::build(ApplicationConfig::default());
    app.app = app.app.layer(cors);
    
    app.run().await;
}
```



## 最佳实践

1. **统一错误处理** - 使用标准响应函数确保错误响应的一致性
2. **适当的状态码** - 根据操作结果选择正确的HTTP状态码
3. **结合tracing** - 在错误响应时记录适当的日志信息
4. **考虑安全性** - 在生产环境中谨慎使用 `cors_any()`

## 与标准axum响应的对比

### 传统方式
```rust
use axum::{http::StatusCode, response::IntoResponse};

#[get("/api/data")]
async fn get_data() -> impl IntoResponse {
    (StatusCode::OK, "数据获取成功")
}
```

### 使用fast_builder
```rust
use exum::fast_builder::*;

#[get("/api/data")]
async fn get_data() -> impl IntoResponse {
    response_ok("数据获取成功")  // 更简洁直观
}
```

`fast_builder` 提供了更简洁、更直观的API，特别适合快速构建标准的HTTP响应。