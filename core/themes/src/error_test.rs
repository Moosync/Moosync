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
fn test_themes_error_from_decode() {
    let decode_err =
        <themes_proto::moosync::types::ThemeDetails as prost::Message>::decode(&b"\xff\xff"[..])
            .unwrap_err();

    let theme_err: ThemesError = decode_err.into();

    assert_matches!(theme_err, ThemesError::Decode(_));
}
