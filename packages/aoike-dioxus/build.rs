use tracing_subscriber::fmt;

fn main() {
    // Initialize a simple tracing subscriber to handle rsass's tracing events
    // This prevents panics from tracing-indicatif in CI environments
    // try_init() returns an error if a subscriber is already initialized, which is fine
    let _ = fmt()
        .with_target(false)
        .with_level(false)
        .without_time()
        .try_init();

    println!("cargo:rerun-if-changed=assets/main.scss");
    // println!("cargo:rerun-if-changed=tailwind.css");
    // dioxus_tailwindcss::build::npx_tailwindcss("./", "tailwind.css", "assets/tailwind.css").unwrap();
    let format = rsass::output::Format {
        style: rsass::output::Style::Compressed,
        ..Default::default()
    };
    let res = rsass::compile_scss_path("assets/main.scss".as_ref(), format).unwrap();
    std::fs::write("assets/main.css", res).unwrap();
}
