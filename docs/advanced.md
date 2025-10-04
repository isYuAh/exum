# 高级功能

## 快速响应构建器

Exum 提供了便捷的响应构建函数：

```rust
use exum::fast_builder::*;

#[get("/api/data")]
async fn get_data() -> impl IntoResponse {
    response_ok("数据获取成功")
}

#[get("/api/not-found")]
async fn not_found() -> impl IntoResponse {
    reponse_not_found()
}

#[post("/api/bad-request")]
async fn bad_request() -> impl IntoResponse {
    response_bad_request()
}

#[get("/api/error")]
async fn server_error() -> impl IntoResponse {
    internal_server_error()
}

#[post("/api/method-not-allowed")]
async fn method_not_allowed() -> impl IntoResponse {
    response_method_not_allowed()
}
```

## CORS跨域支持

### 使用 StaticFileServiceBuilder 的 CORS 支持

```rust
use exum::layers::StaticFileServiceBuilder;

#[tokio::main]
async fn main() {
    let static_service = StaticFileServiceBuilder::new("./public")
        .cors_any()  // 为静态文件服务启用CORS支持
        .build_router("/static");
    
    let mut app = Application::build(ApplicationConfig::default());
    app.merge(static_service);
    
    app.run().await;
}
```

### 自定义CORS配置

```rust
use exum::*;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(["http://localhost:3000".parse().unwrap()])
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST]);
    
    let mut app = Application::build(ApplicationConfig::default());
    app.app = app.app.layer(cors);
    
    app.run().await;
}
```

## AppChainExt 特性

`app_chain_ext` 特性为 `Application` 提供了链式调用方法，支持更灵活的路由配置：

```rust
use exum::*;

#[get("/hello")]
async fn hello() -> &'static str {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    let app = Application::build(ApplicationConfig::default())
        .route("/custom", axum::routing::get(|| async { "Custom route" }))
        .nest("/api", axum::Router::new().route("/users", axum::routing::get(|| async { "Users API" })))
        .merge(axum::Router::new().route("/extra", axum::routing::get(|| async { "Extra route" })));
    
    app.run().await;
}
```

### 可用方法

- `.route(path, method)`: 添加单个路由
- `.nest(path, router)`: 嵌套子路由
- `.nest_service(path, service)`: 嵌套服务
- `.merge(router)`: 合并其他路由器