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

### 手动配置

```rust
use exum::config::ApplicationConfig;

let config = ApplicationConfig {
    addr: [127, 0, 0, 1],
    port: 3000,
};

let app = Application::build(config);
```

### 从配置文件加载

创建 `config.toml` 文件：

```toml
addr = [127, 0, 0, 1]
port = 3000
```

然后在代码中使用：

```rust
use exum::config::ApplicationConfig;

let config = ApplicationConfig::from_file("config.toml");
let app = Application::build(config);
```

### 环境特定配置

Exum 支持环境特定的配置文件覆盖。系统会自动检测当前环境并加载对应的配置文件：

1. **环境检测规则**：
   - 如果设置了 `EXUM_ENV` 环境变量，则使用该值
   - 否则，在调试模式下使用 `dev`，生产模式下使用 `prod`

2. **配置文件加载顺序**：
   - 首先加载 `config.toml` 作为基础配置
   - 然后加载 `config.{env}.toml` 并覆盖基础配置

示例：
```toml
# config.toml (基础配置)
addr = [127, 0, 0, 1]
port = 8080
database_url = "postgres://localhost:5432/mydb"
```

```toml
# config.dev.toml (开发环境配置)
port = 3000
database_url = "postgres://localhost:5432/mydb_dev"
```

```toml
# config.prod.toml (生产环境配置)
addr = [0, 0, 0, 0]
database_url = "${DATABASE_URL}"
```

### 环境变量注入

配置文件支持环境变量注入，使用 `${VARIABLE_NAME}` 语法：

```toml
# config.toml
database_url = "${DATABASE_URL}"
api_key = "${API_KEY}"
port = "${PORT:-8080}"  # 支持默认值
```

在代码中自动加载配置：
```rust
use exum::config::ApplicationConfig;

// 自动加载配置，包含环境变量注入和环境特定配置
let config = ApplicationConfig::load();
let app = Application::build(config);
```

### #[main] 宏

Exum 提供了 `#[main]` 宏来自动注入 Application 的构建和运行代码：

```rust
#[main]
async fn main() {
    // 这里可以添加自定义逻辑
    println!("服务器启动中...");
}

// 或者从配置文件加载配置
#[main(config = "config.toml")]
async fn main() {
    // 这里可以添加自定义逻辑
    println!("使用配置文件启动服务器...");
}
```

`#[main]` 宏会自动注入以下代码：
- `#[tokio::main]` 属性
- `Application::build()` 调用
- `app.run().await` 调用


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
- `app_chain_ext`: 为 `Application` 提供链式调用方法，支持更灵活的路由配置
- `app_chain_ext_full`: 包含 `app_chain_ext` 和静态文件服务功能
- `full`: 包含所有特性

### AppChainExt 特性

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

#### 可用方法

- `.route(path, method)`: 添加单个路由
- `.nest(path, router)`: 嵌套子路由
- `.nest_service(path, service)`: 嵌套服务
- `.merge(router)`: 合并其他路由器

#### 静态文件服务（需要 `app_chain_ext_full` 特性）

```rust
use exum::*;

#[tokio::main]
async fn main() {
    let app = Application::build(ApplicationConfig::default())
        .serve_dir("/static", "./public");
    
    app.run().await;
}
```

## 许可证

MIT License