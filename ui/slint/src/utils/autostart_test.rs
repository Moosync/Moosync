use super::*;

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_auto_launch() {
    #[cfg(not(target_os = "android"))]
    {
        let auto = get_auto_launch();
        assert!(auto.is_some());
    }
}
