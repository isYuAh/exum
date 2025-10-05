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

### 路由宏中的自动注入 (`#[dep]`)

**注意：目前 `#[dep]` 属性只支持在路由宏中使用。**

在路由处理函数中使用 `#[dep]` 属性自动注入已定义的状态。

### 手动获取依赖

除了在路由宏中使用 `#[dep]` 属性外，也可以手动获取依赖：

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
async fn get_config(#[dep] config: AppConfig) -> String {
    format!("应用配置: {:?}", config)
}

#[get("/counter")]
async fn get_counter(#[dep] counter: Counter) -> String {
    let current = counter.value.load(std::sync::atomic::Ordering::Relaxed);
    format!("当前计数: {}", current)
}

#[post("/increment")]
async fn increment_counter(#[dep] counter: Counter) -> String {
    let prev = counter.value.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!("计数从 {} 增加到 {}", prev, prev + 1)
}
```

### 注入多个依赖

```rust
use exum::*;

#[get("/status")]
async fn get_status(
    #[dep] config: AppConfig,
    #[dep] counter: Counter,
    #[dep] db: Database,
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

// 路由处理函数
#[get("/")]
async fn index(#[dep] config: AppConfig) -> String {
    format!("欢迎使用 {} 版本应用", config.version)
}

#[get("/counter")]
async fn get_counter(#[dep] counter: Counter) -> String {
    let current = counter.value.load(std::sync::atomic::Ordering::Relaxed);
    format!("当前计数: {}", current)
}

#[post("/increment")]
async fn increment_counter(#[dep] counter: Counter) -> String {
    let prev = counter.value.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!("计数从 {} 增加到 {}", prev, prev + 1)
}

#[get("/status")]
async fn status(
    #[dep] config: AppConfig,
    #[dep] counter: Counter,
    #[dep] db: Database,
) -> String {
    let count = counter.value.load(std::sync::atomic::Ordering::Relaxed);
    format!(
        "应用状态 - 版本: {}, 计数: {}, 数据库: {}",
        config.version, count, db.connection_string
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
- 使用 `#[dep]` 注入的状态参数名必须与状态函数名匹配
- 预热状态会在应用启动时立即初始化，可能影响启动时间
- **目前 `#[dep]` 属性只支持在路由宏中使用**