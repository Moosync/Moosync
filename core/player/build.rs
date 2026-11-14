use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS");
    if let Ok("android") = target_os.as_ref().map(|x| &**x) {
        println!("cargo:rustc-link-lib=dylib=stdc++");
        println!("cargo:rustc-link-lib=c++_shared");
    }
}
