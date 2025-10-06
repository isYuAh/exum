# 静态文件服务

Exum 提供了强大的静态文件服务，基于 `StaticFileServiceBuilder` 构建，支持多种高级特性。

## 功能特性

### 🌐 跨域支持
- **CORS配置** - 支持完整的CORS跨域配置
- **预置CORS策略** - 提供 `cors_any()` 方法快速启用全允许CORS

### 📱 SPA支持
- **SPA回退** - 支持单页应用的回退机制，未找到文件时返回 `index.html`
- **智能路由** - 适合Vue、React等前端框架的单页应用部署

### 📊 文件处理
- **流式传输** - 使用 `tokio_util::io::ReaderStream` 实现高效的文件流传输
- **MIME类型推测** - 自动根据文件扩展名推测正确的MIME类型
- **UTF-8字符集** - 对文本文件自动添加 `charset=utf-8` 字符集
- **Last-Modified头** - 自动添加文件修改时间，支持浏览器缓存
- **HEAD请求支持** - 正确处理HEAD请求，返回文件元信息

### 🔧 错误处理
- **智能404处理** - 文件不存在时返回适当的404响应
- **错误日志** - 使用tracing记录服务错误信息

## 使用 StaticFileServiceBuilder

### 基础用法

```rust
use exum::layers::StaticFileServiceBuilder;

#[tokio::main]
async fn main() {
    let static_service = StaticFileServiceBuilder::new("./public")
        .build_router("/static");
    
    let mut app = Application::build(ApplicationConfig::default());
    app.merge(static_service);
    
    app.run().await;
}
```

### 启用SPA回退

```rust
use exum::layers::StaticFileServiceBuilder;

#[tokio::main]
async fn main() {
    let static_service = StaticFileServiceBuilder::new("./public")
        .with_spa_fallback(true)  // 启用SPA回退
        .build_router("/static");
    
    let mut app = Application::build(ApplicationConfig::default());
    app.merge(static_service);
    
    app.run().await;
}
```

### 启用CORS支持

```rust
use exum::layers::StaticFileServiceBuilder;

#[tokio::main]
async fn main() {
    let static_service = StaticFileServiceBuilder::new("./public")
        .cors_any()  // 启用全允许CORS
        .build_router("/static");
    
    let mut app = Application::build(ApplicationConfig::default());
    app.merge(static_service);
    
    app.run().await;
}
```

### 完整配置示例

```rust
use exum::layers::StaticFileServiceBuilder;

#[tokio::main]
async fn main() {
    let static_service = StaticFileServiceBuilder::new("./public")
        .with_spa_fallback(true)  // 启用SPA回退
        .cors_any()               // 启用CORS跨域支持
        .build_router("/assets");
    
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



## 最佳实践

1. **优先使用StaticFileServiceBuilder** - 相比基础的 `static_` 方法，提供更多高级功能
2. **生产环境启用SPA回退** - 适合部署单页应用
3. **开发环境启用CORS** - 方便前后端分离开发
4. **使用相对路径** - 服务会自动解析为绝对路径，确保安全性