use extensions_proto::moosync::types::{
    ContextMenuActionRequest, ContextMenuReturnType, ExtensionCommand, ExtensionCommandResponse,
    RequestedAlbumSongsResponse, RequestedArtistSongsResponse,
    RequestedPlaylistContextMenuResponse, RequestedPlaylistSongsResponse,
    RequestedPlaylistsResponse, RequestedRecommendationsResponse, RequestedSearchResultResponse,
    RequestedSongContextMenuResponse, RequestedSongFromIdResponse, extension_command,
    extension_command_response,
};
use songs_proto::moosync::types::{Album, Artist, Playlist, Song};

pub trait Sanitize {
    fn sanitize(self, ext_name: &str) -> Self;
}

impl Sanitize for ExtensionCommand {
    /// Sanitizes an outgoing extension command before dispatching to the
    /// extension.
    #[tracing::instrument(level = "debug", skip_all)]
    fn sanitize(mut self, ext_name: &str) -> Self {
        self.event = self.event.map(|e| e.sanitize(ext_name));
        self
    }
}

impl Sanitize for extension_command::Event {
    /// Sanitizes the inner event of an outgoing extension command.
    #[tracing::instrument(level = "debug", skip_all)]
    fn sanitize(self, ext_name: &str) -> Self {
        match self {
            Self::ContextMenuAction(req) => Self::ContextMenuAction(req.sanitize(ext_name)),
            other => other,
        }
    }
}

impl Sanitize for ContextMenuActionRequest {
    /// Strips the extension namespace prefix from a context menu action ID.
    #[tracing::instrument(level = "debug", skip_all)]
    fn sanitize(mut self, ext_name: &str) -> Self {
        let stripped = self
            .action_id
            .strip_prefix(&format!("extension:{ext_name}:"))
            .or_else(|| self.action_id.strip_prefix(&format!("{ext_name}:")));
        if let Some(id) = stripped {
            self.action_id = id.to_string();
        }
        self
    }
}

impl Sanitize for ExtensionCommandResponse {
    /// Sanitizes an incoming extension command response by prefixing entity and
    /// action IDs.
    #[tracing::instrument(level = "debug", skip_all)]
    fn sanitize(mut self, ext_name: &str) -> Self {
        self.response = self.response.map(|r| r.sanitize(ext_name));
        self
    }
}

impl Sanitize for extension_command_response::Response {
    /// Dispatches sanitization to the concrete extension response variant.
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
            Self::RequestedSongFromId(resp) => Self::RequestedSongFromId(resp.sanitize(ext_name)),
            Self::RequestedRecommendations(resp) => {
                Self::RequestedRecommendations(resp.sanitize(ext_name))
            }
            Self::RequestedSongContextMenu(resp) => {
                Self::RequestedSongContextMenu(resp.sanitize(ext_name))
            }
            Self::RequestedPlaylistContextMenu(resp) => {
                Self::RequestedPlaylistContextMenu(resp.sanitize(ext_name))
            }
            other => other,
        }
    }
}

impl Sanitize for RequestedPlaylistsResponse {
    /// Sanitizes all playlists in a `RequestedPlaylistsResponse`.
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
                /// Sanitizes all songs in the response.
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
    /// Sanitizes songs, playlists, artists, and albums in a search result
    /// response.
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

impl Sanitize for RequestedSongFromIdResponse {
    /// Sanitizes the song returned from an ID lookup response.
    #[tracing::instrument(level = "debug", skip_all)]
    fn sanitize(mut self, ext_name: &str) -> Self {
        if let Some(song) = &mut self.song {
            sanitize_song(song, ext_name);
        }
        self
    }
}

impl Sanitize for RequestedSongContextMenuResponse {
    /// Sanitizes the song context menu item returned by an extension.
    #[tracing::instrument(level = "debug", skip_all)]
    fn sanitize(mut self, ext_name: &str) -> Self {
        if let Some(menu) = &mut self.menu {
            sanitize_context_menu(menu, ext_name);
        }
        self
    }
}

impl Sanitize for RequestedPlaylistContextMenuResponse {
    /// Sanitizes the playlist context menu item returned by an extension.
    #[tracing::instrument(level = "debug", skip_all)]
    fn sanitize(mut self, ext_name: &str) -> Self {
        if let Some(menu) = &mut self.menu {
            sanitize_context_menu(menu, ext_name);
        }
        self
    }
}

/// Prefixes a context menu item's `action_id` with `extension:<ext_name>:`.
#[tracing::instrument(level = "debug", skip_all)]
pub fn sanitize_context_menu(menu: &mut ContextMenuReturnType, ext_name: &str) {
    if !menu.action_id.is_empty() {
        let prefix = format!("extension:{ext_name}:");
        if !menu.action_id.starts_with(&prefix) {
            menu.action_id = format!("{prefix}{}", menu.action_id);
        }
    }
}

/// Prefixes an optional entity ID with `<ext_name>:` if not already prefixed.
#[tracing::instrument(level = "debug", skip_all)]
fn sanitize_id(id: Option<String>, ext_name: &str) -> Option<String> {
    let id = id?;
    let prefix = format!("{ext_name}:");
    if id.starts_with(&prefix) {
        return Some(id);
    }
    Some(format!("{prefix}{id}"))
}

/// Sanitizes an entity's ID and sets its source extension package name.
#[tracing::instrument(level = "debug", skip_all)]
fn sanitize_entity(id: &mut Option<String>, extension: &mut Option<String>, ext_name: &str) {
    *id = sanitize_id(id.take(), ext_name);
    *extension = Some(ext_name.to_string());
}

/// Sanitizes an artist's ID and extension field.
#[tracing::instrument(level = "debug", skip_all)]
pub fn sanitize_artist(artist: &mut Artist, ext_name: &str) {
    sanitize_entity(&mut artist.artist_id, &mut artist.extension, ext_name);
}

/// Sanitizes an album's ID and extension field.
#[tracing::instrument(level = "debug", skip_all)]
pub fn sanitize_album(album: &mut Album, ext_name: &str) {
    sanitize_entity(&mut album.album_id, &mut album.extension, ext_name);
}

/// Sanitizes a playlist's ID and extension field.
#[tracing::instrument(level = "debug", skip_all)]
pub fn sanitize_playlist(playlist: &mut Playlist, ext_name: &str) {
    sanitize_entity(&mut playlist.playlist_id, &mut playlist.extension, ext_name);
}

/// Sanitizes a song and its nested album and artist entities.
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

/// Sanitizes an extension account detail's ID and package name.
#[tracing::instrument(level = "debug", skip_all)]
pub fn sanitize_account(
    account: &mut extensions_proto::moosync::types::ExtensionAccountDetail,
    ext_name: &str,
) {
    account.id = sanitize_id(Some(std::mem::take(&mut account.id)), ext_name).unwrap_or_default();
    account.package_name = ext_name.to_string();
}
