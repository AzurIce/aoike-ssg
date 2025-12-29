fn main() {
    println!("cargo:rerun-if-changed=../doc-src");

    let root = std::path::Path::new("../doc-src")
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from("../doc-src"));
    let vault = aoike::build::build_vault(&root);
    aoike::build::export_vault(&vault, "static/vault", "/");
}
