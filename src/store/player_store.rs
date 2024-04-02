use std::{cmp::min, collections::HashMap};

use types::{songs::Song, ui::player_details::PlayerState};

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
    pub state: PlayerState,
    pub volume: f64,
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

    pub fn set_state(&mut self, state: PlayerState) {
        self.player_details.state = state;
    }

    pub fn set_volume(&mut self, volume: f64) {
        self.player_details.volume = volume;
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
}
