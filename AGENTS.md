# AGENTS.md — aoike 项目指南

## 项目概述

aoike 是一个"内容即数据，前端即应用"的静态站点生成器。核心库定义数据协议（类型 + JSON 输出格式），后端将内容源编译到该协议，前端运行时消费 JSON 数据。

## Workspace 结构

```
aoike/
├── src/                        # 核心库（协议类型 + 默认构建后端）
│   ├── data.rs                 # 协议层：所有 JSON 导出类型
│   ├── lib.rs                  # 内部类型（Article, Section, Vault）
│   └── build/                  # 默认后端（feature = "build"）
│       ├── mod.rs              # build_vault, export_vault
│       ├── article.rs          # Markdown/Typst 解析
│       ├── cli.rs              # CLI 入口
│       └── utils.rs            # HTML 处理、Git 时间戳
├── packages/
│   ├── aoike-leptos/           # Leptos CSR 前端
│   └── aoike-obsidian/         # Obsidian vault 后端
├── examples/
│   └── leptos/                 # Leptos 示例应用
└── docs/                       # 项目文档
```

## 关键约定

### ID Slugify 规则
所有实体 ID 通过 `slug::slugify()` 生成。例如 `"Test Article"` → `"test-article"`，`"牛🐮逼"` → `"niu-cow-bi"`。

### 时间戳格式
协议层使用 Unix 时间戳（`i64`，秒级）。内部类型使用 `time::UtcDateTime`。

### Feature Flags
- `build`（默认启用）：包含构建管线依赖（walkdir, pulldown-cmark, rayon 等）
- 不启用 `build` 时，仅暴露数据类型，适合前端或第三方后端使用

### EntityPath
每个实体有两个路径：
- `ids: Vec<String>` — slugified ID 链，用于 URL 生成（如 `["posts", "my-article"]`）
- `rel_path: String` — 相对于 vault 根目录的源文件路径

## 构建与运行

```bash
# 构建核心库
cargo build

# 构建整个 workspace
cargo build --workspace

# 运行默认后端导出 vault
cargo run -- <VAULT_DIR> -o <OUTPUT_DIR>

# 示例：导出到 leptos 示例的 static 目录
cargo run -- examples/doc-src -o examples/leptos/static/vault
```

## 文档索引

- [docs/architecture.md](docs/architecture.md) — 三层解耦架构
- [docs/data-model.md](docs/data-model.md) — 协议类型规范
- [docs/output-format.md](docs/output-format.md) — 输出目录结构与文件命名
- [docs/backend-guide.md](docs/backend-guide.md) — 如何编写新后端
