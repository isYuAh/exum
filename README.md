# Exum

一个轻量级的 Axum 语法糖库，提供更简洁的路由定义语法。

## 特性

- 🚀 简洁的路由宏语法
- 📦 自动参数提取和类型转换
- 🔧 支持多种HTTP方法
- 🎯 路径参数自动解析
- 📝 查询参数和请求体处理
- ⚡ 省略返回值时默认返回 `impl IntoResponse`
- 🌍 环境自动检测和配置覆盖
- 🔧 环境变量注入支持
- 📁 多环境配置文件管理
- 🔗 链式调用支持（AppChainExt）

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
exum = "0.1.0"
```

## 快速开始

```rust
use exum::*;

#[get("/hello/:id")]
async fn hello(id: String, #[q] q: String) -> String {
    format!("id: {}, query: {}", id, q)
}

#[post("/users")]
async fn create_user(#[b] user: User) -> String {
    format!("Created user: {:?}", user)
}

#[tokio::main]
async fn main() {
    let app = Application::build(ApplicationConfig::default());
    app.run().await;
}
```

## 文档

详细的文档请参考以下章节：

- [📖 快速开始](docs/getting-started.md) - 安装和基础使用
- [🛣️ 路由宏](docs/routing.md) - 路由定义和URL编码路径支持
- [📋 参数提取](docs/parameters.md) - 查询参数和请求体处理
- [⚙️ 配置管理](docs/configuration.md) - 环境配置和配置文件
- [📁 静态文件服务](docs/static-files.md) - 静态文件服务和SPA回退
- [🚀 高级功能](docs/advanced.md) - 快速响应构建器、CORS支持等

## Features

- `deref-app`: 为 `Application` 实现 `Deref` trait，可以直接访问底层的 `Router`
- `app_chain_ext`: 为 `Application` 提供链式调用方法，支持更灵活的路由配置
- `app_chain_ext_full`: 包含 `app_chain_ext` 和静态文件服务功能
- `full`: 包含所有特性

## 许可证

MIT License