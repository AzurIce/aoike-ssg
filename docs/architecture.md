# 架构概述

aoike 采用三层解耦架构：

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   后端       │     │  协议层      │     │   前端       │
│  (Backend)   │ ──▶ │  (Protocol)  │ ──▶ │  (Frontend)  │
│              │     │              │     │              │
│ Markdown/    │     │ VaultMeta    │     │ Leptos CSR   │
│ Obsidian/    │     │ ArticleMeta  │     │ fetch JSON   │
│ 任意内容源    │     │ ArticleData  │     │ 渲染页面      │
└─────────────┘     └─────────────┘     └─────────────┘
```

## 协议层（Protocol）

定义在 `src/data.rs`，是整个系统的契约：

- `VaultMeta` — vault.json 的根结构，包含 posts 列表和 notes 树
- `ArticleMeta` — 文章元数据（标题、摘要、时间戳、标签）
- `ArticleData` — 文章详情（元数据 + 完整 HTML 内容 + 链接关系）
- `SectionMeta` — 目录节点（子节点树 + 可选索引文章）
- `EntityPath` — 实体的 ID 链和相对路径

协议层不依赖任何构建工具，仅需 `serde` 和 `serde_json`。

## 后端（Backend）

后端负责将内容源转换为协议类型并导出 JSON 文件。

### 默认后端（`src/build/`）
- 扫描本地目录，支持 Markdown 和 Typst
- 使用 Git 历史获取创建/更新时间
- 通过 `build` feature flag 启用

### aoike-obsidian
- 独立 crate，专门处理 Obsidian vault
- 支持 `[[wikilink]]` 解析、YAML frontmatter、aoike.toml 配置

后端的唯一职责：产出符合协议的 JSON 文件和资源文件。

## 前端（Frontend）

### aoike-leptos
- Leptos 0.8 CSR 应用
- 运行时通过 HTTP 获取 vault.json 和各文章 JSON
- 提供 posts 列表、notes 树形导航、文章阅读、Giscus 评论

前端不关心内容如何产生，只消费协议定义的 JSON 数据。

## 数据流

```
内容源文件 → 后端解析 → 内部类型 (Article/Section/Vault)
                              ↓
                        export() / to_meta() / to_detail()
                              ↓
                        协议类型 (data::*)
                              ↓
                        序列化为 JSON 文件
                              ↓
                        前端 fetch → 渲染
```
