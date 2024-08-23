use rand::seq::SliceRandom;
use serde::Serialize;
use std::{cmp::min, collections::HashMap};
use types::{
    extensions::ExtensionExtraEvent,
    preferences::CheckboxPreference,
    songs::Song,
    ui::player_details::{PlayerState, RepeatModes, VolumeMode},
};

use crate::{console_log, utils::extensions::send_extension_event};

#[derive(Debug, Default, PartialEq, Clone, Serialize)]
pub struct Queue {
    pub song_queue: Vec<String>,
    pub current_index: usize,
    pub data: HashMap<String, Song>,
}

#[derive(Debug, Default)]
pub struct PlayerDetails {
    pub current_time: f64,
    pub force_seek: f64,
    pub state: PlayerState,
    pub has_repeated: bool,
    pub repeat: RepeatModes,
    volume: f64,
    volume_mode: VolumeMode,
    volume_map: HashMap<String, f64>,
    clamp_map: HashMap<String, f64>,
}

#[derive(Debug, Default)]
pub struct PlayerStore {
    pub queue: Queue,
    pub current_song: Option<Song>,
    pub player_details: PlayerDetails,
    pub player_blacklist: Vec<String>,
    pub force_load_song: bool,
}

impl PlayerStore {
    pub fn new() -> PlayerStore {
        PlayerStore::default()
    }

    pub fn update_current_song(&mut self) {
        if self.queue.current_index >= self.queue.song_queue.len() {
            self.queue.current_index = 0;
        }
        let id = self.queue.song_queue[self.queue.current_index].clone();
        let song = self.queue.data.get(&id).cloned();

        if song == self.current_song && self.player_blacklist.is_empty() {
            return;
        }

        console_log!("Upading song in queue");
        self.current_song = song.clone();

        self.clear_blacklist();

        send_extension_event(ExtensionExtraEvent::SongChanged([song]))
    }

    pub fn add_to_queue(&mut self, songs: Vec<Song>) {
        self.add_to_queue_at_index(songs, self.queue.song_queue.len());
        self.update_current_song();
    }

    fn add_to_queue_at_index(&mut self, songs: Vec<Song>, index: usize) {
        let mut index = index;
        for song in songs {
            self.insert_song_at_index(song, index);
            index += 1;
        }
    }

    pub fn remove_from_queue(&mut self, index: usize) {
        self.queue.song_queue.remove(index);
    }

    fn insert_song_at_index(&mut self, song: Song, index: usize) {
        let song_id = song.song._id.clone().unwrap();
        self.queue.data.insert(song_id.clone(), song);
        let insertion_index = min(self.queue.song_queue.len(), index);
        self.queue.song_queue.insert(insertion_index, song_id);
    }

    pub fn play_now(&mut self, song: Song) {
        self.insert_song_at_index(song, self.queue.current_index + 1);
        self.queue.current_index += 1;
        self.update_current_song();
    }

    pub fn play_now_multiple(&mut self, songs: Vec<Song>) {
        if songs.is_empty() {
            return;
        }

        let first_song = songs.first();
        if let Some(first_song) = first_song {
            self.play_now(first_song.clone())
        }

        if songs.len() > 1 {
            self.add_to_queue_at_index(songs[1..].to_vec(), self.queue.current_index + 1);
        }
    }

    pub fn play_next(&mut self, song: Song) {
        self.insert_song_at_index(song, self.queue.current_index + 1);
    }

    pub fn play_next_multiple(&mut self, songs: Vec<Song>) {
        if songs.is_empty() {
            return;
        }

        let first_song = songs.first();
        if let Some(first_song) = first_song {
            self.play_next(first_song.clone())
        }

        if songs.len() > 1 {
            self.add_to_queue_at_index(songs[1..].to_vec(), self.queue.current_index + 1);
        }
    }

    pub fn change_index(&mut self, new_index: usize) {
        if new_index >= self.queue.song_queue.len() {
            return;
        }

        self.queue.current_index = new_index;
        self.update_current_song();
    }

    pub fn update_time(&mut self, new_time: f64) {
        self.player_details.current_time = new_time;
    }

    pub fn get_time(&self) -> f64 {
        self.player_details.current_time
    }

    pub fn force_seek_percent(&mut self, new_time: f64) {
        let new_time = if let Some(current_song) = &self.current_song {
            current_song.song.duration.unwrap_or_default() * new_time
        } else {
            0f64
        };

        self.player_details.force_seek = new_time;
        send_extension_event(ExtensionExtraEvent::Seeked([new_time]))
    }

