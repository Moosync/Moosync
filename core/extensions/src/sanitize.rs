use extensions_proto::moosync::types::{
    ExtensionCommandResponse, RequestedAlbumSongsResponse, RequestedArtistSongsResponse,
    RequestedPlaylistFromUrlResponse, RequestedPlaylistSongsResponse, RequestedPlaylistsResponse,
    RequestedRecommendationsResponse, RequestedSearchResultResponse, RequestedSongFromIdResponse,
    RequestedSongFromUrlResponse, extension_command_response,
};
use songs_proto::moosync::types::{Album, Artist, Playlist, Song};

pub trait Sanitize {
    fn sanitize(self, ext_name: &str) -> Self;
}

impl Sanitize for ExtensionCommandResponse {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sanitize(mut self, ext_name: &str) -> Self {
        self.response = self.response.map(|r| r.sanitize(ext_name));
        self
    }
}

impl Sanitize for extension_command_response::Response {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sanitize(self, ext_name: &str) -> Self {
        match self {
            Self::RequestedPlaylists(resp) => Self::RequestedPlaylists(resp.sanitize(ext_name)),
            Self::RequestedPlaylistSongs(resp) => {
                Self::RequestedPlaylistSongs(resp.sanitize(ext_name))
            }
            Self::RequestedArtistSongs(resp) => Self::RequestedArtistSongs(resp.sanitize(ext_name)),
            Self::RequestedAlbumSongs(resp) => Self::RequestedAlbumSongs(resp.sanitize(ext_name)),
            Self::RequestedSearchResult(resp) => {
                Self::RequestedSearchResult(resp.sanitize(ext_name))
            }
            Self::RequestedSongFromUrl(resp) => Self::RequestedSongFromUrl(resp.sanitize(ext_name)),
            Self::RequestedPlaylistFromUrl(resp) => {
                Self::RequestedPlaylistFromUrl(resp.sanitize(ext_name))
            }
            Self::RequestedSongFromId(resp) => Self::RequestedSongFromId(resp.sanitize(ext_name)),
            Self::RequestedRecommendations(resp) => {
                Self::RequestedRecommendations(resp.sanitize(ext_name))
            }
            other => other,
        }
    }
}

impl Sanitize for RequestedPlaylistsResponse {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sanitize(mut self, ext_name: &str) -> Self {
        for playlist in &mut self.playlists {
            sanitize_playlist(playlist, ext_name);
        }
        self
    }
}

macro_rules! impl_sanitize_songs_response {
    ($($t:ty),*) => {
        $(
            impl Sanitize for $t {
                #[tracing::instrument(level = "debug", skip_all)]
                fn sanitize(mut self, ext_name: &str) -> Self {
                    for song in &mut self.songs {
                        sanitize_song(song, ext_name);
                    }
                    self
                }
            }
        )*
    };
}

impl_sanitize_songs_response!(
    RequestedPlaylistSongsResponse,
    RequestedArtistSongsResponse,
    RequestedAlbumSongsResponse,
    RequestedRecommendationsResponse
);

impl Sanitize for RequestedSearchResultResponse {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sanitize(mut self, ext_name: &str) -> Self {
        for song in &mut self.songs {
            sanitize_song(song, ext_name);
        }
        for playlist in &mut self.playlists {
            sanitize_playlist(playlist, ext_name);
        }
        for artist in &mut self.artists {
            sanitize_artist(artist, ext_name);
        }
        for album in &mut self.albums {
            sanitize_album(album, ext_name);
        }
        self
    }
}

impl Sanitize for RequestedSongFromUrlResponse {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sanitize(mut self, ext_name: &str) -> Self {
        if let Some(song) = &mut self.song {
            sanitize_song(song, ext_name);
        }
        self
    }
}

impl Sanitize for RequestedPlaylistFromUrlResponse {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sanitize(mut self, ext_name: &str) -> Self {
        if let Some(playlist) = &mut self.playlist {
            sanitize_playlist(playlist, ext_name);
        }
        for song in &mut self.songs {
            sanitize_song(song, ext_name);
        }
        self
    }
}

impl Sanitize for RequestedSongFromIdResponse {
    #[tracing::instrument(level = "debug", skip_all)]
    fn sanitize(mut self, ext_name: &str) -> Self {
        if let Some(song) = &mut self.song {
            sanitize_song(song, ext_name);
        }
        self
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn sanitize_id(id: Option<String>, ext_name: &str) -> Option<String> {
    let id = id?;
    let prefix = format!("{ext_name}:");
    if id.starts_with(&prefix) {
        return Some(id);
    }
    Some(format!("{prefix}{id}"))
}

#[tracing::instrument(level = "debug", skip_all)]
fn sanitize_entity(id: &mut Option<String>, extension: &mut Option<String>, ext_name: &str) {
    *id = sanitize_id(id.take(), ext_name);
    *extension = Some(ext_name.to_string());
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn sanitize_artist(artist: &mut Artist, ext_name: &str) {
    sanitize_entity(&mut artist.artist_id, &mut artist.extension, ext_name);
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn sanitize_album(album: &mut Album, ext_name: &str) {
    sanitize_entity(&mut album.album_id, &mut album.extension, ext_name);
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn sanitize_playlist(playlist: &mut Playlist, ext_name: &str) {
    sanitize_entity(&mut playlist.playlist_id, &mut playlist.extension, ext_name);
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn sanitize_song(song: &mut Song, ext_name: &str) {
    if let Some(inner) = song.song.as_mut() {
        inner.id = sanitize_id(inner.id.take(), ext_name);
        inner.extension = Some(ext_name.to_string());
    }
    if let Some(album) = song.album.as_mut() {
        sanitize_album(album, ext_name);
    }
    for artist in &mut song.artists {
        sanitize_artist(artist, ext_name);
    }
}
