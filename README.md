


          
我来为你创建一个README.md文件，介绍这个Exum库的功能和用法。
        
            
toolName: write_to_file
            
status: success
          
            
filePath: e:\Proj\Rust\exum\README.md
          
content: # Exum

一个轻量级的 Axum 语法糖库，提供更简洁的路由定义语法。

## 特性

- 🚀 简洁的路由宏语法
- 📦 自动参数提取和类型转换
- 🔧 支持多种HTTP方法
- 🎯 路径参数自动解析
- 📝 查询参数和请求体处理
- ⚡ 省略返回值时默认返回 `impl IntoResponse`

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

## 路由宏

### 支持的HTTP方法

- `#[get(path)]` - GET 请求
- `#[post(path)]` - POST 请求
- `#[put(path)]` - PUT 请求
- `#[delete(path)]` - DELETE 请求
- `#[options(path)]` - OPTIONS 请求
- `#[head(path)]` - HEAD 请求
- `#[trace(path)]` - TRACE 请求
- `#[route(path, method = "METHOD")]` - 自定义方法

### 路径参数

路径参数使用 `:` 前缀或 `{}` 语法：

```rust
#[get("/users/:id")]
async fn get_user(id: String) -> String {
    format!("User ID: {}", id)
}

#[get("/posts/{post_id}/comments/{comment_id}")]
async fn get_comment(post_id: String, comment_id: String) -> String {
    format!("Post: {}, Comment: {}", post_id, comment_id)
}
```

## 参数提取

### 查询参数 (`#[q]`)

使用 `#[q]` 属性自动提取查询参数：

```rust
#[get("/search")]
async fn search(#[q] query: String, #[q] page: Option<i32>) -> String {
    format!("Search: {}, Page: {:?}", query, page)
}

// 支持元组模式
#[get("/search2")]
async fn search2(#[q] (query, page): (String, Option<i32>)) -> String {
    format!("Search: {}, Page: {:?}", query, page)
}
```

### 请求体 (`#[b]`)

使用 `#[b]` 属性处理请求体，支持多种格式：

```rust
use serde::Deserialize;

#[derive(Deserialize)]
struct User {
    name: String,
    age: i32,
}

// JSON 格式（默认）
#[post("/users")]
async fn create_user(#[b] user: User) -> String {
    format!("Created user: {:?}", user)
}

// Form 格式
#[post("/users/form")]
async fn create_user_form(#[b(form)] user: User) -> String {
    format!("Created user from form: {:?}", user)
}

// Multipart 格式
#[post("/users/upload")]
async fn upload_user(#[b(multipart)] user: User) -> String {
    format!("Uploaded user: {:?}", user)
}

// 可选参数
#[post("/users/optional")]
async fn create_user_optional(#[b] user: Option<User>) -> String {
    match user {
        Some(u) => format!("Created user: {:?}", u),
        None => "No user provided".to_string(),
    }
}
```

## 配置

```rust
use exum::config::ApplicationConfig;

let config = ApplicationConfig {
    addr: [127, 0, 0, 1],
    port: 3000,
};

let app = Application::build(config);
```

## 完整示例

```rust
use exum::*;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct CreateUserRequest {
    name: String,
    email: String,
}

#[get("/users/:id")]
async fn get_user(id: String) -> String {
    format!("Getting user with ID: {}", id)
}

#[post("/users")]
async fn create_user(#[b] user: CreateUserRequest) -> String {
    format!("Creating user: {:?}", user)
}

#[put("/users/:id")]
async fn update_user(id: String, #[b] user: CreateUserRequest) -> String {
    format!("Updating user {}: {:?}", id, user)
}

#[delete("/users/:id")]
async fn delete_user(id: String) -> String {
    format!("Deleting user with ID: {}", id)
}

#[get("/search")]
async fn search(#[q] query: String, #[q] page: Option<i32>) -> String {
    format!("Searching for '{}' on page {:?}", query, page)
}

#[tokio::main]
async fn main() {
    let app = Application::build(ApplicationConfig::default());
    app.run().await;
}
```

## Features

- `deref-app`: 为 `Application` 实现 `Deref` trait，可以直接访问底层的 `Router`
- `full`: 包含所有特性

## 许可证

MIT License