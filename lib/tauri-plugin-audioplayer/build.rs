const COMMANDS: &[&str] = &["registerListener", "load", "play", "pause", "stop", "seek"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .ios_path("ios")
        .build();
}
