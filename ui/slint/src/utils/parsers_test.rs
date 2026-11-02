use assertables::{assert_none, assert_some, assert_some_eq_x};
use rstest::rstest;
use tracing_test::traced_test;

use super::{default_rgba, parse_color, parse_length, parse_rgba_color};

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

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_rgba_color() {
    let input = "#ff0000";

    let color = parse_rgba_color(input);

    assert_eq!(color.r, 255.0);
    assert_eq!(color.g, 0.0);
    assert_eq!(color.b, 0.0);
    assert_eq!(color.a, 1.0);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_rgba_color_fallback_default() {
    let input = "not-a-color";

    let color = parse_rgba_color(input);
    let default = default_rgba();

    assert_eq!(color.r, default.r);
    assert_eq!(color.g, default.g);
    assert_eq!(color.b, default.b);
    assert_eq!(color.a, default.a);
}
