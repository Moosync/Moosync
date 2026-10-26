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

use std::{error::Error, sync::Arc};

use async_trait::async_trait;
use extensions_proto::moosync::types::{PlayerState, player_event::Event as PlayerEvent};
use mpris::{MediaControlEvent, MprisHolder};
use player::PlayerHandler;
use tracing::Instrument;

use super::Hook;
use crate::StateManager;

pub struct MprisHook;

impl Default for MprisHook {
    #[tracing::instrument(level = "debug", skip_all)]
    fn default() -> Self { Self::new() }
}

impl MprisHook {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self { Self }

    #[tracing::instrument(level = "debug", skip_all)]
    fn bind_mpris(player_handler: &Arc<PlayerHandler>, mpris_holder: &Arc<MprisHolder>) {
        Self::bind_mpris_song_change(player_handler, mpris_holder);
        Self::bind_mpris_player_event(player_handler, mpris_holder);
        Self::bind_mpris_controls(player_handler, mpris_holder);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn bind_mpris_song_change(
        player_handler: &Arc<PlayerHandler>,
        mpris_holder: &Arc<MprisHolder>,
    ) {
        let mpris = mpris_holder.clone();
        player_handler.on_song_changed.listen(move |song| {
            if let Some(song) = song.as_ref()
                && let Err(e) = mpris.set_metadata(song)
            {
                tracing::error!("Failed to update MPRIS metadata: {:?}", e);
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn bind_mpris_player_event(
        player_handler: &Arc<PlayerHandler>,
        mpris_holder: &Arc<MprisHolder>,
    ) {
        let mpris = mpris_holder.clone();
        player_handler.on_player_event.listen(move |event| {
            if let Some(ref ev) = event.event {
                let res = match ev {
                    PlayerEvent::Play(_) => mpris.set_playback_state(PlayerState::Playing),
                    PlayerEvent::Pause(_) => mpris.set_playback_state(PlayerState::Paused),
                    PlayerEvent::TimeUpdate(dur) => mpris
                        .set_position(dur.seconds as f64 + (dur.nanos as f64 / 1_000_000_000.0)),
                    _ => Ok(()),
                };
                if let Err(e) = res {
                    tracing::error!("Failed to update MPRIS playback state: {:?}", e);
                }
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn bind_mpris_controls(player_handler: &Arc<PlayerHandler>, mpris_holder: &Arc<MprisHolder>) {
        let player = player_handler.clone();
        let event_rx = mpris_holder.event_rx.clone();
        tokio::task::spawn_blocking(move || {
            while let Ok(event) = event_rx.lock().unwrap().recv() {
                Self::handle_media_control_event(&player, event);
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn handle_media_control_event(player: &Arc<PlayerHandler>, event: MediaControlEvent) {
        match event {
            MediaControlEvent::Play => {
                let player_clone = player.clone();
                tokio::spawn(
                    async move {
                        if let Err(e) = player_clone.play().await {
                            tracing::error!("Failed to handle MPRIS Play: {:?}", e);
                        }
                    }
                    .in_current_span(),
                );
            }
            MediaControlEvent::Pause => {
                if let Err(e) = player.pause() {
                    tracing::error!("Failed to handle MPRIS Pause: {:?}", e);
                }
            }
            MediaControlEvent::Toggle => {
                let player_clone = player.clone();
                tokio::spawn(
                    async move {
                        if player_clone.get_player_state() == PlayerState::Playing as i32 {
                            let _ = player_clone.pause();
                        } else if let Err(e) = player_clone.play().await {
                            tracing::error!("Failed to handle MPRIS Toggle: {:?}", e);
                        }
                    }
                    .in_current_span(),
                );
            }
            MediaControlEvent::Next => {
                let player_clone = player.clone();
                tokio::spawn(
                    async move {
                        player_clone.next().await;
                    }
                    .in_current_span(),
                );
            }
            MediaControlEvent::Previous => {
                let player_clone = player.clone();
                tokio::spawn(
                    async move {
                        player_clone.prev().await;
                    }
                    .in_current_span(),
                );
            }
            MediaControlEvent::Stop => {
                if let Err(e) = player.pause() {
                    tracing::error!("Failed to handle MPRIS Stop: {:?}", e);
                }
            }
            _ => {}
        }
    }
}

#[async_trait]
impl Hook for MprisHook {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_startup(
        &self,
        state_manager: &StateManager,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let player_handler = state_manager.plugins.get::<PlayerHandler>();
        let mpris_holder = state_manager.plugins.get::<MprisHolder>();
        Self::bind_mpris(&player_handler, &mpris_holder);
        Ok(())
    }
}
