# 输出格式规范

后端导出的目录结构和文件命名规则。

## 目录结构

```
<output_dir>/
├── vault.json                          # VaultMeta
└── articles/
    ├── index.json                      # 根 index 文章（可选）
    ├── posts/
    │   ├── my-article.json             # ArticleData
    │   ├── my-article/
    │   │   └── image.png               # 文章引用的资源
    │   └── another-post.json
    └── notes/
        ├── rust/
        │   ├── index.json              # Section 的 index 文章
        │   ├── ownership.json
        │   └── ownership/
        │       └── diagram.svg
        └── web/
            └── leptos.json
```

## 文件命名规则

### vault.json
- 位于输出目录根
- 包含 `VaultMeta` 的完整 JSON

### 文章 JSON
- 路径：`articles/{ids_path}.json`
- `ids_path` 由 EntityPath 的 ids 用 `/` 连接
- 例：ids 为 `["posts", "my-article"]` → `articles/posts/my-article.json`

### 资源文件
- 路径：`articles/{ids_path}/{filename}`
- 与文章 JSON 同级目录下的子目录
- HTML 中的相对链接会被重写为 `{public_url_prefix}/static/vault/articles/{ids_path}/{filename}`

## 资源处理规则

1. 构建时扫描文章 HTML 中的 `src="..."` 属性
2. 排除 `data:` URI（内联数据）
3. 将相对路径解析为绝对路径
4. 复制资源文件到输出目录对应位置
5. 重写 HTML 中的链接为公开 URL

## 清理策略

导出完成后，输出目录中不在本次生成文件集合中的 `.json` 文件会被自动删除，防止残留过期内容。资源文件（非 JSON）不会被自动清理。
