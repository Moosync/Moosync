use std::{env, fs, path::PathBuf};

fn main() {
    slint_build::compile("src/app.slint").unwrap();

    let out_dir = env::var("OUT_DIR").unwrap();
    let app_rs_path = PathBuf::from(out_dir).join("app.rs");
    let content = fs::read_to_string(&app_rs_path).unwrap();

    let mut new_content = String::new();
    let mut last_idx = 0;

    let target = "/ui/slint/src/";
    while let Some(pos) = content[last_idx..].find(target) {
        let absolute_pos = last_idx + pos;
        let prefix = &content[last_idx..absolute_pos];
        if let Some(quote_pos) = prefix.rfind('"') {
            let quote_abs_pos = last_idx + quote_pos;
            new_content.push_str(&content[last_idx..quote_abs_pos + 1]);
            new_content.push_str("../../../../../../");
            last_idx = absolute_pos + 1; // Skip the leading slash, continue from "ui/slint/src/"
        } else {
            new_content.push_str(&content[last_idx..absolute_pos + 1]);
            last_idx = absolute_pos + 1;
        }
    }
    new_content.push_str(&content[last_idx..]);

    fs::write(&app_rs_path, new_content).unwrap();
}
