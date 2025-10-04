# 参数提取

## 查询参数 (`#[q]`)

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

## 请求体 (`#[b]`)

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