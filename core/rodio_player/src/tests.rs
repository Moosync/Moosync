use super::RodioPlayer;

#[test]
fn test_rodio_player_new() {
    let player = RodioPlayer::new();
    // We can't assert much about internal state without more access, but we can verify it constructed
    let rx = player.get_events_rx();
    let _locked_rx = rx.lock().unwrap();
}

#[tokio::test]
async fn test_rodio_player_commands() {
    let player = RodioPlayer::new();

    // Test that commands can be sent without error
    // Note: rodio::OutputStreamBuilder::open_default_stream() might fail if no audio device.
    // In that case, the thread inside new() might panic or just fail to initialize sink.
    // If it panics, the tests might fail.

    // We might need to ensure we don't panic on missing audio device in the implementation,
    // or we accept that tests fail in this env if no audio device.
    // But user said "I just want these dependencies to be build. In tests they have to be mocked anyways".
    // I can't mock rodio crate easily without changing BUILD.

    // Let's see if it crashes.

    assert!(player.rodio_play().await.is_ok());
    assert!(player.rodio_pause().await.is_ok());
    assert!(player.rodio_stop().await.is_ok());
    assert!(player.rodio_seek(10.0).await.is_ok());
    assert!(player.rodio_set_volume(0.5).await.is_ok());
}
