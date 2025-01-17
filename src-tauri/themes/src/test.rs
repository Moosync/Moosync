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

use std::{fs, path::PathBuf, str::FromStr, sync::mpsc::channel};

use types::errors::Result;

use crate::themes::{transform_css, ThemeHolder};

#[test]
fn test_transformcss() -> Result<()> {
    let root_theme = PathBuf::from_str("./test.css").unwrap();
    let subroot_theme = PathBuf::from_str("./test1.css").unwrap();
    fs::write(
        root_theme.clone(),
        "@import \"./test1.css;\"\n\n@import \"./test1.css;\"",
    )?;
    fs::write(subroot_theme.clone(), "hello1")?;

    let (res, imports) = transform_css(root_theme.to_string_lossy().to_string(), None)?;

    fs::remove_file(root_theme)?;
    fs::remove_file(subroot_theme)?;
    //
    println!("{:?}", imports);

    if res == "hello1\n\nhello1" {
        return Ok(());
    }
    panic!("Invalid css transformation");
}
