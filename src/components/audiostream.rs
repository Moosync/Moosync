use std::{
    rc::Rc,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    vec,
};
use tokio::sync::oneshot;
use types::{
    errors::errors::{MoosyncError, Result},
    songs::Song,
    ui::player_details::{PlayerEvents, PlayerState},
};

use leptos::{
    component, create_effect, create_node_ref, create_read_slice, create_slice, create_write_slice,
    html::Div, spawn_local, use_context, view, IntoView, NodeRef, RwSignal, SignalGet,
    SignalGetUntracked,
};

use crate::{
    console_log,
    players::{
        generic::GenericPlayer, librespot::LibrespotPlayer, local::LocalPlayer,
        youtube::YoutubePlayer,
    },
    store::{player_store::PlayerStore, provider_store::ProviderStore},
};

pub struct PlayerHolder {
    providers: Rc<ProviderStore>,
    players: Arc<Mutex<Vec<Box<dyn GenericPlayer>>>>,
    active_player: Arc<AtomicUsize>,
    state_setter: Rc<Box<dyn Fn(PlayerEvents)>>,
    player_container: NodeRef<Div>,
}

impl PlayerHolder {
    pub fn new(player_container: NodeRef<Div>, providers: Rc<ProviderStore>) -> PlayerHolder {
        let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
        let state_setter = Rc::new(Self::register_internal_state_listeners(player_store));
        let holder = PlayerHolder {
            players: Arc::new(Mutex::new(vec![])),
            state_setter,
            active_player: Arc::new(AtomicUsize::new(0)),
            providers,
            player_container,
        };
        holder.register_external_state_listeners(player_store);

        let mut players = holder.players.lock().unwrap();

        let local_player = LocalPlayer::new();
        players.push(Box::new(local_player));

        let youtube_player = YoutubePlayer::new();
        players.push(Box::new(youtube_player));

        let librespot_player = LibrespotPlayer::new();
        players.push(Box::new(librespot_player));

        drop(players);

        holder
    }

    pub fn initialize_players(&self) {
        for player in self.players.lock().unwrap().iter_mut() {
            player.initialize(self.player_container);
        }
        console_log!("Initialized players")
    }

    pub fn stop_playback(&self) -> Result<()> {
        let active_player = self.active_player.load(Ordering::Relaxed);
        let mut players = self.players.lock().unwrap();
        if let Some(player) = players.get_mut(active_player) {
            player.stop()?;
        }

        Ok(())
    }

    pub async fn get_player(&self, song: &Song) -> Result<(usize, Option<Song>)> {
        let players = self.players.lock().unwrap();
        let player = players
            .iter()
            .position(|p| p.provides().contains(&song.song.type_) && p.can_play(song));

        if let Some(player) = player {
            return Ok((player, None));
        }

        if player.is_none() {
            console_log!("Found no players, trying to refetch playback url");
            let mut song_tmp = song.clone();
            for (i, player) in players.iter().enumerate() {
                let playback_url = self.get_playback_url(song, player.key()).await?;
                console_log!("Got new playback url {}", playback_url);
                song_tmp.song.playback_url = Some(playback_url);
                if player.can_play(&song_tmp) {
                    return Ok((i, Some(song_tmp)));
                }
            }
        }

        Err(MoosyncError::String("Player not found".into()))
    }

    pub fn set_volume(&self, volume: f64) -> Result<()> {
        let active_player_pos = self.active_player.load(Ordering::Relaxed);
        let players = self.players.lock().unwrap();
        let active = players.get(active_player_pos);
        if active.is_none() {
            return Ok(());
        }

        active.unwrap().set_volume(volume)?;
        Ok(())
    }

    pub async fn get_playback_url(&self, song: &Song, player: String) -> Result<String> {
        let id = song.song._id.clone().unwrap();
        let provider = self.providers.get_provider_key_by_id(id.clone()).await?;
        self.providers
            .fetch_playback_url(provider, song.clone(), player)
            .await
    }

    pub async fn load_audio(&mut self, song: &Song, current_volume: f64) -> Result<Option<Song>> {
        let (pos, new_song) = self.get_player(song).await?;

        // Stop current player only if we need to switch;
        let old_pos = self.active_player.load(Ordering::Relaxed);
        if old_pos != pos {
            self.stop_playback()?;
        }

        let ret = new_song.clone();
        let src = if let Some(new_song) = new_song {
            new_song
                .song
                .playback_url
                .clone()
                .or(new_song.song.path.clone())
        } else {
            song.song.playback_url.clone().or(song.song.path.clone())
        };
        self.active_player.store(pos, Ordering::Relaxed);

        let mut players = self.players.lock().unwrap();
        let player = players.get_mut(pos).unwrap();
        player.add_listeners(self.state_setter.clone());

        let (resolver_tx, resolver_rx) = oneshot::channel();
        player.load(src.unwrap(), resolver_tx);

        resolver_rx.await.expect("Load failed to resolve");
        player.set_volume(current_volume).unwrap();
        player.play()?;

        Ok(ret)
    }

