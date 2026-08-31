use super::{parse_color, parse_length};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_color_valid() {
    let col = parse_color("#ff5733");

    assert!(col.is_some());
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_color_invalid() {
    let col = parse_color("not-a-color");

    assert!(col.is_none());
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_length_px() {
    let len = parse_length("16px");

    assert_eq!(len, Some(16.0));
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_length_raw_number() {
    let len = parse_length("32");

    assert_eq!(len, Some(32.0));
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_length_invalid() {
    let len = parse_length("invalid");

    assert!(len.is_none());
}
