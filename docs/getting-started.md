# 快速开始

## 安装

使用 `cargo add` 命令添加依赖（推荐）：

```bash
cargo add exum
```

或者手动在 `Cargo.toml` 中添加

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

## ⚠️ 高级特性警告

Exum 提供了一个高级特性 `#[controller]` 宏，**使用前必须阅读文档**！

这个宏会破坏 Rust 的正常语法规则，导致 `impl` 块失去全部特性。虽然它提供了控制器级别的路径前缀功能，但使用时需要特别注意。

**强烈建议在使用前阅读：[Controller 宏文档](./controller-macro.md)**