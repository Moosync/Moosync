use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable is not set");
    let output_file = Path::new(&out_dir)
        .join("../../")
        .join("function_details.json");

    println!("cargo:rerun-if-changed={}", output_file.to_string_lossy());
}
