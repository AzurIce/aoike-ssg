## 实体定义

### Article(name)

一个 ARTICLE 可以是一个单文件的形式（`<name>.(typ,md)`），或者是一个目录的形式（`<name>/main.(typ,md)`）。

`Article("index")` 为特殊的 ARTICLE。

## Note(name)

一个 NOTE 为一个包含 INDEX_ARTICLE 的目录 `<name>/Article("index")`，它以及其非 ARTICLE 目录的子目录中有若干个 ARTICLE。

## 目录结构

每一个实体都有一个 `<identifier>`。

 ```sh
aoike_vault/
├── posts/
│   ├── Article("index") # posts/index
│   ├── Article("name1") # posts/name1
│   ├── Article("name2") # posts/name2
│   └── ...
├── notes/
│   ├── Article("index") # notes/index
│   ├── Note("name1") # notes/name1
│   ├── Note("name2") # notes/name2
│   └── ...
└── ...
```

```sh
Note("name1") # notes/name1
├── Article("index") # notes/name1/index
├── subfolder/
├── ├── Article("name1") # notes/name1/subfolder/name1
├── ├── Article("name2") # notes/name1/subfolder/name2
├── └── ...
├── Article("name3") # notes/name1/name3
├── Article("name4") # notes/name1/name4
└── ...
```

## 文件名 & 标题 & Slug

首先，为了更符合 URL 的标准，应该让生成出的 json 文件只包含 ASCII 符号。
然而，为了原始文件的组织，原始文件名应该允许更多类型的字符。

标题是内容的一部分，不应该被“卷入”这个问题。

因此：原始文件名 -> slugify -> json 文件名。（同时也是对应实体的 identifier）
