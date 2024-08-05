use std::{cmp::max, cmp::min, collections::HashMap, default};

use types::{preferences::CheckboxPreference, songs::Song, ui::player_details::PlayerState};

use crate::console_log;

#[derive(Debug, Default, Clone)]
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
    volume: f64,
    volume_mode: VolumeMode,
    volume_map: HashMap<String, f64>,
    clamp_map: HashMap<String, f64>,
}

#[derive(Debug, Default)]
enum VolumeMode {
    #[default]
    Normal,
    PersistSeparate,
    PersistClamp,
}

#[derive(Debug, Default)]
pub struct PlayerStore {
    pub queue: Queue,
    pub current_song: Option<Song>,
    pub player_details: PlayerDetails,
}

impl PlayerStore {
    pub fn new() -> PlayerStore {
        PlayerStore::default()
    }

    pub fn update_current_song(&mut self) {
        if self.queue.current_index >= self.queue.song_queue.len() {
            return;
        }
        let id = self.queue.song_queue[self.queue.current_index].clone();
        let song = self.queue.data.get(&id).cloned();

        if song == self.current_song {
            return;
        }

        console_log!("Upading song in queue");
        self.current_song = song;
    }

    pub fn add_to_queue(&mut self, song: Song) {
        let song_id = song.song._id.clone().unwrap();
        self.queue.data.insert(song_id.clone(), song);
        self.queue.song_queue.push(song_id);
        self.update_current_song();
    }

    pub fn remove_from_queue(&mut self, index: usize) {
        self.queue.song_queue.remove(index);
    }

    pub fn play_now(&mut self, song: Song) {
        let song_id = song.song._id.clone().unwrap();
        self.queue.data.insert(song_id.clone(), song);
        let insertion_index = min(self.queue.song_queue.len(), self.queue.current_index + 1);
        self.queue.song_queue.insert(insertion_index, song_id);
        self.queue.current_index = insertion_index;
        self.update_current_song();
    }

    pub fn update_time(&mut self, new_time: f64) {
        self.player_details.current_time = new_time;
    }

    pub fn force_seek_percent(&mut self, new_time: f64) {
        let new_time = if let Some(current_song) = &self.current_song {
            current_song.song.duration.unwrap_or_default() * new_time
        } else {
            0f64
        };

        self.player_details.force_seek = new_time;
    }

    pub fn set_state(&mut self, state: PlayerState) {
        self.player_details.state = state;
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
                self.player_details.volume_map.insert(song_key, volume);
            }
        }
        self.player_details.volume = volume;
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
}
