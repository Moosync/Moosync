use assertables::{assert_none, assert_some, assert_some_eq_x};
use rstest::rstest;
use tracing_test::traced_test;

use super::{parse_color, parse_length};

#[rstest]
#[case("#ff5733", true)]
#[case("not-a-color", false)]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_color(#[case] input: &str, #[case] is_valid: bool) {
    let col = parse_color(input);

    if is_valid {
        assert_some!(col);
        return;
    }

    assert_none!(col);
}

#[rstest]
#[case("16px", Some(16.0))]
#[case("32", Some(32.0))]
#[case("invalid", None)]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_length(#[case] input: &str, #[case] expected: Option<f32>) {
    let len = parse_length(input);

    if let Some(exp) = expected {
        assert_some_eq_x!(len, exp);
        return;
    }

    assert_none!(len);
}
