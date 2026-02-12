# 后端开发指南

如何编写一个新的 aoike 后端，将自定义内容源转换为 aoike 协议格式。

## 概述

后端的职责是：
1. 扫描内容源（文件系统、API、数据库等）
2. 解析内容为 HTML
3. 构建 aoike 协议类型（`VaultMeta`, `ArticleMeta`, `ArticleData` 等）
4. 序列化为 JSON 并写入输出目录

## 依赖配置

在 `Cargo.toml` 中引用 aoike 核心库，禁用默认 feature：

```toml
[dependencies]
aoike = { path = "../..", default-features = false }
```

这样只引入数据类型（`aoike::data::*`），不引入构建管线依赖。

## 需要产出的文件

### vault.json
```rust
let vault_meta = aoike::data::VaultMeta {
    version: "0.2.0".to_string(),
    posts: vec![/* ArticleMeta */],
    notes: vec![/* SectionMeta */],
};
let json = serde_json::to_string(&vault_meta)?;
std::fs::write(out_dir.join("vault.json"), json)?;
```

### 文章 JSON
每篇文章导出为 `articles/{ids_path}.json`：
```rust
let article_data = aoike::data::ArticleData {
    meta: aoike::data::ArticleMeta {
        entity_path: aoike::data::EntityPath {
            ids: vec!["posts".into(), "my-article".into()],
            rel_path: "posts/my-article.md".into(),
        },
        title: "My Article".into(),
        summary: "<p>Summary...</p>".into(),
        created: 1700000000,
        updated: 1700100000,
        tags: vec!["rust".into()],
        extra: None,
    },
    content: "<p>Full content...</p>".into(),
    outlinks: vec!["notes/linked-page".into()],
    backlinks: vec![],
};
```

### 资源文件
将文章引用的图片等资源复制到 `articles/{ids_path}/` 目录下，并重写 HTML 中的链接。

## 映射规则

### EntityPath
- `ids`：从 vault 根到实体的 slugified ID 链
- `rel_path`：源文件相对于 vault 根的路径
- ID 通过 `slug::slugify()` 生成

### 时间戳
- 使用 Unix 时间戳（秒级 `i64`）
- 优先使用内容源的元数据（frontmatter、Git 历史）
- 回退到文件系统时间

### 摘要
- HTML 格式，建议截取前 200 字符
- 保持 HTML 标签完整性

### 标签与扩展
- `tags`：字符串数组，空时省略
- `extra`：任意 JSON，用于后端特有的元数据

## Backlinks 处理

1. 第一遍：解析所有文章，收集每篇文章的 outlinks
2. 第二遍：反转 outlinks 映射，为每篇文章填充 backlinks
3. 写入 JSON

## 参考实现

- 默认后端：`src/build/` — Markdown/Typst 文件系统后端
- aoike-obsidian：`packages/aoike-obsidian/` — Obsidian vault 后端
