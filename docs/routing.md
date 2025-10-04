# 路由宏

## 支持的HTTP方法

- `#[get(path)]` - GET 请求
- `#[post(path)]` - POST 请求
- `#[put(path)]` - PUT 请求
- `#[delete(path)]` - DELETE 请求
- `#[options(path)]` - OPTIONS 请求
- `#[head(path)]` - HEAD 请求
- `#[trace(path)]` - TRACE 请求
- `#[route(path, method = "METHOD")]` - 自定义方法

## 路径参数

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

## URL编码路径支持

Exum 支持URL编码的路径，这意味着你可以使用中文和其他特殊字符作为路由路径：

```rust
#[get("/api/用户/:id")]
async fn get_user(id: String) -> String {
    format!("获取用户ID: {}", id)
}

#[post("/api/产品/创建")]
async fn create_product() -> &'static str {
    "产品创建成功"
}

#[get("/搜索/:关键词")]
async fn search(keyword: String) -> String {
    format!("搜索关键词: {}", keyword)
}
```

路由宏会自动处理URL编码，使得中文路由能够正常工作。