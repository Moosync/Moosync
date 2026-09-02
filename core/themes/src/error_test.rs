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

use assertables::assert_matches;
use rstest::rstest;
use tracing_test::traced_test;

use crate::error::ThemesError;

#[rstest]
#[case(ThemesError::ThemeNotFound, "Theme not found")]
#[case(ThemesError::ParseThemeFailed, "Failed to parse theme")]
#[case(ThemesError::Zip("bad zip".to_string()), "Zip error: bad zip")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_themes_error_display(#[case] error: ThemesError, #[case] expected: &str) {
    let display = format!("{}", error);

    assert_eq!(display, expected);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_themes_error_from_json() {
    let json_err = serde_json::from_str::<bool>("invalid").unwrap_err();

    let theme_err: ThemesError = json_err.into();

    assert_matches!(theme_err, ThemesError::Json(_));
}
