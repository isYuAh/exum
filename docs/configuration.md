# 配置

## 手动配置

```rust
use exum::config::ApplicationConfig;

let config = ApplicationConfig {
    addr: [127, 0, 0, 1],
    port: 3000,
};

let app = Application::build(config);
```

## 从配置文件加载

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

## 环境特定配置

Exum 支持环境特定的配置文件覆盖。系统会自动检测当前环境并加载对应的配置文件：

### 环境检测规则
- 如果设置了 `EXUM_ENV` 环境变量，则使用该值
- 否则，在调试模式下使用 `dev`，生产模式下使用 `prod`

### 配置文件加载顺序
1. 首先加载 `config.toml` 作为基础配置
2. 然后加载 `config.{env}.toml` 并覆盖基础配置

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

## 环境变量注入

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