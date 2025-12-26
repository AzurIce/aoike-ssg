#[cfg(feature = "build")]
mod tests {
    use aoike::build::{build_vault, export_vault};
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    fn create_file(dir: &std::path::Path, name: &str, content: &str) {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut file = File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_build_vault_structure() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create structure:
        // posts/
        //   hello.md
        //   tech/
        //     rust.md
        // notes/
        //   math/
        //     algebra.md
        //     main.md (Directory Article)

        create_file(
            root,
            "posts/hello.md",
            "---\ntitle: Hello\n---\n# Hello World",
        );
        create_file(
            root,
            "posts/tech/rust.md",
            "---\ntitle: Rust\n---\n# Rust is great",
        );

        create_file(
            root,
            "notes/math/algebra.md",
            "---\ntitle: Algebra\n---\n# Algebra",
        );
        create_file(
            root,
            "notes/math/main.md",
            "---\ntitle: Math Root\n---\n# Math Root",
        );

        // Ensure directories exist even if empty (though create_file handles parents)
        fs::create_dir_all(root.join("posts")).unwrap();
        fs::create_dir_all(root.join("notes")).unwrap();

        let vault = build_vault(root);

        // Check posts (flattened)
        // posts/hello.md -> id: hello, ids: [posts, hello]
        // posts/tech/rust.md -> id: rust, ids: [posts, tech, rust]

        let posts = vault.posts.articles().collect::<Vec<_>>();
        assert_eq!(posts.len(), 2, "Should have 2 posts");

        let hello = posts
            .iter()
            .find(|p| p.entity_path.id().0 == "hello")
            .expect("hello post not found");
        assert_eq!(hello.title, "Hello World");
        assert_eq!(hello.entity_path.ids.len(), 2);
        assert_eq!(hello.entity_path.ids[0].0, "posts");
        assert_eq!(hello.entity_path.ids[1].0, "hello");

        let rust = posts
            .iter()
            .find(|p| p.entity_path.id().0 == "rust")
            .expect("rust post not found");
        assert_eq!(rust.title, "Rust is great");
        assert_eq!(rust.entity_path.ids.len(), 3);
        assert_eq!(rust.entity_path.ids[0].0, "posts");
        assert_eq!(rust.entity_path.ids[1].0, "tech");
        assert_eq!(rust.entity_path.ids[2].0, "rust");

        // Check notes (hierarchical)
        // notes/math (Directory Article because of main.md)
        // notes/math/algebra.md (Child of math? No, ignored because math is atomic)

        let notes_children = &vault.notes.children;

        let math_node = notes_children
            .iter()
            .find(|n| n.id().0 == "math")
            .expect("math node not found");
        assert!(
            math_node.as_article().is_some(),
            "math should be an article because it has main.md"
        );
        assert!(math_node.as_container().is_none());

        let math_article = math_node.as_article().unwrap();
        assert_eq!(math_article.title, "Math Root");
    }

    #[test]
    fn test_vault_export() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let out_dir = temp_dir.path().join("dist");

        create_file(
            root,
            "posts/hello.md",
            "# Hello World\n\nSummary content here.",
        );
        create_file(root, "notes/math/main.md", "# Math Root\n\nMath content.");

        fs::create_dir_all(root.join("posts")).unwrap();
        fs::create_dir_all(root.join("notes")).unwrap();

        let vault = build_vault(root);
        export_vault(&vault, &out_dir);

        // Check vault.json
        let vault_json_path = out_dir.join("vault.json");
        assert!(vault_json_path.exists());
        let vault_json = fs::read_to_string(vault_json_path).unwrap();
        let vault_data: aoike::data::VaultData = serde_json::from_str(&vault_json).unwrap();

        assert_eq!(vault_data.posts.len(), 1);
        assert_eq!(vault_data.posts[0].title, "Hello World");
        assert_eq!(vault_data.posts[0].id, "hello");
        assert_eq!(vault_data.posts[0].ids, vec!["posts", "hello"]);

        assert_eq!(vault_data.notes.len(), 1);
        assert_eq!(vault_data.notes[0].title, "Math Root");
        assert_eq!(vault_data.notes[0].id, "math");
        assert_eq!(vault_data.notes[0].ids, vec!["notes", "math"]);

        // Check article JSON
        // posts/hello -> articles/posts/hello.json
        let hello_json_path = out_dir.join("articles/posts/hello.json");
        assert!(hello_json_path.exists());
        let hello_json = fs::read_to_string(hello_json_path).unwrap();
        let hello_detail: aoike::data::ArticleDetail = serde_json::from_str(&hello_json).unwrap();
        assert_eq!(hello_detail.meta.title, "Hello World");
        assert!(hello_detail.content.contains("Summary content here."));

        // notes/math -> articles/notes/math.json
        let math_json_path = out_dir.join("articles/notes/math.json");
        assert!(math_json_path.exists());
        let math_json = fs::read_to_string(math_json_path).unwrap();
        let math_detail: aoike::data::ArticleDetail = serde_json::from_str(&math_json).unwrap();
        assert_eq!(math_detail.meta.title, "Math Root");
        assert!(math_detail.content.contains("Math content."));
    }
}
