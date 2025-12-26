# Aoike Architecture & Design Specification

This document outlines the architecture, data models, and build processes for the Aoike static site generator.

## 1. Core Entities

The content model is built around **Articles** and **Containers**.

### 1.1 Article
An Article is the fundamental unit of content. It can be defined in two ways:

1.  **Single-file Article**: `path/to/<name>.(md|typ)`
    -   **ID**: `<name>` (filename without extension).
    -   **Condition**: `<name>` is not `index` or `main`.
    -   **Example**: `posts/hello.md` -> ID: `hello`.

2.  **Directory Article**: `path/to/<name>/main.(md|typ)`
    -   **ID**: `<name>` (the parent directory name).
    -   **Definition**: Any directory containing a `main.(md|typ)` file.
    -   **Behavior**: The scanner treats this directory as an atomic Article. It does **not** recursively scan for other articles inside it. Other files in the directory are treated as assets.
    -   **Example**: `notes/math/main.md` -> The article represents the `math` directory.

3.  **Index Article**: `path/to/<name>/index.(md|typ)`
    -   **Role**: Provides metadata (title, summary, dates) and content for a **Container**.
    -   **Example**: `posts/tech/index.md` -> Provides content for the `tech` category.

### 1.2 Container (Directory)
A **Container** is a directory that holds Articles and sub-Containers.

-   **Definition**: Any directory that does **not** contain a `main.(md|typ)` file.
-   **Behavior**: Recursively scanned for children.
-   **Metadata**: Can optionally contain an `index.(md|typ)` file to define its title and summary. If missing, the directory name is used as the title.

## 2. Vault Structure

The source directory (Vault) is organized into two primary sections with distinct behaviors.

### 2.1 Posts (`posts/`)
-   **Structure**: Flattened list.
-   **Behavior**: All Articles found under `posts/` (at any depth) are collected into a single list.
-   **Sorting**: Sorted by creation date (descending).
-   **Use Case**: Blogs, news feeds, chronological streams.
-   **Hierarchy**: Preserved only in the `ids` path metadata, not in the output structure.

### 2.2 Notes (`notes/`)
-   **Structure**: Hierarchical tree.
-   **Behavior**: Preserves the directory structure of the source.
-   **Sorting**: Sorted by ID (slug) ascending.
-   **Use Case**: Wikis, documentation, knowledge bases.

## 3. Internal Architecture

The Rust implementation separates internal representation from the exported data protocol.

### 3.1 Pathing: `EntityPath`
To unify path handling and ID generation, every entity possesses an `EntityPath`.

```rust
pub struct EntityPath {
    /// The chain of slugified IDs from the vault root to this entity.
    /// e.g., ["posts", "tech", "rust"]
    pub ids: Vec<Id>,

    /// The absolute path to the vault root on disk.
    pub vault_root: PathBuf,

    /// The relative path from the vault root to the source file/directory.
    /// e.g., "posts/tech/rust.md"
    pub path: RelativePathBuf
}
```

### 3.2 Identifiers: `Id`
-   **Type**: `pub struct Id(pub String)`
-   **Value**: The slugified version of the filename or directory name.

### 3.3 Node Types
-   **`Article`**: Contains `EntityPath`, title, HTML content (summary & full), timestamps.
-   **`Container`**: Contains `EntityPath`, optional `index` Article, and a list of child `Node`s.
-   **`Node`**: Enum of `Article | Container`.

## 4. Data Protocol (Export)

The build process generates a static JSON API consumed by the frontend.

### 4.1 Manifest: `vault.json`
A lightweight manifest containing the structure and metadata of the entire site, but **excluding** full content.

```json
{
  "posts": [ ...list of PostMeta... ],
  "notes": [ ...tree of NodeMeta... ]
}
```

### 4.2 Article Details: `articles/**/*.json`
Each article is exported to a separate JSON file containing its full content. The path mirrors the `ids` chain.

-   **Path**: `articles/<id_0>/<id_1>/.../<id_n>.json`
-   **Example**: An article with ids `["posts", "tech", "rust"]` is saved to `articles/posts/tech/rust.json`.

### 4.3 JSON Schemas

**`PostMeta`** (Used in `vault.posts` list)
```rust
pub struct PostMeta {
    pub id: String,          // The entity's own slug
    pub ids: Vec<String>,    // Full chain of slugs [root, ..., id]
    pub path: String,        // Original source path relative to vault
    pub title: String,
    pub summary: String,     // HTML summary
    pub created: i64,        // Unix timestamp
    pub updated: i64,        // Unix timestamp
}
```

**`NodeMeta`** (Used in `vault.notes` tree)
```rust
pub struct NodeMeta {
    pub id: String,
    pub ids: Vec<String>,
    pub path: String,
    pub title: String,
    pub summary: Option<String>,
    pub created: i64,
    pub updated: i64,
    pub children: Vec<NodeMeta>, // Empty for leaf articles
}
```

**`ArticleDetail`** (Used in individual JSON files)
```rust
pub struct ArticleDetail {
    // Flattens PostMeta fields here
    pub id: String,
    pub ids: Vec<String>,
    pub path: String,
    pub title: String,
    pub summary: String,
    pub created: i64,
    pub updated: i64,
    
    // The full content
    pub content: String, 
}
```

## 5. Build Process

1.  **Scan**:
    -   Recursively walk `posts/` to build a flat list of Articles.
    -   Recursively walk `notes/` to build a tree of Containers and Articles.
    -   Construct `EntityPath` for every node, calculating `ids` based on the directory traversal stack.
2.  **Parse**:
    -   Convert Markdown/Typst to HTML.
    -   Extract H1 as title (fallback to filename).
    -   Generate summary (first 200 chars or specific logic).
3.  **Generate**:
    -   Serialize the internal `Vault` structure to `vault.json` (using `PostMeta` and `NodeMeta`).
    -   Serialize every `Article` to its specific `.json` file (using `ArticleDetail`).