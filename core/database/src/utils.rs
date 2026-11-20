use songs_proto::moosync::types::{Album, Artist, Genre, InnerSong, Playlist, SongType};

pub trait SearchByTerm {
    fn search_by_term(term: Option<String>) -> Self;
}

impl SearchByTerm for InnerSong {
    #[tracing::instrument(level = "debug", skip_all)]
    fn search_by_term(term: Option<String>) -> Self {
        let mut data = Self::default();
        data.title.clone_from(&term);
        data.path = term;
        data
    }
}

impl SearchByTerm for Album {
    #[tracing::instrument(level = "debug", skip_all)]
    fn search_by_term(term: Option<String>) -> Self {
        Self {
            album_name: term,
            ..Default::default()
        }
    }
}

impl SearchByTerm for Artist {
    #[tracing::instrument(level = "debug", skip_all)]
    fn search_by_term(term: Option<String>) -> Self {
        Self {
            artist_name: term,
            ..Default::default()
        }
    }
}

impl SearchByTerm for Genre {
    #[tracing::instrument(level = "debug", skip_all)]
    fn search_by_term(term: Option<String>) -> Self {
        Self {
            genre_name: term,
            ..Default::default()
        }
    }
}

impl SearchByTerm for Playlist {
    #[tracing::instrument(level = "debug", skip_all)]
    fn search_by_term(term: Option<String>) -> Self {
        Self {
            playlist_name: term.unwrap_or_default(),
            ..Default::default()
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn song_type_from_str(s: &str) -> i32 {
    let t = match s {
        "LOCAL" => SongType::Local,
        "URL" => SongType::Url,
        "SPOTIFY" => SongType::Spotify,
        "DASH" => SongType::Dash,
        "HLS" => SongType::Hls,
        _ => SongType::Local,
    };
    t as i32
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn song_type_to_str(t: i32) -> &'static str {
    match SongType::try_from(t).unwrap_or(SongType::Local) {
        SongType::Local => "LOCAL",
        SongType::Url => "URL",
        SongType::Spotify => "SPOTIFY",
        SongType::Dash => "DASH",
        SongType::Hls => "HLS",
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn proto_to_db_ms(
    proto_dur: &Option<songs_proto::duration_proto::google::protobuf::Duration>,
) -> i64 {
    match proto_dur.as_ref() {
        Some(dur) => (dur.seconds * 1000) + (dur.nanos as i64 / 1_000_000),
        None => 0,
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn db_ms_to_proto(
    db_ms: i64,
) -> songs_proto::duration_proto::google::protobuf::Duration {
    songs_proto::duration_proto::google::protobuf::Duration {
        seconds: db_ms / 1000,
        nanos: ((db_ms % 1000) * 1_000_000) as i32,
    }
}
