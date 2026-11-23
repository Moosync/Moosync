pub mod canvaz;
pub mod debounce;
pub mod errors;
pub mod image;
pub mod player;
pub mod plugin;
pub mod preferences;
pub mod providers;
pub mod subscription;
pub mod ui;
pub mod window;

#[cfg(test)]
mod debounce_test;
#[cfg(test)]
mod image_test;
#[cfg(test)]
mod lib_test;
#[cfg(test)]
mod plugin_test;
#[cfg(test)]
mod subscription_test;

#[cfg(target_os = "android")]
pub mod android;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum ScanProgress {
    STOPPED,
    PROGRESS(u8),
}

pub mod prelude {
    use std::borrow::Cow;

    use extensions_proto::moosync::types::{
        ExtensionDetail, ExtensionManifest, ExtensionProviderScope, ScopePermission,
    };
    use songs_proto::moosync::types::{
        Album, Artist, EntityResult, Genre, InnerSong, Lyrics, PlaybackUrl, Playlist, Song,
        SongType, entity_result,
    };
    use themes_proto::moosync::types::{ThemeDetails, ThemeItem};

    use crate::errors::TypesError;

    pub trait EntityResultExt {
        fn get_albums(self) -> Option<Vec<Album>>;
        fn get_artists(self) -> Option<Vec<Artist>>;
        fn get_genres(self) -> Option<Vec<Genre>>;
        fn get_playlists(self) -> Option<Vec<Playlist>>;
    }

    pub trait ThemeExt {
        fn get_theme_item_or_default(&self) -> ThemeItem;
    }

    pub trait InnerSongExt {
        fn get_type_or_default(&self) -> SongType;
    }

