// Moosync
// Copyright (C) 2026  Moosync <support@moosync.app>
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

use crate::utils::validation::validate_input;

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_validate_empty_rule() {
    let value = "any value";
    let rule = "";

    let result = validate_input(value, rule);

    assert!(result);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_validate_regex_matching() {
    let rule = "^https?://.+";
    let valid_url = "https://example.com";
    let valid_http = "http://localhost:8080/manifest.json";
    let invalid_url = "ftp://example.com";
    let invalid_plain = "not a url";

    let res_https = validate_input(valid_url, rule);
    let res_http = validate_input(valid_http, rule);
    let res_ftp = validate_input(invalid_url, rule);
    let res_plain = validate_input(invalid_plain, rule);

    assert!(res_https);
    assert!(res_http);
    assert!(!res_ftp);
    assert!(!res_plain);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_validate_invalid_regex() {
    let value = "hello";
    let invalid_rule = "[a-z";

    let result = validate_input(value, invalid_rule);

    assert!(!result);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_validate_arithmetic_range_and_comparisons() {
    let rule = ">= 0 && <= 128";

    let valid_zero = validate_input("0", rule);
    let valid_mid = validate_input("64", rule);
    let valid_max = validate_input("128", rule);
    let invalid_neg = validate_input("-1", rule);
    let invalid_high = validate_input("129", rule);
    let invalid_str = validate_input("abc", rule);

    assert!(valid_zero);
    assert!(valid_mid);
    assert!(valid_max);
    assert!(!invalid_neg);
    assert!(!invalid_high);
    assert!(!invalid_str);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_validate_arithmetic_or_conditions() {
    let rule = "== -1 || >= 1";

    let valid_neg1 = validate_input("-1", rule);
    let valid_1 = validate_input("1", rule);
    let valid_10 = validate_input("10", rule);
    let invalid_0 = validate_input("0", rule);
    let invalid_neg2 = validate_input("-2", rule);

    assert!(valid_neg1);
    assert!(valid_1);
    assert!(valid_10);
    assert!(!invalid_0);
    assert!(!invalid_neg2);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_validate_arithmetic_range_syntax() {
    let inclusive_rule = "1..=10";
    let exclusive_rule = "1..10";

    let res_inc_1 = validate_input("1", inclusive_rule);
    let res_inc_10 = validate_input("10", inclusive_rule);
    let res_inc_11 = validate_input("11", inclusive_rule);
    let res_exc_10 = validate_input("10", exclusive_rule);

    assert!(res_inc_1);
    assert!(res_inc_10);
    assert!(!res_inc_11);
    assert!(!res_exc_10);
}
