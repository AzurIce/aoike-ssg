# 数据模型规范

所有类型定义在 `src/data.rs`，使用 `serde` 序列化为 JSON。

## VaultMeta

导出为 `vault.json`，是整个 vault 的入口。

```json
{
  "version": "0.2.0",
  "posts": [ /* ArticleMeta[] */ ],
  "notes": [ /* SectionMeta[] */ ]
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `version` | `String` | 协议版本号 |
| `posts` | `ArticleMeta[]` | 扁平文章列表，按创建时间降序 |
| `notes` | `SectionMeta[]` | 笔记树的根节点列表 |

## ArticleMeta

文章元数据，用于列表展示和导航。

```json
{
  "entity_path": { "ids": ["posts", "my-article"], "rel_path": "posts/my-article.md" },
  "title": "My Article",
  "summary": "<p>Article summary...</p>",
  "created": 1700000000,
  "updated": 1700100000,
  "tags": ["rust", "web"],
  "extra": { "custom_field": "value" }
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `entity_path` | `EntityPath` | 是 | 实体路径标识 |
| `title` | `String` | 是 | 文章标题 |
| `summary` | `String` | 是 | HTML 摘要（约 200 字符） |
| `created` | `i64` | 是 | 创建时间（Unix 时间戳，秒） |
| `updated` | `i64` | 是 | 更新时间（Unix 时间戳，秒） |
| `tags` | `String[]` | 否 | 标签列表，默认空 |
| `extra` | `Value?` | 否 | 任意 JSON，用于扩展元数据 |

## ArticleData

文章详情，导出为 `articles/{ids_path}.json`。

```json
{
  "entity_path": { ... },
  "title": "My Article",
  "summary": "...",
  "created": 1700000000,
  "updated": 1700100000,
  "tags": [],
  "content": "<p>Full HTML content...</p>",
  "outlinks": ["notes/linked-page"],
  "backlinks": ["posts/another-article"]
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| *(ArticleMeta 的所有字段，flatten)* | | | |
| `content` | `String` | 是 | 完整 HTML 内容 |
| `outlinks` | `String[]` | 否 | 出链（目标文章的 ids_path） |
| `backlinks` | `String[]` | 否 | 反链（引用本文的文章 ids_path） |

## SectionMeta

目录节点，表示内容树中的一个分组。

```json
{
  "entity_path": { "ids": ["notes", "rust"], "rel_path": "notes/rust" },
  "title": "Rust",
  "children": [ /* NodeMeta[] */ ],
  "index": null,
  "description": "Rust 学习笔记"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `entity_path` | `EntityPath` | 是 | 实体路径标识 |
| `title` | `String` | 是 | 节标题（优先取 index 文章标题） |
| `children` | `NodeMeta[]` | 是 | 子节点列表 |
| `index` | `ArticleMeta?` | 否 | 索引文章（index.md） |
| `description` | `String?` | 否 | 节描述 |

## NodeMeta

内容树节点，tagged union。

```json
{ "Section": { ... } }
// 或
{ "Article": { ... } }
```

## EntityPath

实体的路径标识。

```json
{
  "ids": ["posts", "my-article"],
  "rel_path": "posts/my-article.md"
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `ids` | `String[]` | slugified ID 链，用于 URL 构建 |
| `rel_path` | `String` | 相对于 vault 根的源文件路径 |

`ids_path()` 方法将 ids 用 `/` 连接，如 `"posts/my-article"`。
