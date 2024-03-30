use std::{
    borrow::BorrowMut,
    cell::RefCell,
    fmt::Debug,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    vec,
};
use tokio::sync::mpsc::{Receiver, Sender};
use types::{
    errors::errors::{MoosyncError, Result},
    songs::{GetSongOptions, Song},
    ui::player_details::{PlayerEvents, PlayerState},
};

use leptos::{
    component, create_action, create_effect, create_node_ref, create_read_slice, create_rw_signal,
    create_write_slice, html::Div, spawn_local, use_context, view, IntoView, RwSignal, SignalGet,
    SignalUpdate,
};

use crate::{
    console_log,
    players::{generic::GenericPlayer, local::LocalPlayer},
    store::player_store::PlayerStore,
    utils::{common::convert_file_src, db_utils::get_songs_by_option},
};

#[derive(Debug)]
pub struct PlayerHolder {
    players: Arc<Mutex<Vec<Box<dyn GenericPlayer>>>>,
    active_player: Arc<AtomicUsize>,
    listener_tx: Sender<PlayerEvents>,
}

impl PlayerHolder {
    pub fn new() -> PlayerHolder {
        let (tx, rx) = tokio::sync::mpsc::channel::<PlayerEvents>(1);

        let holder = PlayerHolder {
            players: Arc::new(Mutex::new(vec![])),
            listener_tx: tx,
            active_player: Arc::new(AtomicUsize::new(0)),
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

    pub fn get_player(&self, song: &Song) -> Result<usize> {
        let players = self.players.lock().unwrap();
        let player = players
            .iter()
            .position(|p| p.provides().contains(&song.song.type_));

        player
            .map(Ok)
            .unwrap_or(Err(MoosyncError::String("Player not found".into())))
    }

    pub fn load_audio(&mut self, song: Song) -> Result<()> {
        let src = song.song.playback_url.clone().or(song.song.path.clone());
        if src.is_none() {
            return Ok(());
        }

        let pos = self.get_player(&song)?;
        self.active_player.store(pos, Ordering::Relaxed);

        console_log!("getting src");
        let src = convert_file_src(src.clone().unwrap(), "asset");

        let mut players = self.players.lock().unwrap();
        let player = players.get_mut(pos).unwrap();
        player.add_listeners(self.listener_tx.clone());
        player.load(src);
        player.play()?;

        Ok(())
    }

    fn register_external_state_listeners(&self, player_store: RwSignal<PlayerStore>) {
        let player_state_getter = create_read_slice(player_store, move |p| p.player_details.state);

        let players = self.players.clone();
        let active_player = self.active_player.clone();
        create_effect(move |_| {
            let player_state = player_state_getter.get();
            console_log!("{:?}", player_state);

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
    let players = RefCell::new(PlayerHolder::new());
    players.borrow().initialize_players();

    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
    let current_song_sig = create_read_slice(player_store, |player_store| {
        player_store.current_song.clone()
    });

    create_effect(move |_| {
        let current_song = current_song_sig.get();
        if let Some(current_song) = current_song {
            let mut players = players.borrow_mut();
            players.load_audio(current_song).unwrap();
        }
    });

    let songs = create_rw_signal(vec![]);
    get_songs_by_option(GetSongOptions::default(), songs);

    create_effect(move |_| {
        let songs_list = songs.get();
        player_store.write_only().update(|p| {
            let first_song = songs_list.first();
            if let Some(first_song) = first_song {
                p.add_to_queue(first_song.clone());
            }
        });
    });

    let player_container_ref = create_node_ref::<Div>();

    view! { <div id="player_container" _ref=player_container_ref></div> }
}
