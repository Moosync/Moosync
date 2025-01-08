use flapigen::{JavaConfig, LanguageConfig};
use rifgen::{Generator, Language, TypeCases};
use std::path::Path;
use std::{env, fs};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let in_src = "src/java_glue.rs.in";
    let out_src = Path::new(&out_dir).join("java_glue.rs");
    Generator::new(TypeCases::CamelCase, Language::Java, "src").generate_interface(in_src);
    //delete the lib folder then create it again to prevent obsolete files from staying
    let java_folder = Path::new("../src/main/java/services/players/librespot");
    if java_folder.exists() {
        fs::remove_dir_all(java_folder);
    }
    fs::create_dir(java_folder).unwrap();
    let swig_gen = flapigen::Generator::new(LanguageConfig::JavaConfig(
        JavaConfig::new(java_folder.into(), "services.players.librespot".into())
            .use_null_annotation_from_package("androidx.annotation".into()),
    ))
    .rustfmt_bindings(true);
    swig_gen.expand("android bindings", &in_src, &out_src);
    println!("cargo:rerun-if-changed=src");

    let target_os = env::var("CARGO_CFG_TARGET_OS");
    match target_os.as_ref().map(|x| &**x) {
        Ok("android") => {
            println!("cargo:rustc-link-lib=dylib=stdc++");
            println!("cargo:rustc-link-lib=c++_shared");
        }
        _ => {}
    }
}
