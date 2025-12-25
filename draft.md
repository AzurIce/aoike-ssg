## 实体定义

### Article

Article 是内容的基本单元。它可以来源于单文件，也可以来源于目录。

1.  **单文件 Article**: `path/to/<name>.(md|typ)`
    -   **ID**: `<name>` (文件名，不含扩展名)
    -   **Path**: `path/to` (父目录路径，相对于根)
    -   **条件**: `<name>` 不为 `index` 或 `main`。

2.  **目录 Article (Main)**: `path/to/<name>/main.(md|typ)`
    -   **ID**: `<name>` (父目录名)
    -   **Path**: `path/to` (父目录的父目录路径，相对于根)
    -   **定义**: 任何包含 `main.(md|typ)` 的目录都被视为一个**目录 Article**。
    -   **行为**: 扫描器将该目录视为一个原子实体，**不会**扫描该目录下的其他 Article 或子目录。该目录下的其他文件可作为资源被引用。

3.  **索引 Article (Index)**: `path/to/<name>/index.(md|typ)`
    -   **ID**: `<name>` (父目录名)
    -   **Path**: `path/to` (父目录的父目录路径，相对于根)
    -   **定义**: 位于普通目录（非目录 Article）下的 `index` 文件。
    -   **行为**: 它代表了该目录节点本身的 Article 内容（如章节介绍）。

### Directory (Container)

任何**不包含** `main.(md|typ)` 的目录都被视为**容器目录**（Subdirectory）。

-   容器目录会被递归扫描。
-   容器目录中可以包含：
    -   普通 Article 文件
    -   目录 Article (包含 main 的子目录)
    -   索引 Article (`index` 文件)
    -   更深层的容器目录
-   **变化**: 不再强制要求目录必须包含 `index` 才是有效子目录。任何普通目录都会被遍历。

## 结构逻辑

整个 vault 可以被视为一个 *容器目录*，不过其中有两个约定的目录：
-   `posts` 目录，用于存放文章。
-   `notes` 目录，用于存放笔记。

### Posts

`posts` 目录下的所有 Article（包括普通文件、目录 Article、以及各级容器目录中的索引 Article）都被收集到一个扁平的列表中。
嵌套结构完全通过 Article 的 `path` 属性体现。

### Notes

会保留层级关系。要注意的是第一层级一定是 *容器目录*，在前端展示的时候可能会以第一层级作为不同的 Note。

## 目录结构示例

```sh
aoike_vault/
├── posts/
│   ├── index.md            # Article("posts"), path=[] (Posts 根索引)
│   ├── hello.md            # Article("hello"), path=[]
│   ├── tech/               # 容器目录
│   │   ├── index.md        # Article("tech"), path=[] (tech 节点索引)
│   │   ├── rust.md         # Article("rust"), path=["tech"]
│   │   └── deep/           # 目录 Article (包含 main)
│   │       ├── main.md     # Article("deep"), path=["tech"]
│   │       └── image.png   # 资源文件，不被视为 Article
│   └── misc/               # 容器目录 (无 index)
│       └── random.md       # Article("random"), path=["misc"]
├── notes/
│   ├── index.md            # Article("notes"), path=[]
│   ├── math/               # 容器目录 同时是一个
│   │   ├── index.md        # Article("math")
│   │   ├── algebra.md      # Article("algebra")
│   │   └── calculus/       # 容器目录
│   │       └── limit.md    # Article("limit")
│   └── ...
└── ...
```

## 文件名 & 标题 & Slug

规则保持不变：
原始文件名 -> slugify -> json 文件名 / ID。
