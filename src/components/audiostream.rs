use std::{
    fmt::Debug,
    rc::Rc,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    vec,
};
use tokio::sync::{
    mpsc::{Receiver, Sender},
    oneshot,
};
use types::{
    errors::errors::{MoosyncError, Result},
    songs::Song,
    ui::player_details::{PlayerEvents, PlayerState},
};

use leptos::{
    component, create_effect, create_node_ref, create_read_slice, create_slice, create_write_slice,
    html::Div, spawn_local, use_context, view, IntoView, RwSignal, SignalGet, SignalGetUntracked,
};

use crate::{
    console_log,
    players::{generic::GenericPlayer, local::LocalPlayer},
    store::{player_store::PlayerStore, provider_store::ProviderStore},
    utils::common::convert_file_src,
};

#[derive(Debug)]
pub struct PlayerHolder {
    providers: Rc<ProviderStore>,
    players: Arc<Mutex<Vec<Box<dyn GenericPlayer>>>>,
    active_player: Arc<AtomicUsize>,
    listener_tx: Sender<PlayerEvents>,
}

impl PlayerHolder {
    pub fn new(providers: Rc<ProviderStore>) -> PlayerHolder {
        let (tx, rx) = tokio::sync::mpsc::channel::<PlayerEvents>(1);

        let holder = PlayerHolder {
            players: Arc::new(Mutex::new(vec![])),
            listener_tx: tx,
            active_player: Arc::new(AtomicUsize::new(0)),
            providers,
        };

        let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
        holder.register_internal_state_listeners(rx, player_store);
        holder.register_external_state_listeners(player_store);

        let mut players = holder.players.lock().unwrap();

        let local_player = LocalPlayer::new();
        players.push(Box::new(local_player));

        drop(players);

        holder
    }

    pub fn initialize_players(&self) {
        for player in self.players.lock().unwrap().iter() {
            player.initialize();
        }
    }

    pub async fn get_player(&self, song: &Song) -> Result<(usize, Option<Song>)> {
        let players = self.players.lock().unwrap();
        let player = players
            .iter()
            .position(|p| p.provides().contains(&song.song.type_) && p.can_play(&song));

        if player.is_some() {
            return Ok((player.unwrap(), None));
        }

        if player.is_none() {
            let mut song_tmp = song.clone();
            for (i, player) in players.iter().enumerate() {
                let playback_url = self.get_playback_url(song, player.key()).await?;
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

        active.unwrap().set_volume(volume / 100f64)?;
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

        let src = convert_file_src(src.unwrap());
        console_log!("got src {}", src);

        let mut players = self.players.lock().unwrap();
        let player = players.get_mut(pos).unwrap();
        player.add_listeners(self.listener_tx.clone());

        let (resolver_tx, resolver_rx) = oneshot::channel();
        player.load(src, resolver_tx);

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
        &self,
        mut listeners_rx: Receiver<PlayerEvents>,
        player_store: RwSignal<PlayerStore>,
    ) {
        let player_state_setter = create_write_slice(player_store, move |store, state| {
            store.set_state(state);
        });

        let player_time_setter = create_write_slice(player_store, move |store, time| {
            store.update_time(time);
        });

        spawn_local(async move {
            loop {
                let event = listeners_rx.recv().await;
                if let Some(event) = event {
                    match event {
                        PlayerEvents::Play => player_state_setter.set(PlayerState::Playing),
                        PlayerEvents::Pause => player_state_setter.set(PlayerState::Paused),
                        PlayerEvents::Loading => player_state_setter.set(PlayerState::Loading),
                        PlayerEvents::Ended => {
                            console_log!("ended")
                        }
                        PlayerEvents::TimeUpdate(t) => player_time_setter.set(t),
                    }
                }
            }
        });
    }
}

#[component()]
pub fn AudioStream() -> impl IntoView {
    let provider_store = use_context::<Rc<ProviderStore>>().unwrap();
    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();

    let players = PlayerHolder::new(provider_store);
    players.initialize_players();

    let current_song_sig = create_read_slice(player_store, |player_store| {
        player_store.current_song.clone()
    });
    let current_volume = create_read_slice(player_store, |player_store| {
        player_store.player_details.volume
    });

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

    let player_container_ref = create_node_ref::<Div>();

    view! { <div id="player_container" _ref=player_container_ref></div> }
}
