use player_proto::moosync::types::{PlayerData, RepeatMode};
use rand::{rng, seq::SliceRandom};
use songs_proto::moosync::types::Song;

pub trait PlayerDataExt {
    fn append_songs(&mut self, songs: Vec<Song>);
    fn insert_songs_at(&mut self, idx: usize, songs: Vec<Song>);
    fn insert_songs_next(&mut self, songs: Vec<Song>);
    fn clear_and_set(&mut self, songs: Vec<Song>);
    fn shuffle_queue(&mut self);
    fn set_repeat_mode(&mut self, mode: RepeatMode);
    fn advance_index(&mut self);
    fn rewind_index(&mut self);
    fn set_index(&mut self, idx: usize);
    fn remove_at_index(&mut self, idx: usize);
    fn clear_except_current(&mut self);
    fn reorder(&mut self, from_idx: usize, to_idx: usize);
    fn current_song(&self) -> Option<&Song>;
}

impl PlayerDataExt for PlayerData {
    #[tracing::instrument(level = "debug", skip_all)]
    fn append_songs(&mut self, songs: Vec<Song>) { self.song_queue.extend(songs); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn insert_songs_at(&mut self, idx: usize, songs: Vec<Song>) {
        let insert_pos = idx.min(self.song_queue.len());
        self.song_queue.splice(insert_pos..insert_pos, songs);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn insert_songs_next(&mut self, songs: Vec<Song>) {
        let insert_pos = (self.current_idx as usize) + 1;
        self.insert_songs_at(insert_pos, songs);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn clear_and_set(&mut self, songs: Vec<Song>) {
        self.song_queue = songs;
        self.current_idx = 0;
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn shuffle_queue(&mut self) {
        if self.song_queue.len() <= 1 {
            return;
        }
        let current_song = self.song_queue.remove(self.current_idx as usize);
        let mut rng = rng();
        self.song_queue.shuffle(&mut rng);
        self.song_queue
            .insert(self.current_idx as usize, current_song);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_repeat_mode(&mut self, mode: RepeatMode) { self.repeat_mode = mode.into(); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn advance_index(&mut self) {
        if self.song_queue.is_empty() {
            return;
        }
        if (self.current_idx as usize) + 1 < self.song_queue.len() {
            self.current_idx += 1;
        } else {
            self.current_idx = 0;
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn rewind_index(&mut self) {
        if self.song_queue.is_empty() {
            return;
        }
        if self.current_idx > 0 {
            self.current_idx -= 1;
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_index(&mut self, idx: usize) {
        if idx < self.song_queue.len() {
            self.current_idx = idx as u64;
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn remove_at_index(&mut self, idx: usize) {
        if idx >= self.song_queue.len() {
            return;
        }

        self.song_queue.remove(idx);
        if self.song_queue.is_empty() {
            self.current_idx = 0;
            return;
        }

        if idx < self.current_idx as usize {
            self.current_idx -= 1;
            return;
        }

        if (self.current_idx as usize) >= self.song_queue.len() {
            self.current_idx = (self.song_queue.len() - 1) as u64;
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn clear_except_current(&mut self) {
        if self.song_queue.len() > 1 && (self.current_idx as usize) < self.song_queue.len() {
            let song = self.song_queue.remove(self.current_idx as usize);
            self.song_queue.clear();
            self.song_queue.push(song);
            self.current_idx = 0;
        } else if self.song_queue.len() == 1 {
            self.song_queue.clear();
            self.current_idx = 0;
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn reorder(&mut self, from_idx: usize, to_idx: usize) {
        if from_idx < self.song_queue.len() && to_idx < self.song_queue.len() {
            let song = self.song_queue.remove(from_idx);
            self.song_queue.insert(to_idx, song);
            if (self.current_idx as usize) == from_idx {
                self.current_idx = to_idx as u64;
            } else if from_idx < (self.current_idx as usize)
                && to_idx >= (self.current_idx as usize)
            {
                self.current_idx -= 1;
            } else if from_idx > (self.current_idx as usize)
                && to_idx <= (self.current_idx as usize)
            {
                self.current_idx += 1;
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn current_song(&self) -> Option<&Song> { self.song_queue.get(self.current_idx as usize) }
}
