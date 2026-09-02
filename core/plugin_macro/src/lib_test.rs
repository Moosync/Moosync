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

use rstest::rstest;
use tracing_test::traced_test;

use crate::to_snake_case;

#[rstest]
#[case("Database", "database")]
#[case("PreferenceConfig", "preference_config")]
#[case("ScannerHolder", "scanner_holder")]
#[case("MprisHolder", "mpris_holder")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_snake_case(#[case] input: &str, #[case] expected: &str) {
    assert_eq!(to_snake_case(input), expected);
}
