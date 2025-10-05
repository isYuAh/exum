# Controller 宏

`#[controller]` 宏是 Exum 提供的一个高级特性，用于为整个控制器类设置统一的前缀路径。**使用前请务必仔细阅读本文档！**

## ⚠️ 重要警告

**这是一个"黑魔法"级别的宏，会破坏 Rust 的正常语法规则！**

`#[controller]` 宏会：
- 修改 `impl` 块内的属性宏
- 最终移除 `impl` 块本身
- 无法使用 `self` 和函数间的相互访问
- 无法使用任何 impl 块内部的特性

**请仅在理解其工作原理的情况下使用！**

## 基本用法

```rust
use exum::*;

// 为整个控制器设置前缀路径
#[controller("/files")]
impl FileController {
    #[route(path="/123")]
    async fn list(&self) -> String {
        format!("file list")
    }
    
    #[get("/hello")]
    async fn hello() {
        "hello"
    }
}

#[main]
async fn main() {
    // 应用会自动注册路由
}
```

## 工作原理

### 宏展开过程

1. **解析阶段**：宏读取 `impl` 块中的所有函数
2. **属性修改**：为每个路由宏添加前缀路径
3. **块消除**：移除 `impl` 块，只保留函数定义

### 展开后的代码

上面的示例代码实际上会被展开为：

```rust
#[route(path="/files/123")]
async fn list(&self) -> String {
    format!("file list")
}

#[get("/files/hello")]
async fn hello() {
    "hello"
}
```

## 为什么需要这种设计？

### 技术限制

1. **属性宏的限制**：需要读取 `impl` 块内部的项目，`macro_rules!` 无法实现
2. **代码生成需求**：路由宏需要生成 `const` 和 `inventory` 代码
3. **路径前缀处理**：需要统一为所有路由方法添加前缀

### 设计权衡

虽然这种设计破坏了正常的 Rust 语法体验，但它是目前唯一可行的方案：
- ✅ 实现了控制器级别的路径前缀
- ✅ 保持了路由宏的简洁性
- ❌ 牺牲了 `impl` 块的正常功能

## 使用限制

### 语法限制

- ❌ 不能使用 `self` 访问其他方法
- ❌ 不能定义关联函数（`fn method() -> Self`）
- ❌ 不能使用 `impl` 块的其他特性

### 替代方案

如果不需要路径前缀功能，建议使用传统的独立函数定义：

```rust
#[get("/files/list")]
async fn list_files() -> String {
    format!("file list")
}

#[get("/files/hello")]
async fn hello_files() {
    "hello"
}
```

## 最佳实践

### 适用场景

- 需要为多个相关路由设置统一前缀
- 逻辑上属于同一个控制器的功能
- 不依赖 `impl` 块内部交互的场景

### 不适用场景

- 需要方法间相互调用的复杂逻辑
- 依赖 `impl` 块特性的场景
- 对 IDE 支持要求高的项目

## 技术细节

### 宏实现位置

`#[controller]` 宏实现在 `exum_macros` crate 中：
- 文件：`exum_macros/src/lib.rs`
- 函数：`controller` proc macro

### 路径处理逻辑

宏会自动处理路径拼接：
- 移除重复的 `/` 字符
- 正确处理空路径和根路径
- 支持所有标准路由宏

## 总结

`#[controller]` 宏是一个强大的工具，但也是一个"危险"的工具。**请确保在使用前完全理解其工作原理和限制**。

如果遇到问题，请优先考虑使用传统的独立函数定义方式。