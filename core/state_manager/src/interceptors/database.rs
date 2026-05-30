use std::{
    io::Read,
    sync::{Arc, Mutex, RwLock},
};

pub use database::{Database, DatabaseInterceptor, InterceptedDatabase};
use lru::LruCache;
use songs_proto::moosync::types::{GetSongOptions, Song};
use tracing::trace;
use types::{errors::Result, plugin::CallContext, prelude::SongsExt};

pub struct CacheDatabaseInterceptor {
    pub cache: Arc<Mutex<LruCache<String, Song>>>,
}

impl CacheDatabaseInterceptor {
    pub fn new(cache: &Arc<Mutex<LruCache<String, Song>>>) -> Self {
        Self {
            cache: cache.clone(),
        }
    }
}

impl DatabaseInterceptor for CacheDatabaseInterceptor {
    fn after_get_songs_by_options(&self, _: &mut CallContext, result: &mut Result<Vec<Song>>) {
        if let Ok(songs) = result {
            let mut cache = self.cache.lock().unwrap();
            for song in songs {
                cache.put(song.get_id().unwrap(), song.clone());
            }
        }
    }
}
