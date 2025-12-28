fn main() {
    println!("cargo:rerun-if-changed=../doc-src");

    aoike_sycamore::build::init_aoike_sycamore();

    let vault = aoike::build::build_vault("../doc-src");
    aoike::build::export_vault(&vault, "static/vault");
}
