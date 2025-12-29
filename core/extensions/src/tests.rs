use super::models::{MainCommand, RunnerCommand};
use serde_json::Value;
use types::songs::{InnerSong, Song};
use types::ui::extensions::PackageNameArgs;

#[test]
fn test_runner_command_try_from() {
    // Test FindNewExtensions
    let cmd = RunnerCommand::try_from(("findNewExtensions", &Value::Null));
    assert!(matches!(cmd.unwrap(), RunnerCommand::FindNewExtensions));

    // Test GetInstalledExtensions
    let cmd = RunnerCommand::try_from(("getInstalledExtensions", &Value::Null));
    assert!(matches!(
        cmd.unwrap(),
        RunnerCommand::GetInstalledExtensions
    ));

    // Test GetExtensionIcon
    let args = serde_json::to_value(PackageNameArgs {
        package_name: "test.pkg".to_string(),
    })
    .unwrap();
    let cmd = RunnerCommand::try_from(("getExtensionIcon", &args));
    if let RunnerCommand::GetExtensionIcon(pkg_args) = cmd.unwrap() {
        assert_eq!(pkg_args.package_name, "test.pkg");
    } else {
        panic!("Wrong command type");
    }

    // Invalid command
    let cmd = RunnerCommand::try_from(("invalidCommand", &Value::Null));
    assert!(cmd.is_err());
}

#[test]
fn test_main_command_sanitize() {
    let song = Song {
        song: InnerSong {
            _id: Some("123".to_string()),
            path: Some("/path".to_string()),
            ..Default::default()
        },
        ..Default::default()
    };

    let mut cmd = MainCommand::AddSongs(vec![song.clone()]);

    // Use to_request to trigger sanitization
    let _ = cmd.to_request("chan".to_string(), "test.pkg".to_string());

    if let MainCommand::AddSongs(songs) = cmd {
        // Sanitization logic in extensions uses "test.pkg:" prefix
        // Let's verify what sanitize_song does. It usually prefixes the ID if present.
        assert_eq!(songs[0].song._id.as_ref().unwrap(), "test.pkg:123");
    } else {
        panic!("Wrong command type");
    }
}
