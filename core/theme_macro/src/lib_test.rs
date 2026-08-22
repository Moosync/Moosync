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

use crate::{parse_slint_theme, strip_comments};

#[test]
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
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_slint_theme_properties() {
    let content = r#"
    global Theme {
        property <color> primary: #112233;
        property <length> cardWidth: 200px;
    }
    "#;

    let props = parse_slint_theme(content);
    assert_eq!(props.len(), 2);
    assert_eq!(props[0].0, "primary");
    assert_eq!(props[0].1, "color");
    assert_eq!(props[0].2, "#112233");

    assert_eq!(props[1].0, "cardWidth");
    assert_eq!(props[1].1, "length");
    assert_eq!(props[1].2, "200px");
}
