use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    lock::Mutex,
    SinkExt, StreamExt,
};
use std::{
    rc::Rc,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    vec,
};
use tokio::sync::oneshot;
use types::{
    errors::{MoosyncError, Result},
    songs::Song,
    ui::player_details::{PlayerEvents, PlayerState},
};

use leptos::{
    component, create_effect, create_node_ref, create_read_slice, create_slice, create_write_slice,
    html::Div, spawn_local, use_context, view, IntoView, NodeRef, RwSignal, SignalGet,
    SignalGetUntracked, SignalUpdate,
};

use crate::{
    players::{
        generic::GenericPlayer, librespot::LibrespotPlayer, local::LocalPlayer, rodio::RodioPlayer,
        youtube::YoutubePlayer,
    },
    store::{player_store::PlayerStore, provider_store::ProviderStore},
};

pub struct PlayerHolder {
    providers: Rc<ProviderStore>,
    players: Rc<Mutex<Vec<Box<dyn GenericPlayer>>>>,
    active_player: Arc<AtomicUsize>,
    state_setter: Rc<Box<dyn Fn(PlayerEvents)>>,
    player_container: NodeRef<Div>,
    player_blacklist_receiver: Rc<Mutex<UnboundedReceiver<()>>>,
    listeners_active: Arc<AtomicBool>,
}

impl PlayerHolder {
    #[tracing::instrument(level = "trace", skip(player_container, providers))]
    pub fn new(player_container: NodeRef<Div>, providers: Rc<ProviderStore>) -> PlayerHolder {
        let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();

        let (player_blacklist_sender, player_blacklist_receiver) = unbounded();
        let state_setter = Rc::new(Self::register_internal_state_listeners(
            player_store,
            player_blacklist_sender,
        ));

        let mut players: Vec<Box<dyn GenericPlayer>> = vec![];

        let mut rodio_player = RodioPlayer::new();
        // Initialize listeners on first player
        rodio_player.add_listeners(state_setter.clone());

        players.push(Box::new(rodio_player));

        let local_player = LocalPlayer::new();
        players.push(Box::new(local_player));

        // let youtube_player = YoutubePlayer::new();
        // players.push(Box::new(youtube_player));

        let librespot_player = LibrespotPlayer::new();
        players.push(Box::new(librespot_player));

        let holder = PlayerHolder {
            players: Rc::new(Mutex::new(players)),
            state_setter,
            active_player: Arc::new(AtomicUsize::new(0)),
            providers,
            player_container,
            player_blacklist_receiver: Rc::new(Mutex::new(player_blacklist_receiver)),
            listeners_active: Arc::new(AtomicBool::new(true)),
        };
        holder.register_external_state_listeners(player_store);
        holder.listen_player_blacklist(player_store);

        holder
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn initialize_players(&self) {
        let players = self.players.lock().await;
        for player in players.iter() {
            player.initialize(self.player_container);
        }
        tracing::debug!("Initialized players")
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn stop_playback(&self) -> Result<()> {
        let active_player = self.active_player.load(Ordering::Relaxed);
        let mut players = self.players.lock().await;
        self.listeners_active.store(false, Ordering::Relaxed);
        if let Some(player) = players.get_mut(active_player) {
            player.stop()?;
        }

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, player_store, song))]
    pub async fn get_player(
        &self,
        player_store: RwSignal<PlayerStore>,
        song: &Song,
    ) -> Result<(usize, Option<Song>)> {
        tracing::debug!("Getting players for song {:?}", song);
        let players = self.players.lock().await;
        let player_blacklist = create_read_slice(player_store, |player_store| {
            player_store.get_player_blacklist()
        });
        let player = players.iter().position(|p| {
            tracing::debug!("Checking player capabilities {}", p.key());
            let res = !player_blacklist.get_untracked().contains(&p.key())
                && p.provides().contains(&song.song.type_)
                && p.can_play(song);

            tracing::debug!("Checked player capabilities {}", p.key());
            res
        });

        if let Some(player) = player {
            tracing::info!("Found player {}", player);
            return Ok((player, None));
        }

        if player.is_none() {
            tracing::warn!("Found no players, trying to refetch playback url");
            let mut song_tmp = song.clone();
            for (i, player) in players.iter().enumerate() {
                if player_blacklist.get_untracked().contains(&player.key()) {
                    continue;
                }
                tracing::debug!("Trying player {}", player.key());
                let playback_url = self.get_playback_url(song, player.key()).await;
                if let Ok(playback_url) = playback_url {
                    tracing::info!("Got new playback url {}", playback_url);
                    song_tmp.song.playback_url = Some(playback_url);
                    if player.can_play(&song_tmp) {
                        tracing::info!("Using player {}", player.key());
                        return Ok((i, Some(song_tmp)));
                    }
                } else {
                    tracing::error!(
                        "Failed to get playback url for player {}: {:?}",
                        player.key(),
                        playback_url
                    );
                }
            }
        }

        Err(MoosyncError::String("Player not found".into()))
    }

