# 快速开始

## 安装

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
exum = "0.1.0"
```

## 基础示例

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

// 省略返回值时，默认返回 impl IntoResponse
#[get("/simple")]
async fn simple_handler() {
    "Hello, World!"
}

#[tokio::main]
async fn main() {
    let app = Application::build(ApplicationConfig::default());
    app.run().await;
}
```

## #[main] 宏

Exum 提供了 `#[main]` 宏来自动注入 Application 的构建和运行代码，简化启动流程：

```rust
#[main]
async fn main() {
    // 这里可以添加自定义逻辑
    println!("服务器启动中...");
}
```

`#[main]` 宏会自动处理配置加载、应用构建和运行等繁琐步骤。

> 详细用法请参考：[main 宏文档](./main-macro.md)