#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    flutter_rust_bridge::setup_default_user_utils();
}

pub fn greet(name: String) -> String {
    format!("Hello, {}! This is a greeting from pure Rust FFI built by Bazel!", name)
}
