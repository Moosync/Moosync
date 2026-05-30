pub use database::{Database, DatabaseInterceptor, InterceptedDatabase};
use songs_proto::moosync::types::{GetSongOptions, Song};
use types::errors::Result;

pub struct DummyDatabaseInterceptor;

impl DatabaseInterceptor for DummyDatabaseInterceptor {
    fn before_get_songs_by_options(&self, options: &GetSongOptions) -> Option<Result<Vec<Song>>> {
        println!(
            "DummyDatabaseInterceptor: [before_get_songs_by_options] options = {:?}",
            options
        );
        None
    }
}
