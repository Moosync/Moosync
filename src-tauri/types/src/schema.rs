// @generated automatically by Diesel CLI.

diesel::table! {
    album_bridge (id) {
        id -> Nullable<Integer>,
        song -> Nullable<Text>,
        album -> Nullable<Text>,
    }
}

diesel::table! {
    albums (album_id) {
        album_id -> Nullable<Text>,
        album_name -> Nullable<Text>,
        album_artist -> Nullable<Text>,
        album_coverpath_high -> Nullable<Text>,
        album_song_count -> Double,
        year -> Nullable<Text>,
        album_coverpath_low -> Nullable<Text>,
        album_extra_info -> Nullable<Text>,
    }
}

diesel::table! {
    allsongs (_id) {
        _id -> Nullable<Text>,
        path -> Nullable<Text>,
        size -> Nullable<Double>,
        inode -> Nullable<Text>,
        deviceno -> Nullable<Text>,
        title -> Nullable<Text>,
        date -> Nullable<Text>,
        year -> Nullable<Text>,
        lyrics -> Nullable<Text>,
        releasetype -> Nullable<Text>,
        bitrate -> Nullable<Double>,
        codec -> Nullable<Text>,
        container -> Nullable<Text>,
        duration -> Nullable<Double>,
        samplerate -> Nullable<Double>,
        hash -> Nullable<Text>,
        #[sql_name = "type"]
        type_ -> Text,
        url -> Nullable<Text>,
        song_coverpath_high -> Nullable<Text>,
        playbackurl -> Nullable<Text>,
        song_coverpath_low -> Nullable<Text>,
        date_added -> Nullable<BigInt>,
        provider_extension -> Nullable<Text>,
        icon -> Nullable<Text>,
        show_in_library -> Nullable<Bool>,
        track_no -> Nullable<Double>,
        library_item -> Nullable<Bool>,
    }
}

diesel::table! {
    analytics (id) {
        id -> Nullable<Text>,
        song_id -> Nullable<Text>,
        play_count -> Nullable<Integer>,
        play_time -> Nullable<Double>,
    }
}

diesel::table! {
    artist_bridge (id) {
        id -> Nullable<Integer>,
        song -> Nullable<Text>,
        artist -> Nullable<Text>,
    }
}

diesel::table! {
    artists (artist_id) {
        artist_id -> Nullable<Text>,
        artist_mbid -> Nullable<Text>,
        artist_name -> Nullable<Text>,
        artist_coverpath -> Nullable<Text>,
        artist_song_count -> Double,
        artist_extra_info -> Nullable<Text>,
        sanitized_artist_name -> Nullable<Text>,
    }
}

diesel::table! {
    genre_bridge (id) {
        id -> Nullable<Integer>,
        song -> Nullable<Text>,
        genre -> Nullable<Text>,
    }
}

diesel::table! {
    genres (genre_id) {
        genre_id -> Nullable<Text>,
        genre_name -> Nullable<Text>,
        genre_song_count -> Double,
    }
}

diesel::table! {
    playlist_bridge (id) {
        id -> Nullable<Integer>,
        song -> Nullable<Text>,
        playlist -> Nullable<Text>,
    }
}

diesel::table! {
    playlists (playlist_id) {
        playlist_id -> Nullable<Text>,
        playlist_name -> Text,
        playlist_coverpath -> Nullable<Text>,
        playlist_song_count -> Double,
        playlist_desc -> Nullable<Text>,
        playlist_path -> Nullable<Text>,
        extension -> Nullable<Text>,
        icon -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    album_bridge,
    albums,
    allsongs,
    analytics,
    artist_bridge,
    artists,
    genre_bridge,
    genres,
    playlist_bridge,
    playlists,
);
