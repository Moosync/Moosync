use songs_proto::moosync::types::Song;
use tracing::debug;

pub struct QueueManager {
    song_queue: Vec<Song>,
    current_idx: usize,
}

#[plugin_macro::generate]
impl QueueManager {
    pub fn new() -> Self {
        QueueManager {
            song_queue: vec![],
            current_idx: 0,
        }
    }

    pub fn add_song(&mut self, song: Song) {
        self.song_queue.push(song);
    }

    pub fn play_now(&mut self, song: Song) {
        debug!("Playing song {:?}", song);
    }

    pub fn get_current_song(&self) -> Option<&Song> {
        self.song_queue.get(self.current_idx)
    }
}

impl types::plugin::Plugin for QueueManager {
    fn init(_context: &types::plugin::PluginContext) -> Self {
        QueueManager::new()
    }
}
