# 静态文件服务

Exum 提供了强大的静态文件服务功能，支持SPA（单页应用）回退。

## 使用 StaticFileServiceBuilder

```rust
use exum::layers::StaticFileServiceBuilder;

#[tokio::main]
async fn main() {
    let static_service = StaticFileServiceBuilder::new("./public")
        .with_spa_fallback(true)  // 启用SPA回退
        .cors_any()               // 启用CORS支持
        .build_router("/static");
    
    let mut app = Application::build(ApplicationConfig::default());
    app.merge(static_service);
    
    app.run().await;
}
```

## 基础静态文件服务（需要 app_chain_ext_full 特性）

> **注意**：`static_` 方法使用的是基础的 `tower_http::services::ServeDir` 服务，体验不如 `StaticFileServiceBuilder`，例如不支持自动URL编码路径、SPA回退等高级功能。建议优先使用 `StaticFileServiceBuilder`。

```rust
use exum::*;

#[tokio::main]
async fn main() {
    let mut app = Application::build(ApplicationConfig::default());
    app.static_("/static", "./public");
    
    app.run().await;
}
```

## 高级配置选项

```rust
use exum::layers::StaticFileServiceBuilder;

let static_service = StaticFileServiceBuilder::new("./public")
    .with_spa_fallback(true)  // 启用SPA回退，适合单页应用
    .cors_any()               // 启用CORS跨域支持
    .build_router("/assets");

let mut app = Application::build(ApplicationConfig::default());
app.merge(static_service);
```