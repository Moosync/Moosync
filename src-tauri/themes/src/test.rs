use std::{fs, path::PathBuf, str::FromStr, sync::mpsc::channel};

use types::errors::Result;

use crate::themes::{transform_css, ThemeHolder};

#[test]
fn test_transformcss() -> Result<()> {
    let (tx, _) = channel();
    let theme_holder = ThemeHolder::new(PathBuf::new(), PathBuf::new(), tx);

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
