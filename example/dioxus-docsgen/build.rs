use aoike_dioxus::aoike::Id;
use aoike_dioxus::aoike::build::{Entity, article::ArticleSource, build_vault};

fn main() {
    println!("cargo:rerun-if-changed=doc-src");

    // Build the vault to get posts
    // This assumes doc-src/posts exists and follows the structure
    let vault = build_vault("doc-src");
    // Extract flattened posts from the vault
    let posts = vault
        .posts
        .as_container()
        .expect("posts root must be a container")
        .articles()
        .cloned()
        .collect::<Vec<_>>();

    // Parse index manually
    let index_entity = Entity::new("doc-src/index.md");
    let index_src = ArticleSource::try_from(index_entity).expect("Failed to parse index");
    let index_article = index_src.to_article(Id::new("index".to_string()), vec![]);

    // Convert to Dioxus posts and generate RSX code
    let dioxus_posts: Vec<_> = posts
        .into_iter()
        .map(aoike_dioxus::build::DioxusPost::from)
        .collect();
    let dioxus_index = aoike_dioxus::build::DioxusPost::from(index_article);

    let out_dir = std::env::current_dir().unwrap().join("src");
    let code = aoike_dioxus::build::generate_code(dioxus_posts, dioxus_index);
    std::fs::write(out_dir.join("docsgen.rs"), code).unwrap();
}
