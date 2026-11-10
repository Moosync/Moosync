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

use std::time::Duration;

use assertables::{assert_ok, assert_ok_eq_x};
use extensions_proto::moosync::types::PlayerState;
use rstest::{fixture, rstest};
use tokio::sync::mpsc::unbounded_channel;
use tracing_test::traced_test;

use super::{
    AudioPlayerContext, DummyAudioPlayerContext, PersistContext, RodioPlayerContext,
    file_persist::FilePersist,
};
use crate::source::ValidSrc;

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn dummy_player_context() -> DummyAudioPlayerContext { DummyAudioPlayerContext::new() }

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_dummy_audio_player_context_operations(dummy_player_context: DummyAudioPlayerContext) {
    let (events_tx, _events_rx) = unbounded_channel();

    assert_ok_eq_x!(
        dummy_player_context.get_player_state(),
        PlayerState::Stopped
    );
    assert_ok_eq_x!(dummy_player_context.get_volume(), 100);

    dummy_player_context.set_volume(80).await.unwrap();
    assert_ok_eq_x!(dummy_player_context.get_volume(), 80);

    let src = ValidSrc::new_url("https://example.com/test.mp3");
    dummy_player_context
        .set_src(src.into_owned(), events_tx)
        .await
        .unwrap();
    assert_ok_eq_x!(
        dummy_player_context.get_player_state(),
        PlayerState::Playing
    );

    dummy_player_context.pause().await.unwrap();
    assert_ok_eq_x!(dummy_player_context.get_player_state(), PlayerState::Paused);

    dummy_player_context.play().await.unwrap();
    assert_ok_eq_x!(
        dummy_player_context.get_player_state(),
        PlayerState::Playing
    );

    dummy_player_context
        .seek(Duration::from_secs(45))
        .await
        .unwrap();
    assert_ok_eq_x!(
        dummy_player_context.get_current_pos(),
        Duration::from_secs(45)
    );

    dummy_player_context.stop().await.unwrap();
    assert_ok_eq_x!(
        dummy_player_context.get_player_state(),
        PlayerState::Stopped
    );
    assert_ok_eq_x!(dummy_player_context.get_current_pos(), Duration::default());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_file_persist_lifecycle() {
    let tmp = tempdir::TempDir::new("test_file_persist").unwrap();
    let persist = FilePersist::new(tmp.path().join("nested"));

    let loaded_default = persist.load().unwrap();
    assert_eq!(
        loaded_default,
        player_proto::moosync::types::PlayerData::default()
    );

    let data = player_proto::moosync::types::PlayerData {
        current_idx: 3,
        ..Default::default()
    };
    let persist_res = persist.persist(&data);
    assert!(persist_res.is_ok());

    let loaded_saved = persist.load().unwrap();
    assert_eq!(loaded_saved.current_idx, 3);

    std::fs::write(
        tmp.path().join("nested").join("player_state.pb"),
        b"corrupted bytes",
    )
    .unwrap();
    let load_corrupt = persist.load();
    assert!(load_corrupt.is_err());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_rodio_player_context_operations() {
    let ctx = RodioPlayerContext::default();
    assert_ok!(ctx.play().await);
    assert_ok!(ctx.pause().await);
    assert_ok!(ctx.stop().await);
    assert_ok!(ctx.set_volume(80).await);
    assert_ok_eq_x!(ctx.get_volume(), 100);
    assert_ok!(ctx.seek(Duration::from_secs(5)).await);
    assert_ok_eq_x!(ctx.get_current_pos(), Duration::default());
    assert_ok_eq_x!(ctx.get_player_state(), PlayerState::Stopped);

    let tmp = tempdir::TempDir::new("test_rodio_can_play").unwrap();
    let file_path = tmp.path().join("song.mp3");
    std::fs::write(&file_path, b"dummy audio").unwrap();
    assert!(ctx.can_play(&ValidSrc::Path(file_path)).await);
    assert!(
        !ctx.can_play(&ValidSrc::Path(tmp.path().join("nonexistent.mp3")))
            .await
    );
    assert!(
        ctx.can_play(&ValidSrc::new_url("https://example.com/stream.mp3"))
            .await
    );
    assert!(
        ctx.can_play(&ValidSrc::new_url("http://example.com/stream.mp3"))
            .await
    );
    assert!(!ctx.can_play(&ValidSrc::new_url("file:///test")).await);
}
