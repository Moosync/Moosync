// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use assertables::{assert_len_eq_x, assert_ok};
use syn::parse_quote;
use tracing_test::traced_test;

use crate::{
    generate_theme_apply_from_content, generate_theme_impl_from_content,
    generate_theme_ui_helpers_from_content, parse_slint_theme, read_file, strip_comments,
};

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_strip_comments_line_and_block() {
    let raw = r#"
    // Single line comment
    global Theme {
        /* Block comment
           multiline */
        property <color> primary: #ff0000;
    }
    "#;

    let stripped = strip_comments(raw);

    assert!(!stripped.contains("Single line comment"));
    assert!(!stripped.contains("Block comment"));
    assert!(stripped.contains("property <color> primary: #ff0000;"));
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_slint_theme_properties() {
    let content = r#"
    global Theme {
        property <color> primary: #112233;
        property <length> cardWidth: 200px;
        property <string> noColonProp;
        invalid statement here;
        property no_angles;
        property <unclosed;
    }
    "#;

    let props = parse_slint_theme(content);

    assert_len_eq_x!(&props, 3);
    assert_eq!(props[0].0, "primary");
    assert_eq!(props[0].1, "color");
    assert_eq!(props[0].2, "#112233");

    assert_eq!(props[1].0, "cardWidth");
    assert_eq!(props[1].1, "length");
    assert_eq!(props[1].2, "200px");

    assert_eq!(props[2].0, "noColonProp");
    assert_eq!(props[2].1, "string");
    assert_eq!(props[2].2, "");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_slint_theme_missing_theme_block() {
    let content = "no theme block present here";
    let props = parse_slint_theme(content);
    assert!(props.is_empty());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_generate_theme_impl_all_proto_fields() {
    let content = r#"
    global Theme {
        property <color> primary: #111111;
        property <color> secondary: #222222;
        property <color> tertiary: #333333;
        property <color> textPrimary: #444444;
        property <color> textSecondary: #555555;
        property <color> textInverse: #666666;
        property <color> accent: #777777;
        property <color> divider: #888888;
        property <color> customColor: #999999;
    }
    "#;

    let output = generate_theme_impl_from_content(content);
    let output_str = output.to_string();

    assert!(output_str.contains("pub trait ThemeItemExt"));
    assert!(output_str.contains("impl ThemeItemExt for ThemeItem"));
    assert!(output_str.contains("\"primary\" =>"));
    assert!(output_str.contains("\"secondary\" =>"));
    assert!(output_str.contains("\"tertiary\" =>"));
    assert!(output_str.contains("\"textPrimary\" =>"));
    assert!(output_str.contains("\"textSecondary\" =>"));
    assert!(output_str.contains("\"textInverse\" =>"));
    assert!(output_str.contains("\"accent\" =>"));
    assert!(output_str.contains("\"divider\" =>"));
    assert!(output_str.contains("pub fn get_default_theme_item () -> ThemeItem"));
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_generate_theme_apply_from_content() {
    let content = r#"
    global Theme {
        property <color> primary: #111111;
        property <length> margin: 16px;
    }
    "#;
    let theme_global: syn::Expr = parse_quote!(global);
    let name: syn::Expr = parse_quote!(prop_name);
    let set_color: syn::Expr = parse_quote!(set_c);
    let set_length: syn::Expr = parse_quote!(set_l);

    let output =
        generate_theme_apply_from_content(content, &theme_global, &name, &set_color, &set_length);
    let output_str = output.to_string();

    assert!(output_str.contains("\"primary\" => set_c (& | c | global . set_primary (c))"));
    assert!(output_str.contains("\"margin\" => set_l (& | l | global . set_margin (l))"));
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_generate_theme_ui_helpers_from_content() {
    let content = r#"
    global Theme {
        property <color> primary: #111111;
        property <length> marginWithPx: 16px;
        property <length> marginRaw: 24;
    }
    "#;

    let output = generate_theme_ui_helpers_from_content(content);
    let output_str = output.to_string();

    assert!(output_str.contains("pub fn update_theme_constants_ui"));
    assert!(output_str.contains("is_color : true"));
    assert!(output_str.contains("is_color : false"));
    assert!(output_str.contains("\"marginWithPx\""));
    assert!(output_str.contains("\"marginRaw\""));
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_read_file_direct_path() {
    let temp_file = std::env::temp_dir().join("test_theme_read_file.slint");
    std::fs::write(&temp_file, "global Theme {}").expect("write failed");

    let content = read_file(temp_file.to_str().unwrap());
    assert_ok!(&content);
    assert_eq!(content.unwrap(), "global Theme {}");

    let _ = std::fs::remove_file(temp_file);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_theme_apply_input_parse() {
    let raw = r#""path/to/theme.slint", global_theme, prop_name, set_c, set_l"#;
    let parsed: Result<crate::ThemeApplyInput, _> = syn::parse_str(raw);
    assert!(parsed.is_ok());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_find_theme_block_nested_and_unclosed() {
    let nested = "global Theme { nested { val: 1 } property <color> a: #000; }";
    let block = crate::find_theme_block(nested);
    assert!(block.is_some());
    assert!(block.unwrap().contains("nested { val: 1 }"));

    let unclosed = "global Theme { no closing brace";
    let block_unclosed = crate::find_theme_block(unclosed);
    assert!(block_unclosed.is_none());

    let no_brace = "global Theme";
    assert!(crate::find_theme_block(no_brace).is_none());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_find_file_in_ancestors_and_manifest_dir() {
    let temp_root = std::env::temp_dir().join("moosync_theme_ancestor_test");
    let deep_dir = temp_root.join("a").join("b");
    std::fs::create_dir_all(&deep_dir).expect("dir creation failed");
    std::fs::write(temp_root.join("WORKSPACE"), "").expect("write failed");
    std::fs::write(temp_root.join("target.slint"), "theme data").expect("write failed");

    let found = crate::find_file_in_ancestors(deep_dir.clone(), "target.slint");
    assert_eq!(found.as_deref(), Some("theme data"));

    let not_found = crate::find_file_in_ancestors(deep_dir, "missing.slint");
    assert!(not_found.is_none());

    // Test CARGO_MANIFEST_DIR branch in read_file
    let old_manifest = std::env::var_os("CARGO_MANIFEST_DIR");
    unsafe {
        std::env::set_var("CARGO_MANIFEST_DIR", &temp_root);
    }
    let read_res = read_file("target.slint");
    assert_ok!(&read_res);
    assert_eq!(read_res.unwrap(), "theme data");
    unsafe {
        match old_manifest {
            Some(v) => std::env::set_var("CARGO_MANIFEST_DIR", v),
            None => std::env::remove_var("CARGO_MANIFEST_DIR"),
        }
    }

    let _ = std::fs::remove_dir_all(temp_root);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_read_file_current_dir_and_missing() {
    if let Ok(cur) = std::env::current_dir() {
        let p = cur.join("test_temp_theme_macro.txt");
        if std::fs::write(&p, "theme content").is_ok() {
            let res = read_file("test_temp_theme_macro.txt");
            assert_ok!(&res);
            assert_eq!(res.unwrap(), "theme content");
            let _ = std::fs::remove_file(p);
        }
    }

    let missing = read_file("non_existent_file_xyz_123.slint");
    assert!(missing.is_err());
}
