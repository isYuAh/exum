# 依赖注入

Exum 提供了依赖注入功能，支持全局状态管理和自动依赖注入。

## 使用建议

建议使用 `#[main]` 宏来启动应用，因为依赖注入功能需要正确的初始化顺序。使用 `#[main]` 宏可以确保：
1. 全局状态容器在应用构建前正确初始化
2. 预热状态在应用启动前完成初始化
3. 配置加载和依赖注入的顺序正确

如果手动初始化，需要调用 `init_global_state().await` 来初始化全局状态容器，如果需要预热状态，还需要调用 `global_container().prewarm_all().await`。

## 状态定义 (`#[state]`)

使用 `#[state]` 宏定义全局状态，这些状态会在应用启动时自动初始化。

### 基本用法

```rust
use exum::*;
use std::sync::atomic::AtomicUsize;

#[derive(Debug)]
pub struct AppConfig {
    pub version: String,
    pub debug: bool,
}

#[derive(Debug)]
pub struct Counter {
    pub value: AtomicUsize,
}

// 定义应用配置状态
#[state]
async fn config() -> AppConfig {
    println!("[config] 初始化配置");
    AppConfig {
        version: "v1.0".into(),
        debug: true,
    }
}

// 定义计数器状态
#[state]
async fn counter() -> Counter {
    println!("[counter] 初始化计数器");
    Counter { value: AtomicUsize::new(0) }
}
```

### 预热状态 (`#[state(prewarm)]`)

使用 `prewarm` 参数可以在应用启动时立即启动线程准备初始化状态，而不是直到第一次使用时才初始化。

```rust
use exum::*;

#[derive(Debug)]
pub struct Database {
    pub connection_string: String,
}

// 预热数据库连接，应用启动时立即准备初始化
#[state(prewarm)]
async fn database() -> Database {
    println!("[database] 预热数据库连接");
    Database {
        connection_string: "postgres://localhost:5432/mydb".into(),
    }
}
```

## 依赖注入

### 线程安全机制

Exum 使用 `Mutex` 包装依赖项，确保线程安全。依赖注入现在直接定义为原始类型，实际上是 `MutexGuard<'_, T>` 类型。


### 自动依赖判断规则

Exum 现在会自动判断哪些参数需要依赖注入，无需使用 `#[dep]` 属性。自动判断规则如下：

1. **没有使用其他属性宏**：参数没有使用 `#[b]`、`#[q]` 等属性宏
2. **不在参数路由中**：参数不是路径参数（如 `:id` 或 `{param}`）
3. **不是提取器模式**：参数不是提取器模式，如 `Json(data)`
4. **类型不在排除列表中**：参数类型不在以下排除列表中：

```rust
static NOT_DEPENCENCY_TYPE: &[&str] = &[
    // HTTP 核心类型
    "Method", "Uri", "Version", "HeaderMap",
    // 请求体类型
    "String", "Bytes", "Body",
    // axum 特定类型
    "OriginalUri", "MatchedPath", "RawQuery",
];
```

满足以上条件的参数会自动被当作依赖注入。

### Service 宏 (`#[service]`)

使用 `#[service]` 宏修饰 `impl` 块，自动为服务类提供依赖注入功能：

- 必须包含 `new` 函数
- `new` 函数的参数会被自动依赖注入
- 参数必须是可以被注入的依赖类型

```rust
use exum::*;

// 基础依赖服务
struct DatabaseService {
    connection_string: String,
}

#[service]
impl DatabaseService {
    async fn new() -> Self {
        // 模拟数据库连接初始化
        Self {
            connection_string: "postgres://localhost:5432/mydb".to_string(),
        }
    }
}

// 业务服务，依赖 DatabaseService
#[derive(Debug)]
struct UserService {
    db_service: DatabaseService,
    user_count: usize,
}

#[service]
impl UserService {
    // new 函数的参数会被自动注入
    // DatabaseService 会被框架自动提供
    async fn new(db_service: DatabaseService) -> Self {
        println!("UserService 初始化，使用数据库: {}", db_service.connection_string);
        
        Self {
            db_service,
            user_count: 0, // 模拟用户计数
        }
    }
    
    // 可以添加其他业务方法
    async fn add_user(&mut self) {
        self.user_count += 1;
        println!("添加用户，当前用户数: {}", self.user_count);
    }
}

// 在路由中使用服务
#[get("/users")]
async fn get_users(user_service: UserService) -> String {
    format!(
        "用户服务信息 - 数据库: {}, 当前用户数: {}",
        user_service.db_service.connection_string, user_service.user_count
    )
}

#[post("/users")]
async fn add_user(mut user_service: UserService) -> String {
    // 现在直接使用可变引用，无需处理 Arc
    user_service.add_user().await;
    
    format!("用户添加成功，当前用户数: {}", user_service.user_count)
}
```

### 手动获取依赖

除了自动依赖注入外，也可以手动获取依赖：

```rust
use exum::*;

// 手动获取依赖
async fn some_function() {
    let config = global_container().get::<AppConfig>().await;
    let counter = global_container().get::<Counter>().await;
    
    // 使用获取到的依赖
    println!("版本: {}", config.version);
}
```

手动获取依赖的方式不如路由宏方便，但在某些场景下可能更灵活。

### 基本用法