    #[tracing::instrument(level = "trace", skip(self, volume))]
    pub async fn set_volume(&self, volume: f64) -> Result<()> {
        let players = self.players.lock().await;
        let active_player_pos = self.active_player.load(Ordering::Relaxed);
        let active = players.get(active_player_pos);
        if active.is_none() {
            return Ok(());
        }

        tracing::info!("Active player {}", active.unwrap().key());
        active.unwrap().set_volume(volume)?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, song, player))]
    pub async fn get_playback_url(&self, song: &Song, player: String) -> Result<String> {
        let id = song.song._id.clone().unwrap();
        let provider = self.providers.get_provider_key_by_id(id.clone()).await?;
        self.providers
            .fetch_playback_url(provider, song.clone(), player)
            .await
    }

    fn set_player_listeners(&self, player: &mut Box<dyn GenericPlayer>) {
        if !self.listeners_active.load(Ordering::Relaxed) {
            player.add_listeners(self.state_setter.clone());
            self.listeners_active.store(true, Ordering::Relaxed);
        }
    }

    #[tracing::instrument(level = "trace", skip(self, song, current_volume, player_store))]
    pub async fn load_audio(
        &mut self,
        song: &Song,
        current_volume: f64,
        player_store: RwSignal<PlayerStore>,
    ) -> Result<Option<Song>> {
        let autoplay = create_read_slice(player_store, |p| {
            p.get_player_state() == PlayerState::Playing
        })
        .get();

        let (pos, new_song) = self.get_player(player_store, song).await?;

        // Stop current player only if we need to switch;
        let old_pos = self.active_player.load(Ordering::Relaxed);
        if old_pos != pos {
            self.stop_playback().await?;
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

        let mut players = self.players.lock().await;
        let player = players.get_mut(pos).unwrap();

        self.set_player_listeners(player);
        self.active_player.store(pos, Ordering::Relaxed);
        tracing::info!("Active player: {}", player.key());

        let (resolver_tx, resolver_rx) = oneshot::channel();
        player.load(src.unwrap(), resolver_tx);

        resolver_rx.await.expect("Load failed to resolve");
        player.set_volume(current_volume).unwrap();

        if autoplay {
            tracing::info!("Autoplaying");
            player.play()?;
        } else {
            player.pause()?;
        }

        Ok(ret)
    }

    #[tracing::instrument(level = "trace", skip(self, player_store))]
    fn listen_player_state(&self, player_store: RwSignal<PlayerStore>) {
        let player_state_getter = create_read_slice(player_store, move |p| p.get_player_state());
        let players = self.players.clone();
        let active_player = self.active_player.clone();
        create_effect(move |_| {
            let player_state = player_state_getter.get();

            let active_player_pos = active_player.load(Ordering::Relaxed);

            let players = players.clone();
            spawn_local(async move {
                let players = players.lock().await;
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
        });
    }

    #[tracing::instrument(level = "trace", skip(self, player_store))]
    fn listen_force_seek(&self, player_store: RwSignal<PlayerStore>) {
        let (force_seek, reset_force_seek) = create_slice(
            player_store,
            |p| p.get_force_seek(),
            |p, _| p.force_seek_percent(-1f64),
        );
        let players = self.players.clone();
        let active_player = self.active_player.clone();
        create_effect(move |_| {
            let force_seek = force_seek.get();
            tracing::debug!("Got force seek {}", force_seek);
            if force_seek < 0f64 {
                return;
            }

            let active_player_pos = active_player.load(Ordering::Relaxed);

            let players = players.clone();
            spawn_local(async move {
                let players = players.lock().await;
                let active = players.get(active_player_pos);
                if active.is_none() {
                    return;
                }
                let active = active.unwrap();

                tracing::debug!("Seeking player {}", active.key());
                active.seek(force_seek).unwrap();

                reset_force_seek.set(-1f64);
            });
        });
    }

    #[tracing::instrument(level = "trace", skip(self, player_store))]
    fn register_external_state_listeners(&self, player_store: RwSignal<PlayerStore>) {
        self.listen_player_state(player_store);
        self.listen_force_seek(player_store);
    }

    #[tracing::instrument(level = "trace", skip(player_store, player_blacklist_sender))]
    fn register_internal_state_listeners(
        player_store: RwSignal<PlayerStore>,
        player_blacklist_sender: UnboundedSender<()>,
    ) -> Box<dyn Fn(PlayerEvents)> {
        let player_state_setter = create_write_slice(player_store, move |store, state| {
            store.set_state(state);
        });

        let next_song_setter =
            create_write_slice(player_store, move |store, _| match store.get_repeat() {
                types::ui::player_details::RepeatModes::None => store.next_song(),
                types::ui::player_details::RepeatModes::Once => {
                    if !store.get_has_repeated() {
                        tracing::info!("repeating now");
                        store.force_seek_percent(0f64);
                        store.set_state(PlayerState::Playing);
                        store.set_has_repeated(true);
                    } else {
                        store.set_has_repeated(false);
                        store.next_song();
                        store.set_state(PlayerState::Playing);
                    }
                }
                types::ui::player_details::RepeatModes::Loop => {
                    tracing::info!("repeating now");
                    store.force_seek_percent(0f64);
                    store.set_state(PlayerState::Playing);
                }
            });

        let player_time_setter = create_write_slice(player_store, move |store, time| {
            store.update_time(time);
        });

        let setter = move |ev: PlayerEvents| match ev {
            PlayerEvents::Play => player_state_setter.set(PlayerState::Playing),
            PlayerEvents::Pause => player_state_setter.set(PlayerState::Paused),
            PlayerEvents::Loading => player_state_setter.set(PlayerState::Loading),
            PlayerEvents::Ended => {
                tracing::debug!("Got ended");
                next_song_setter.set(());
            }
            PlayerEvents::TimeUpdate(t) => player_time_setter.set(t),
            PlayerEvents::Error(err) => {
                tracing::error!("Error playing song: {:?}", err);
                let mut player_blacklist_sender = player_blacklist_sender.clone();
                spawn_local(async move {
                    player_blacklist_sender.send(()).await.unwrap();
                });
                // player_state_setter.set(PlayerState::Stopped);
            }
        };

        Box::new(setter)
    }

    #[tracing::instrument(level = "trace", skip(self, player_store))]
    fn listen_player_blacklist(&self, player_store: RwSignal<PlayerStore>) {
        let player_blacklist_receiver = self.player_blacklist_receiver.clone();
        let active_player = self.active_player.clone();
        let players = self.players.clone();
        spawn_local(async move {
            let mut player_blacklist_receiver = player_blacklist_receiver.lock().await;
            loop {
                player_blacklist_receiver.next().await;
                let active_player_pos = active_player.load(Ordering::Relaxed);
                let players = players.lock().await;
                let active_player = players.get(active_player_pos);
                if let Some(active_player) = active_player {
                    let player_key = active_player.key();
                    tracing::warn!("blacklisting player {}", player_key);
                    player_store.update(|p| p.blacklist_player(player_key))
                }
            }
        });
    }
}

#[tracing::instrument(level = "trace", skip())]
#[component()]
pub fn AudioStream() -> impl IntoView {
    let provider_store = use_context::<Rc<ProviderStore>>().unwrap();
    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();

    let player_container_ref = create_node_ref::<Div>();

    let players = PlayerHolder::new(player_container_ref, provider_store);
    let players = Rc::new(Mutex::new(players));
    let players_clone = players.clone();
    spawn_local(async move {
        let players = players_clone.lock().await;
        players.initialize_players().await;
    });

    let current_song_sig =
        create_read_slice(player_store, |player_store| player_store.get_current_song());

    let force_load_sig =
        create_read_slice(player_store, |player_store| player_store.get_force_load());
    let current_volume = create_read_slice(player_store, |player_store| player_store.get_volume());

    let players_clone = players.clone();
    create_effect(move |_| {
        let current_volume = current_volume.get();
        let players = players.clone();
        spawn_local(async move {
            let players = players.lock().await;
            players
                .set_volume(current_volume)
                .await
                .expect("Failed to set volume");
        });
    });

    create_effect(move |_| {
        let current_song = current_song_sig.get();
        let _ = force_load_sig.get();
        if let Some(current_song) = current_song {
            tracing::info!("Loading song {:?}", current_song.song.title);
            let players = players_clone.clone();
            spawn_local(async move {
                let mut players = players.lock().await;
                let updated_song = players
                    .load_audio(&current_song, current_volume.get_untracked(), player_store)
                    .await;

                if let Ok(updated_song) = updated_song {
                    if updated_song.is_some() {
                        // TODO: Update song in DB
                    }
                } else {
                    tracing::info!("Failed to load Song {:?}", updated_song);
                }
            });
        } else {
            let players = players_clone.clone();
            spawn_local(async move {
                tracing::info!("Unloading audio");
                let players = players.lock().await;
                let _ = players.stop_playback().await;
            });
        }
    });

    view! { <div id="player_container" class="player-container" _ref=player_container_ref></div> }
}
