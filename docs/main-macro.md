# #[main] 宏

`#[main]` 宏是 Exum 框架的核心功能之一，它自动处理应用程序的初始化和运行逻辑，让开发者可以专注于业务代码。

## 基本用法

### 最简单的使用方式

```rust
#[main]
async fn main() {
    // 这里可以添加自定义逻辑
    println!("服务器启动中...");
}
```

### 使用配置文件

```rust
#[main(config = "config.toml")]
async fn main() {
    // 这里可以添加自定义逻辑
    println!("使用配置文件启动服务器...");
}
```

## 自动注入的功能

`#[main]` 宏会自动注入以下代码：

### 1. 自动添加 `#[tokio::main]`
- 自动为 `main` 函数添加 `#[tokio::main]` 属性
- 无需手动处理异步运行时配置

### 2. 自动加载配置
- 如果没有指定配置文件，使用 `ApplicationConfig::load()` 自动加载默认配置
- 如果指定了配置文件路径，使用 `ApplicationConfig::from_file(path)` 加载指定配置

> 详细配置加载逻辑请参考：[配置管理文档](./configuration.md)

### 3. 自动构建 Application
- 自动创建 `Application::build(config)` 调用
- 配置对象会自动传递给构建器

### 4. 自动运行应用
- 自动添加 `app.run().await` 调用
- 确保应用程序正确启动和运行

## 与 `app_chain!` 宏的配合使用

`#[main]` 宏内部可以配合 `app_chain!` 宏来构建复杂的应用链：

```rust
#[main]
async fn main() {
    app_chain!(app, {
        route("/custom", axum::routing::get(|| async { "Custom route" })),
        nest("/api", axum::Router::new().route("/users", axum::routing::get(|| async { "Users API" }))),
        layer(tower::ServiceBuilder::new()
            .layer(tower_http::trace::TraceLayer::new_for_http()))
    });
}
```

`app_chain!` 宏的语法是逗号分隔的方法调用列表，每个方法调用都会转换为 `app.app = app.app.方法名(...)` 的形式。

## 展开后的代码

`#[main]` 宏展开后的代码相当于：

```rust
#[tokio::main]
async fn main() {
    let _CONFIG = ApplicationConfig::load(); // 或 from_file()
    let mut app = Application::build(_CONFIG);
    {
        // 用户自定义的代码块
        println!("服务器启动中...");
    }
    app.run().await;
}
```

## 配置文件的自动发现

当使用 `#[main]` 宏时，框架会自动尝试从以下位置加载配置文件：

1. 如果指定了 `config = "路径"`，则从指定路径加载
2. 如果没有指定，按以下顺序加载：
   - 首先加载 `./config.toml` 作为基础配置
   - 然后根据当前环境（如 `prod`、`dev`）加载对应的环境配置文件 `./config.{env}.toml`
   - 环境配置文件会覆盖基础配置中的相同字段
   - 最后注入环境变量中的配置值
3. 如果都没有找到，使用默认配置

## 优势

使用 `#[main]` 宏的优势：

1. **简化启动代码**：无需重复编写启动逻辑
2. **统一配置管理**：自动处理配置加载
3. **减少错误**：避免忘记添加必要的启动代码
4. **更好的可维护性**：启动逻辑集中管理

## 注意事项

- `#[main]` 宏只能用于 `main` 函数
- 函数必须是 `async` 的
- 宏会替换整个函数体，但保留用户自定义的代码块