```rust
use exum::*;

#[get("/config")]
async fn get_config(config: AppConfig) -> String {
    format!("应用配置: {:?}", config)
}

#[get("/counter")]
async fn get_counter(counter: Counter) -> String {
    let current = counter.value.load(std::sync::atomic::Ordering::Relaxed);
    format!("当前计数: {}", current)
}

#[post("/increment")]
async fn increment_counter(counter: Counter) -> String {
    let prev = counter.value.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!("计数从 {} 增加到 {}", prev, prev + 1)
}
```

**注意**：现在依赖注入直接使用原始类型，框架会自动处理 `Mutex` 包装，确保线程安全。参数会自动判断是否需要依赖注入，无需使用 `#[dep]` 属性。

### 注入多个依赖

```rust
use exum::*;

#[get("/status")]
async fn get_status(
    config: AppConfig,
    counter: Counter,
    db: Database,
) -> String {
    let count = counter.value.load(std::sync::atomic::Ordering::Relaxed);
    format!(
        "状态检查 - 版本: {}, 调试: {}, 计数: {}, 数据库: {}",
        config.version, config.debug, count, db.connection_string
    )
}
```

## 完整示例

```rust
use exum::*;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

// 状态定义
#[derive(Debug)]
pub struct AppConfig {
    pub version: String,
}

#[derive(Debug)]
pub struct Counter {
    pub value: AtomicUsize,
}

#[derive(Debug)]
pub struct Database {
    pub connection_string: String,
}

// 服务定义
#[derive(Debug)]
pub struct UserService {
    config: AppConfig,
    db: Database,
}

// 使用 service 宏自动注入依赖
#[service]
impl UserService {
    async fn new(config: AppConfig, db: Database) -> Self {
        Self { config, db }
    }
}

// 预热数据库连接
#[state(prewarm)]
async fn database() -> Database {
    println!("[database] 初始化数据库连接");
    Database {
        connection_string: "postgres://localhost:5432/mydb".into(),
    }
}

// 应用配置
#[state]
async fn config() -> AppConfig {
    println!("[config] 初始化应用配置");
    AppConfig { version: "v1.0".into() }
}

// 计数器
#[state]
async fn counter() -> Counter {
    println!("[counter] 初始化计数器");
    Counter { value: AtomicUsize::new(0) }
}

// 用户服务
#[state]
async fn user_service() -> UserService {
    println!("[user_service] 初始化用户服务");
    // 依赖会自动注入到 UserService::new 中
    UserService::new(
        global_container().get::<AppConfig>().await,
        global_container().get::<Database>().await,
    ).await
}

// 路由处理函数
#[get("/")]
async fn index(config: AppConfig) -> String {
    format!("欢迎使用 {} 版本应用", config.version)
}

#[get("/counter")]
async fn get_counter(counter: Counter) -> String {
    let current = counter.value.load(std::sync::atomic::Ordering::Relaxed);
    format!("当前计数: {}", current)
}

#[post("/increment")]
async fn increment_counter(counter: Counter) -> String {
    let prev = counter.value.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!("计数从 {} 增加到 {}", prev, prev + 1)
}

#[get("/status")]
async fn status(
    config: AppConfig,
    counter: Counter,
    db: Database,
) -> String {
    let count = counter.value.load(std::sync::atomic::Ordering::Relaxed);
    format!(
        "应用状态 - 版本: {}, 计数: {}, 数据库: {}",
        config.version, count, db.connection_string
    )
}

// 使用服务
#[get("/users")]
async fn get_users(user_service: UserService) -> String {
    format!(
        "用户服务 - 版本: {}, 数据库: {}",
        user_service.config.version, user_service.db.connection_string
    )
}

// 使用 #[main] 宏自动处理初始化
#[main]
async fn main() {
    println!("服务器启动完成");
}
```

### 状态注册

当使用 `#[state]` 宏时，状态定义会被自动注册到全局依赖容器中。容器会在应用启动时初始化所有状态。

### 依赖解析

在路由处理函数中使用 `#[dep]` 属性时，框架会自动从全局容器中获取对应的状态实例并注入到函数参数中。

### 生命周期管理

- 所有状态都是单例的，在整个应用生命周期内共享
- 状态初始化是惰性的，除非使用 `prewarm` 参数
- 状态是线程安全的，可以在多个请求中安全使用

## 注意事项

- 状态函数必须是 `async` 的
- 状态类型必须实现 `Send + Sync + 'static`
- 预热状态会在应用启动时立即初始化，可能影响启动时间
- **自动依赖判断规则**：
  - 参数没有使用 `#[b]`、`#[q]` 等属性宏
  - 参数不是路径参数（如 `:id` 或 `{param}`）
  - 参数不是提取器模式，如 `Json(data)`
  - 参数类型不在排除列表中（HTTP核心类型、请求体类型、axum特定类型）
- **依赖注入机制变更**：
  - 现在使用 `Mutex` 包装依赖项，确保线程安全
  - 直接使用原始类型，框架会自动处理 `Mutex` 包装
  - 不再支持之前的 `Arc` 自动解析机制
  - 简化了依赖类型判断逻辑
- **Service 宏注意事项**：
  - 必须包含 `new` 函数
  - `new` 函数的参数必须是可以被注入的依赖类型
  - 参数会被自动依赖注入，无需手动调用 `global_container().get()`