    fn listen_player_state(&self, player_store: RwSignal<PlayerStore>) {
        let player_state_getter = create_read_slice(player_store, move |p| p.player_details.state);
        let players = self.players.clone();
        let active_player = self.active_player.clone();
        create_effect(move |_| {
            let player_state = player_state_getter.get();
            let players = players.lock().unwrap();

            let active_player_pos = active_player.load(Ordering::Relaxed);

            let active = players.get(active_player_pos);
            if active.is_none() {
                return;
            }
            let active = active.unwrap();

            match player_state {
                PlayerState::Playing => {
                    active.play().unwrap();
                }
                PlayerState::Paused => {
                    active.pause().unwrap();
                }
                PlayerState::Stopped => {
                    active.pause().unwrap();
                }
                PlayerState::Loading => {}
            }
        });
    }

    fn listen_force_seek(&self, player_store: RwSignal<PlayerStore>) {
        let (force_seek, reset_force_seek) = create_slice(
            player_store,
            |p| p.player_details.force_seek,
            |p, _| p.force_seek_percent(-1f64),
        );
        let players = self.players.clone();
        let active_player = self.active_player.clone();
        create_effect(move |_| {
            let force_seek = force_seek.get();
            if force_seek < 0f64 {
                return;
            }

            let players = players.lock().unwrap();

            let active_player_pos = active_player.load(Ordering::Relaxed);

            let active = players.get(active_player_pos);
            if active.is_none() {
                return;
            }
            let active = active.unwrap();

            active.seek(force_seek).unwrap();

            reset_force_seek.set(-1f64);
        });
    }

    fn register_external_state_listeners(&self, player_store: RwSignal<PlayerStore>) {
        self.listen_player_state(player_store);
        self.listen_force_seek(player_store);
    }

    fn register_internal_state_listeners(
        player_store: RwSignal<PlayerStore>,
    ) -> Box<dyn Fn(PlayerEvents)> {
        let player_state_setter = create_write_slice(player_store, move |store, state| {
            store.set_state(state);
        });

        let player_time_setter = create_write_slice(player_store, move |store, time| {
            store.update_time(time);
        });

        let setter = move |ev: PlayerEvents| match ev {
            PlayerEvents::Play => player_state_setter.set(PlayerState::Playing),
            PlayerEvents::Pause => player_state_setter.set(PlayerState::Paused),
            PlayerEvents::Loading => player_state_setter.set(PlayerState::Loading),
            PlayerEvents::Ended => {
                console_log!("ended")
            }
            PlayerEvents::TimeUpdate(t) => player_time_setter.set(t),
        };

        Box::new(setter)
    }
}

#[component()]
pub fn AudioStream() -> impl IntoView {
    let provider_store = use_context::<Rc<ProviderStore>>().unwrap();
    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();

    let player_container_ref = create_node_ref::<Div>();

    let players = PlayerHolder::new(player_container_ref, provider_store);
    players.initialize_players();

    let current_song_sig = create_read_slice(player_store, |player_store| {
        player_store.current_song.clone()
    });
    let current_volume = create_read_slice(player_store, |player_store| player_store.get_volume());

    let players = Rc::new(Mutex::new(players));
    let players_copy = players.clone();
    create_effect(move |_| {
        let current_volume = current_volume.get();
        let player = players.lock().unwrap();
        player
            .set_volume(current_volume)
            .expect("Failed to set volume");
    });

    create_effect(move |_| {
        let current_song = current_song_sig.get();
        console_log!("Loading song {:?}", current_song);
        if let Some(current_song) = current_song {
            let players = players_copy.clone();
            spawn_local(async move {
                let mut players = players.lock().unwrap();
                let updated_song = players
                    .load_audio(&current_song, current_volume.get_untracked())
                    .await
                    .unwrap();

                if updated_song.is_some() {
                    // TODO: Update song in DB
                }
            });
        }
    });

    view! { <div id="player_container" class="player-container" _ref=player_container_ref></div> }
}
