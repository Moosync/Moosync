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

use crate::error::ThemesError;

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_themes_error_display_and_conversions() {
    let not_found = ThemesError::ThemeNotFound;
    assert_eq!(format!("{}", not_found), "Theme not found");

    let parse_err = ThemesError::ParseThemeFailed;
    assert_eq!(format!("{}", parse_err), "Failed to parse theme");

    let zip_err = ThemesError::Zip("bad zip".to_string());
    assert_eq!(format!("{}", zip_err), "Zip error: bad zip");

    let json_err = serde_json::from_str::<bool>("invalid").unwrap_err();
    let theme_err: ThemesError = json_err.into();
    match theme_err {
        ThemesError::Json(_) => {}
        _ => panic!("Expected ThemesError::Json"),
    }
}
