fn main() {
    println!("cargo:rerun-if-changed=../doc-src");

    let vault = aoike::build::build_vault("../doc-src");
    aoike::build::export_vault(&vault, "static/vault");
}