    pub fn set_state(&mut self, state: PlayerState) {
        self.player_details.state = state;
        send_extension_event(ExtensionExtraEvent::PlayerStateChanged([state]))
    }

    fn get_song_key(&self) -> String {
        if let Some(current_song) = &self.current_song {
            return current_song
                .song
                .provider_extension
                .clone()
                .unwrap_or(current_song.song.type_.to_string());
        }
        "".to_string()
    }

    pub fn set_volume(&mut self, volume: f64) {
        if let VolumeMode::PersistSeparate = self.player_details.volume_mode {
            let song_key = self.get_song_key();
            if !song_key.is_empty() {
                console_log!("Setting volume for song: {}, {}", song_key, volume);
                self.player_details.volume_map.insert(song_key, volume);
            }
        }
        self.player_details.volume = volume;

        send_extension_event(ExtensionExtraEvent::VolumeChanged([volume]))
    }

    pub fn get_volume(&self) -> f64 {
        let mut clamp = 100f64;
        let mut volume = self.player_details.volume;
        let song_key = self.get_song_key();
        if !song_key.is_empty() {
            if let VolumeMode::PersistSeparate = self.player_details.volume_mode {
                if let Some(current_volume) = self.player_details.volume_map.get(&song_key) {
                    volume = *current_volume;
                }
            }

            if let VolumeMode::PersistClamp = self.player_details.volume_mode {
                if let Some(current_clamp) = self.player_details.clamp_map.get(&song_key) {
                    clamp = *current_clamp;
                }
            }
        }
        let maxv = (clamp).ln();
        let scale = maxv / 100f64;
        let volume = volume.clamp(0f64, 100f64);
        if volume > 0f64 {
            return volume.ln() / scale;
        }
        volume
    }

    pub fn get_raw_volume(&self) -> f64 {
        if let VolumeMode::PersistSeparate = self.player_details.volume_mode {
            let song_key = self.get_song_key();
            if !song_key.is_empty() {
                if let Some(volume) = self.player_details.volume_map.get(&song_key) {
                    return *volume;
                }
            }
        }
        self.player_details.volume
    }

    pub fn get_queue_songs(&self) -> Vec<Song> {
        self.queue
            .song_queue
            .iter()
            .map(|index| {
                self.queue
                    .data
                    .get(index)
                    .cloned()
                    .expect("Song does not exist in data")
            })
            .collect()
    }

    pub fn update_volume_mode(&mut self, mode: Vec<CheckboxPreference>) {
        for m in mode {
            if m.enabled {
                self.player_details.volume_mode = match m.key.as_str() {
                    "persist_separate" => VolumeMode::PersistSeparate,
                    "persist_clamp" => VolumeMode::PersistClamp,
                    _ => VolumeMode::Normal,
                };
                return;
            }
        }

        self.player_details.volume_mode = VolumeMode::Normal;
    }

    pub fn next_song(&mut self) {
        self.queue.current_index += 1;
        if self.queue.current_index >= self.queue.song_queue.len() {
            self.queue.current_index = 0;
        }
        self.update_current_song();
    }

    pub fn prev_song(&mut self) {
        if self.queue.current_index == 0 {
            self.queue.current_index = self.queue.song_queue.len() - 1;
        } else {
            self.queue.current_index -= 1;
        }
        self.update_current_song();
    }

    pub fn toggle_repeat(&mut self) {
        let new_mode = match self.player_details.repeat {
            RepeatModes::None => RepeatModes::Once,
            RepeatModes::Once => RepeatModes::Loop,
            RepeatModes::Loop => RepeatModes::None,
        };
        console_log!("new mode {:?}", new_mode);
        self.player_details.repeat = new_mode;
    }

    pub fn shuffle_queue(&mut self) {
        let binding = self.queue.song_queue.clone();
        let current_song = binding.get(self.queue.current_index).unwrap();
        let mut rng = rand::thread_rng();
        self.queue.song_queue.shuffle(&mut rng);
        let new_index = self
            .queue
            .song_queue
            .iter()
            .position(|v| v == current_song)
            .unwrap();
        self.queue.current_index = new_index;
    }

    pub fn clear_queue(&mut self) {
        self.queue.song_queue.clear();
        self.queue.current_index = 0;
        self.update_current_song();
    }

    pub fn blacklist_player(&mut self, key: String) {
        self.player_blacklist.push(key);
        self.force_load_song = !self.force_load_song
    }

    fn clear_blacklist(&mut self) {
        self.player_blacklist.clear();
    }
}
