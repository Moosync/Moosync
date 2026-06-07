use std::sync::{Arc, Mutex};

pub use database::DatabaseInterceptor;
use lru::LruCache;
use songs_proto::moosync::types::Song;
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
                cache.put(song.get_id().unwrap().to_string(), song.clone());
            }
        }
    }
}