    impl InnerSongExt for InnerSong {
        fn get_type_or_default(&self) -> SongType {
            SongType::try_from(self.r#type).unwrap_or_else(|e| {
                tracing::error!(
                    "Failed to parse song type for song: {:?}, {}. Using SongType::Local",
                    self,
                    e
                );
                SongType::Local
            })
        }
    }

    pub fn format_duration(secs: i64) -> String {
        let minutes = secs / 60;
        let seconds = secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    pub trait SongsExt {
        fn get_id(&self) -> Option<Cow<'_, str>>;
        fn get_title(&self) -> Option<Cow<'_, str>>;
        fn get_duration_or_default(&self) -> std::time::Duration;
        fn get_cover_high(&self) -> Option<Cow<'_, str>>;
        fn get_cover_low(&self) -> Option<Cow<'_, str>>;
        fn get_playback_url(&self) -> Option<Cow<'_, str>>;
        fn get_valid_playback_url(&self) -> Option<&PlaybackUrl>;
        fn merge(&mut self, updated: &Song);
        fn get_type_or_default(&self) -> SongType;
        fn get_path(&self) -> Option<Cow<'_, str>>;
        fn get_lyrics(&self) -> Option<&Lyrics>;
        fn get_date(&self) -> Option<Cow<'_, str>>;
        fn get_artist_string(&self) -> Option<String>;
        fn get_album_string(&self) -> Option<Cow<'_, str>>;
        fn format_duration(&self) -> String {
            let duration = self.get_duration_or_default();
            if duration.as_secs() == u64::MAX {
                return "Live".into();
            }
            format_duration(duration.as_secs() as i64)
        }
    }

    impl SongsExt for Song {
        fn get_id(&self) -> Option<Cow<'_, str>> {
            self.song
                .as_ref()
                .and_then(|s| s.id.as_deref().map(Cow::Borrowed))
        }
        fn get_title(&self) -> Option<Cow<'_, str>> {
            self.song
                .as_ref()
                .and_then(|s| s.title.as_deref().map(Cow::Borrowed))
        }
        fn get_duration_or_default(&self) -> std::time::Duration {
            self.song
                .as_ref()
                .and_then(|s| {
                    s.duration
                        .as_ref()
                        .map(|d| proto_duration_to_core(d).unwrap_or_default())
                })
                .unwrap_or(std::time::Duration::ZERO)
        }
        fn get_cover_high(&self) -> Option<Cow<'_, str>> {
            self.song
                .as_ref()
                .and_then(|s| s.song_cover_path_high.as_deref().map(Cow::Borrowed))
        }
        fn get_cover_low(&self) -> Option<Cow<'_, str>> {
            let cover_low = self
                .song
                .as_ref()
                .and_then(|s| s.song_cover_path_low.as_deref().map(Cow::Borrowed));
            if cover_low.is_none() {
                return self.get_cover_high();
            }
            cover_low
        }
        fn get_playback_url(&self) -> Option<Cow<'_, str>> {
            self.song.as_ref().and_then(|s| {
                s.playback_url
                    .as_ref()
                    .map(|p| Cow::Borrowed(p.url.as_str()))
            })
        }
        fn get_valid_playback_url(&self) -> Option<&PlaybackUrl> {
            let playback_url = self.song.as_ref().and_then(|s| s.playback_url.as_ref())?;
            if playback_url.url.is_empty() {
                return None;
            }
            if let Some(expires_at) = playback_url.expires_at.as_ref() {
                let Ok(secs) = u64::try_from(expires_at.seconds) else {
                    return None;
                };
                let nanos = u32::try_from(expires_at.nanos).unwrap_or(0);
                let expire_duration = std::time::Duration::new(secs, nanos);
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                if expire_duration <= now {
                    return None;
                }
            }
            Some(playback_url)
        }
        fn merge(&mut self, updated: &Song) {
            if let Some(ref upd_inner) = updated.song {
                let tgt_inner = self.song.get_or_insert_with(Default::default);
                if upd_inner.title.is_some() {
                    tgt_inner.title = upd_inner.title.clone();
                }
                if upd_inner.path.is_some() {
                    tgt_inner.path = upd_inner.path.clone();
                }
                if upd_inner.url.is_some() {
                    tgt_inner.url = upd_inner.url.clone();
                }
                if upd_inner.playback_url.is_some() {
                    tgt_inner.playback_url = upd_inner.playback_url.clone();
                }
                if upd_inner.extension.is_some() {
                    tgt_inner.extension = upd_inner.extension.clone();
                }
                if upd_inner.song_cover_path_high.is_some() {
                    tgt_inner.song_cover_path_high = upd_inner.song_cover_path_high.clone();
                }
                if upd_inner.song_cover_path_low.is_some() {
                    tgt_inner.song_cover_path_low = upd_inner.song_cover_path_low.clone();
                }
                if upd_inner.duration.is_some() {
                    tgt_inner.duration = upd_inner.duration;
                }
                if upd_inner.lyrics.is_some() {
                    tgt_inner.lyrics = upd_inner.lyrics.clone();
                }
                if upd_inner.date.is_some() {
                    tgt_inner.date = upd_inner.date.clone();
                }
                if upd_inner.year.is_some() {
                    tgt_inner.year = upd_inner.year.clone();
                }
                if upd_inner.track_no.is_some() {
                    tgt_inner.track_no = upd_inner.track_no;
                }
            }
            if updated.album.is_some() {
                self.album = updated.album.clone();
            }
            if !updated.artists.is_empty() {
                self.artists = updated.artists.clone();
            }
            if !updated.genre.is_empty() {
                self.genre = updated.genre.clone();
            }
        }
        fn get_type_or_default(&self) -> SongType {
            self.song
                .as_ref()
                .and_then(|s| match SongType::try_from(s.r#type) {
                    Ok(t) => Some(t),
                    Err(e) => {
                        tracing::error!(
                            "Failed to parse song type for song: {:?}, {}. Using SongType::Local",
                            s,
                            e
                        );
                        None
                    }
                })
                .unwrap_or(SongType::Local)
        }
        fn get_path(&self) -> Option<Cow<'_, str>> {
            self.song
                .as_ref()
                .and_then(|s| s.path.as_deref().map(Cow::Borrowed))
        }
        fn get_lyrics(&self) -> Option<&Lyrics> {
            self.song.as_ref().and_then(|s| s.lyrics.as_ref())
        }
        fn get_date(&self) -> Option<Cow<'_, str>> {
            self.song
                .as_ref()
                .and_then(|s| s.date.as_deref().map(Cow::Borrowed))
        }

        fn get_artist_string(&self) -> Option<String> {
            if self.artists.is_empty() {
                return None;
            }

            Some(
                self.artists
                    .iter()
                    .map(|a| a.artist_name())
                    .collect::<Vec<&str>>()
                    .join(","),
            )
        }
        fn get_album_string(&self) -> Option<Cow<'_, str>> {
            self.album
                .as_ref()
                .and_then(|a| a.album_name.as_deref().map(Cow::Borrowed))
        }
    }

    #[cfg(not(target_os = "android"))]
    theme_macro::generate_theme_impl!("ui/slint/src/constants.slint");
    #[cfg(target_os = "android")]
    theme_macro::generate_theme_impl!("ui/slint/src/constants_mobile.slint");

    impl ThemeExt for ThemeDetails {
        fn get_theme_item_or_default(&self) -> ThemeItem {
            self.theme.clone().unwrap_or_else(get_default_theme_item)
        }
    }

    impl EntityResultExt for EntityResult {
        fn get_albums(self) -> Option<Vec<Album>> {
            match self.result {
                Some(entity_result::Result::Albums(album_list)) => Some(album_list.albums),
                _ => None,
            }
        }

        fn get_artists(self) -> Option<Vec<Artist>> {
            match self.result {
                Some(entity_result::Result::Artists(artist_list)) => Some(artist_list.artists),
                _ => None,
            }
        }

        fn get_genres(self) -> Option<Vec<Genre>> {
            match self.result {
                Some(entity_result::Result::Genres(genre_list)) => Some(genre_list.genres),
                _ => None,
            }
        }

        fn get_playlists(self) -> Option<Vec<Playlist>> {
            match self.result {
                Some(entity_result::Result::Playlists(playlist_list)) => {
                    Some(playlist_list.playlists)
                }
                _ => None,
            }
        }
    }

    pub trait SearchResultExt {
        fn to_songs_proto(self) -> songs_proto::moosync::types::SearchResult;
    }

    impl SearchResultExt for extensions_proto::moosync::types::RequestedSearchResultResponse {
        fn to_songs_proto(self) -> songs_proto::moosync::types::SearchResult {
            songs_proto::moosync::types::SearchResult {
                songs: self.songs,
                albums: self.albums,
                artists: self.artists,
                playlists: self.playlists,
                genres: vec![],
            }
        }
    }

    fn proto_duration_to_core(
        proto_dur: &songs_proto::duration_proto::google::protobuf::Duration,
    ) -> Result<std::time::Duration, TypesError> {
        if proto_dur.seconds < 0 || proto_dur.nanos < 0 {
            return Err(TypesError::NegativeDuration);
        }

        Ok(std::time::Duration::new(
            proto_dur.seconds as u64,
            proto_dur.nanos as u32,
        ))
    }

    pub fn core_to_proto_duration(
        rust_dur: std::time::Duration,
    ) -> songs_proto::duration_proto::google::protobuf::Duration {
        songs_proto::duration_proto::google::protobuf::Duration {
            seconds: rust_dur.as_secs() as i64,
            nanos: rust_dur.subsec_nanos() as i32,
        }
    }

    pub trait ScopeListExt {
        fn contains_scope(&self, scope: ExtensionProviderScope) -> bool;
    }

    impl ScopeListExt for [ScopePermission] {
        #[tracing::instrument(level = "trace", skip_all)]
        fn contains_scope(&self, scope: ExtensionProviderScope) -> bool {
            let scope_val = scope as i32;
            self.iter().any(|s| s.scope == scope_val)
        }
    }

    impl ScopeListExt for ExtensionDetail {
        #[tracing::instrument(level = "trace", skip_all)]
        fn contains_scope(&self, scope: ExtensionProviderScope) -> bool {
            self.scopes.contains_scope(scope)
        }
    }

    impl ScopeListExt for ExtensionManifest {
        #[tracing::instrument(level = "trace", skip_all)]
        fn contains_scope(&self, scope: ExtensionProviderScope) -> bool {
            self.scopes.contains_scope(scope)
        }
    }

    pub trait ScopePermissionExt {
        fn new(scope: ExtensionProviderScope, justification: impl Into<String>) -> Self;
    }

    impl ScopePermissionExt for ScopePermission {
        #[tracing::instrument(level = "trace", skip_all)]
        fn new(scope: ExtensionProviderScope, justification: impl Into<String>) -> Self {
            Self {
                scope: scope as i32,
                justification: justification.into(),
            }
        }
    }
}
