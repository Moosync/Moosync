use std::{path::Path, time::Duration};

use cache::{
    AppCacheEntry, CacheCategory as CoreCacheCategory, ExtensionCacheEntry, OrphanCacheEntry,
    OrphanCacheKind as CoreOrphanKind,
};
use extensions::resolve_env_vars;
use extensions_proto::moosync::types::{
    BinaryPermission, ExtensionAccountDetail, ExtensionDetail, ExtensionManifest,
    ExtensionProviderScope, HostPermission, ManifestPermissions, PathPermission, ScopePermission,
};
use humansize::{BINARY, format_size};
use slint::{Color, Image, Model, ModelRc, SharedString, VecModel};
use songs_proto::moosync::types::{
    Album, Artist, Genre, InnerSong, Playlist, SearchResult as ProtoSearchResult, Song,
};
use types::prelude::{SongsExt, core_to_proto_duration};

use super::{lazy_model::LazySongVecModel, load_image, parse_color, resolve_extension_icon};
use crate::{
    AccountItem, AlbumModel, ArtistModel, CacheCategory, CacheItem, ExtensionBinaryPermission,
    ExtensionCacheItem, ExtensionHostPermission, ExtensionItem, ExtensionPathPermission,
    ExtensionScopePermission, GenreModel, OrphanCacheItem, OrphanCacheKind, PlaylistModel,
    ProviderScope, SearchResult, SongModel, Theme,
};

pub trait IntoVec<T> {
    fn into_vec(self) -> Vec<T>;
}

impl<T: Clone + 'static> IntoVec<T> for ModelRc<T> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn into_vec(self) -> Vec<T> {
        (0..self.row_count())
            .filter_map(|i| self.row_data(i))
            .collect()
    }
}

impl<T: Clone + 'static> IntoVec<T> for &ModelRc<T> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn into_vec(self) -> Vec<T> { self.clone().into_vec() }
}

impl From<SongModel> for Song {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(model: SongModel) -> Self {
        let duration = core_to_proto_duration(Duration::from_millis(model.duration_ms as u64));

        let inner_song = InnerSong {
            id: (!model.id.is_empty()).then(|| model.id.to_string()),
            path: (!model.path.is_empty()).then(|| model.path.to_string()),
            size: (model.size != 0.0).then_some(model.size as f64),
            title: (!model.title.is_empty()).then(|| model.title.to_string()),
            date: (!model.date.is_empty()).then(|| model.date.to_string()),
            year: (!model.year.is_empty()).then(|| model.year.to_string()),
            lyrics: None,
            release_type: (!model.release_type.is_empty()).then(|| model.release_type.to_string()),
            bitrate: (model.bitrate != 0.0).then_some(model.bitrate as f64),
            codec: (!model.codec.is_empty()).then(|| model.codec.to_string()),
            container: (!model.container.is_empty()).then(|| model.container.to_string()),
            duration: (model.duration_ms != 0).then_some(duration),
            sample_rate: (model.sample_rate != 0.0).then_some(model.sample_rate as f64),
            hash: (!model.hash.is_empty()).then(|| model.hash.to_string()),
            r#type: model.r#type,
            url: (!model.url.is_empty()).then(|| model.url.to_string()),
            song_cover_path_high: (!model.song_cover_path_high.is_empty())
                .then(|| model.song_cover_path_high.to_string()),
            playback_url: (!model.playback_url.is_empty()).then(|| {
                songs_proto::moosync::types::PlaybackUrl {
                    url: model.playback_url.to_string(),
                    ..Default::default()
                }
            }),
            song_cover_path_low: (!model.song_cover_path_low.is_empty())
                .then(|| model.song_cover_path_low.to_string()),
            date_added: (model.date_added != 0).then_some(model.date_added as i64),
            track_no: (model.track_no != 0.0).then_some(model.track_no as f64),
            extension: (!model.extension.is_empty()).then(|| model.extension.to_string()),
        };

        let album = if model.album_id.is_empty() && model.album_name.is_empty() {
            None
        } else {
            Some(Album {
                album_id: (!model.album_id.is_empty()).then(|| model.album_id.to_string()),
                album_name: (!model.album_name.is_empty()).then(|| model.album_name.to_string()),
                album_artist: (!model.album_artist.is_empty())
                    .then(|| model.album_artist.to_string()),
                album_coverpath_high: (!model.album_coverpath_high.is_empty())
                    .then(|| model.album_coverpath_high.to_string()),
                album_coverpath_low: (!model.album_coverpath_low.is_empty())
                    .then(|| model.album_coverpath_low.to_string()),
                album_song_count: model.album_song_count as f64,
                year: (!model.album_year.is_empty()).then(|| model.album_year.to_string()),
                extension: (!model.extension.is_empty()).then(|| model.extension.to_string()),
            })
        };

        let artists: Vec<Artist> = model
            .artists
            .into_vec()
            .into_iter()
            .map(Artist::from)
            .collect();
        let genres: Vec<Genre> = model
            .genre
            .into_vec()
            .into_iter()
            .map(Genre::from)
            .collect();

        Self {
            song: Some(inner_song),
            album,
            artists,
            genre: genres,
        }
    }
}

impl From<(Song, Option<&ExtensionDetail>)> for SongModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from((song, detail): (Song, Option<&ExtensionDetail>)) -> Self {
        let extension: SharedString = detail
            .map(|d| d.package_name.as_str())
            .or_else(|| song.song.as_ref().and_then(|s| s.extension.as_deref()))
            .unwrap_or_default()
            .into();
        let extension_icon = resolve_extension_icon(detail, None);
        let raw_duration = song.get_duration_or_default();
        let duration_ms = raw_duration.as_millis() as i32;
        let duration_str = song.format_duration();
        let cover_url_high: SharedString =
            song.get_cover_high().as_deref().unwrap_or_default().into();
        let cover_url_low: SharedString =
            song.get_cover_low().as_deref().unwrap_or_default().into();

        let artists_str: SharedString = song
            .artists
            .iter()
            .filter_map(|a| a.artist_name.as_deref())
            .collect::<Vec<_>>()
            .join(", ")
            .into();
        let artists: Vec<ArtistModel> = song
            .artists
            .into_iter()
            .map(|a| (a, detail).into())
            .collect();
        let genres: Vec<GenreModel> = song.genre.into_iter().map(GenreModel::from).collect();

        let inner = song.song;
        let album = song.album;

        Self {
            id: inner
                .as_ref()
                .and_then(|s| s.id.as_deref())
                .unwrap_or_default()
                .into(),
            path: inner
                .as_ref()
                .and_then(|s| s.path.as_deref())
                .unwrap_or_default()
                .into(),
            size: inner.as_ref().and_then(|s| s.size).unwrap_or_default() as f32,
            title: inner
                .as_ref()
                .and_then(|s| s.title.as_deref())
                .unwrap_or_default()
                .into(),
            date: inner
                .as_ref()
                .and_then(|s| s.date.as_deref())
                .unwrap_or_default()
                .into(),
            year: inner
                .as_ref()
                .and_then(|s| s.year.as_deref())
                .unwrap_or_default()
                .into(),
            release_type: inner
                .as_ref()
                .and_then(|s| s.release_type.as_deref())
                .unwrap_or_default()
                .into(),
            bitrate: inner.as_ref().and_then(|s| s.bitrate).unwrap_or_default() as f32,
            codec: inner
                .as_ref()
                .and_then(|s| s.codec.as_deref())
                .unwrap_or_default()
                .into(),
            container: inner
                .as_ref()
                .and_then(|s| s.container.as_deref())
                .unwrap_or_default()
                .into(),
            duration_ms,
            duration_str: duration_str.into(),
            sample_rate: inner
                .as_ref()
                .and_then(|s| s.sample_rate)
                .unwrap_or_default() as f32,
            hash: inner
                .as_ref()
                .and_then(|s| s.hash.as_deref())
                .unwrap_or_default()
                .into(),
            r#type: inner.as_ref().map(|s| s.r#type).unwrap_or_default(),
            url: inner
                .as_ref()
                .and_then(|s| s.url.as_deref())
                .unwrap_or_default()
                .into(),
            song_cover_path_high: inner
                .as_ref()
                .and_then(|s| s.song_cover_path_high.as_deref())
                .unwrap_or_default()
                .into(),
            playback_url: inner
                .as_ref()
                .and_then(|s| s.playback_url.as_ref().map(|p| p.url.as_str()))
                .unwrap_or_default()
                .into(),
            song_cover_path_low: inner
                .as_ref()
                .and_then(|s| s.song_cover_path_low.as_deref())
                .unwrap_or_default()
                .into(),
            date_added: inner
                .as_ref()
                .and_then(|s| s.date_added)
                .unwrap_or_default() as i32,
            track_no: inner.as_ref().and_then(|s| s.track_no).unwrap_or_default() as f32,
            album_id: album
                .as_ref()
                .and_then(|a| a.album_id.as_deref())
                .unwrap_or_default()
                .into(),
            album_name: album
                .as_ref()
                .and_then(|a| a.album_name.as_deref())
                .unwrap_or_default()
                .into(),
            album_artist: album
                .as_ref()
                .and_then(|a| a.album_artist.as_deref())
                .unwrap_or_default()
                .into(),
            album_coverpath_high: album
                .as_ref()
                .and_then(|a| a.album_coverpath_high.as_deref())
                .unwrap_or_default()
                .into(),
            album_coverpath_low: album
                .as_ref()
                .and_then(|a| a.album_coverpath_low.as_deref())
                .unwrap_or_default()
                .into(),
            album_song_count: album
                .as_ref()
                .map(|a| a.album_song_count)
                .unwrap_or_default() as f32,
            album_year: album
                .as_ref()
                .and_then(|a| a.year.as_deref())
                .unwrap_or_default()
                .into(),
            artists: ModelRc::new(VecModel::from(artists)),
            artists_str,
            genre: ModelRc::new(VecModel::from(genres)),
            coverPathHigh: Image::default(),
            coverPathLow: Image::default(),
            coverPathUrlHigh: cover_url_high,
            coverPathUrlLow: cover_url_low,
            extension,
            extension_icon,
        }
    }
}

impl From<Song> for SongModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(song: Song) -> Self { (song, None).into() }
}

impl From<Album> for AlbumModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(album: Album) -> Self { (album, None).into() }
}

impl From<(Album, Option<&ExtensionDetail>)> for AlbumModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from((album, detail): (Album, Option<&ExtensionDetail>)) -> Self {
        let extension = detail
            .map(|d| d.package_name.as_str())
            .or(album.extension.as_deref())
            .unwrap_or_default();
        let extension_icon = resolve_extension_icon(detail, None);
        let cover_path_url = album.album_coverpath_high();
        Self {
            coverPath: Image::default(),
            coverPathUrl: cover_path_url.into(),
            id: album.album_id().into(),
            songs_count: album.album_song_count as i32,
            title: album.album_name().into(),
            extension: extension.into(),
            extension_icon,
        }
    }
}

impl From<AlbumModel> for Album {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(model: AlbumModel) -> Self {
        Self {
            album_id: (!model.id.is_empty()).then(|| model.id.to_string()),
            album_name: (!model.title.is_empty()).then(|| model.title.to_string()),
            album_coverpath_high: (!model.coverPathUrl.is_empty())
                .then(|| model.coverPathUrl.to_string()),
            album_coverpath_low: (!model.coverPathUrl.is_empty())
                .then(|| model.coverPathUrl.to_string()),
            album_song_count: model.songs_count as f64,
            extension: (!model.extension.is_empty()).then(|| model.extension.to_string()),
            ..Default::default()
        }
    }
}

impl From<(Artist, Option<&ExtensionDetail>)> for ArtistModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from((artist, detail): (Artist, Option<&ExtensionDetail>)) -> Self {
        let extension = detail
            .map(|d| d.package_name.as_str())
            .or(artist.extension.as_deref())
            .unwrap_or_default();
        let extension_icon = resolve_extension_icon(detail, None);
        let cover_path_url = artist.artist_coverpath.as_deref().unwrap_or_default();
        Self {
            coverPath: Image::default(),
            coverPathUrl: cover_path_url.into(),
            id: artist.artist_id.as_deref().unwrap_or_default().into(),
            songs_count: artist.artist_song_count as i32,
            title: artist.artist_name.as_deref().unwrap_or_default().into(),
            mbid: artist.artist_mbid.as_deref().unwrap_or_default().into(),
            sanitized_name: artist
                .sanitized_artist_name
                .as_deref()
                .unwrap_or_default()
                .into(),
            extension: extension.into(),
            extension_icon,
        }
    }
}

impl From<Artist> for ArtistModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(artist: Artist) -> Self { (artist, None).into() }
}

impl From<ArtistModel> for Artist {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(model: ArtistModel) -> Self {
        Self {
            artist_id: (!model.id.is_empty()).then(|| model.id.to_string()),
            artist_name: (!model.title.is_empty()).then(|| model.title.to_string()),
            artist_mbid: (!model.mbid.is_empty()).then(|| model.mbid.to_string()),
            artist_coverpath: (!model.coverPathUrl.is_empty())
                .then(|| model.coverPathUrl.to_string()),
            artist_song_count: model.songs_count as f64,
            sanitized_artist_name: (!model.sanitized_name.is_empty())
                .then(|| model.sanitized_name.to_string()),
            extension: (!model.extension.is_empty()).then(|| model.extension.to_string()),
        }
    }
}

impl From<Genre> for GenreModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(genre: Genre) -> Self {
        Self {
            coverPath: Image::default(),
            coverPathUrl: "".into(),
            id: genre.genre_id.as_deref().unwrap_or_default().into(),
            songs_count: genre.genre_song_count as i32,
            title: genre.genre_name.as_deref().unwrap_or_default().into(),
        }
    }
}

impl From<GenreModel> for Genre {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(model: GenreModel) -> Self {
        Self {
            genre_id: (!model.id.is_empty()).then(|| model.id.to_string()),
            genre_name: (!model.title.is_empty()).then(|| model.title.to_string()),
            genre_song_count: model.songs_count as f64,
        }
    }
}

impl From<Playlist> for PlaylistModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(playlist: Playlist) -> Self { (playlist, None).into() }
}

impl From<(Playlist, Option<&ExtensionDetail>)> for PlaylistModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from((playlist, detail): (Playlist, Option<&ExtensionDetail>)) -> Self {
        let extension = detail
            .map(|d| d.package_name.as_str())
            .or(playlist.extension.as_deref())
            .unwrap_or_default();
        let extension_icon = if !extension.is_empty() {
            resolve_extension_icon(detail, playlist.icon.as_deref())
        } else {
            Image::default()
        };
        let cover_path_url = playlist.playlist_coverpath.as_deref().unwrap_or_default();
        Self {
            coverPath: Image::default(),
            coverPathUrl: cover_path_url.into(),
            id: playlist.playlist_id.as_deref().unwrap_or_default().into(),
            songs_count: playlist.playlist_song_count as i32,
            title: playlist.playlist_name.into(),
            extension: extension.into(),
            extension_icon,
        }
    }
}

impl From<PlaylistModel> for Playlist {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(model: PlaylistModel) -> Self {
        Self {
            playlist_id: (!model.id.is_empty()).then(|| model.id.to_string()),
            playlist_name: model.title.to_string(),
            playlist_coverpath: (!model.coverPathUrl.is_empty())
                .then(|| model.coverPathUrl.to_string()),
            playlist_song_count: model.songs_count as f64,
            extension: (!model.extension.is_empty()).then(|| model.extension.to_string()),
            ..Default::default()
        }
    }
}

impl From<PathPermission> for ExtensionPathPermission {
    #[tracing::instrument(level = "trace", skip_all)]
    fn from(p: PathPermission) -> Self {
        Self {
            source: p.source.into(),
            target: p.target.into(),
            justification: p.justification.into(),
        }
    }
}

impl From<HostPermission> for ExtensionHostPermission {
    #[tracing::instrument(level = "trace", skip_all)]
    fn from(h: HostPermission) -> Self {
        Self {
            host: h.host.into(),
            justification: h.justification.into(),
        }
    }
}

impl From<BinaryPermission> for ExtensionBinaryPermission {
    #[tracing::instrument(level = "trace", skip_all)]
    fn from(b: BinaryPermission) -> Self {
        let mut hash_entries: Vec<String> = b
            .hashes
            .into_iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect();
        hash_entries.sort();
        Self {
            name: b.name.into(),
            hashes: hash_entries.join("\n").into(),
            justification: b.justification.into(),
        }
    }
}

impl From<ScopePermission> for ExtensionScopePermission {
    #[tracing::instrument(level = "trace", skip_all)]
    fn from(s: ScopePermission) -> Self {
        Self {
            scope: ProviderScope::from(s.scope),
            justification: s.justification.into(),
        }
    }
}

trait IntoModelRc<U: 'static> {
    fn into_model_rc(self) -> ModelRc<U>;
}

impl<T, U: From<T> + Clone + 'static> IntoModelRc<U> for Vec<T> {
    #[tracing::instrument(level = "trace", skip_all)]
    fn into_model_rc(self) -> ModelRc<U> {
        let converted: Vec<U> = self.into_iter().map(U::from).collect();
        ModelRc::new(VecModel::from(converted))
    }
}

impl From<ExtensionProviderScope> for ProviderScope {
    #[tracing::instrument(level = "trace", skip_all)]
    fn from(scope: ExtensionProviderScope) -> Self {
        match scope {
            ExtensionProviderScope::Search => ProviderScope::Search,
            ExtensionProviderScope::Playlists => ProviderScope::Playlists,
            ExtensionProviderScope::PlaylistSongs => ProviderScope::PlaylistSongs,
            ExtensionProviderScope::ArtistSongs => ProviderScope::ArtistSongs,
            ExtensionProviderScope::AlbumSongs => ProviderScope::AlbumSongs,
            ExtensionProviderScope::Recommendations => ProviderScope::Recommendations,
            ExtensionProviderScope::Scrobble => ProviderScope::Scrobble,
            ExtensionProviderScope::PlaybackDetails => ProviderScope::PlaybackDetails,
            ExtensionProviderScope::Lyrics => ProviderScope::Lyrics,
            ExtensionProviderScope::SongContextMenu => ProviderScope::SongContextMenu,
            ExtensionProviderScope::PlaylistContextMenu => ProviderScope::PlaylistContextMenu,
            ExtensionProviderScope::Accounts => ProviderScope::Accounts,
            ExtensionProviderScope::DatabaseSongEvents => ProviderScope::DatabaseSongEvents,
            ExtensionProviderScope::DatabasePlaylistEvents => ProviderScope::DatabasePlaylistEvents,
            ExtensionProviderScope::PlayerUiEvents => ProviderScope::PlayerUiEvents,
            ExtensionProviderScope::PlayerDataEvents => ProviderScope::PlayerDataEvents,
            ExtensionProviderScope::CoverArt => ProviderScope::CoverArt,
            ExtensionProviderScope::PlayerProvider => ProviderScope::PlayerProvider,
        }
    }
}

impl From<i32> for ProviderScope {
    #[tracing::instrument(level = "trace", skip_all)]
    fn from(val: i32) -> Self {
        ExtensionProviderScope::try_from(val)
            .map(ProviderScope::from)
            .unwrap_or(ProviderScope::Search)
    }
}

#[tracing::instrument(level = "trace", skip_all)]
fn resolve_permission_env_vars(
    permissions: &mut ManifestPermissions,
    package_name: &str,
    cache_dir: &Path,
) {
    let ext_cache_dir = cache_dir.join("extensions").join(package_name);
    for path in &mut permissions.paths {
        path.source = resolve_env_vars(&path.source, &ext_cache_dir);
    }
    for bin in &mut permissions.binaries {
        bin.name = resolve_env_vars(&bin.name, &ext_cache_dir);
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn extension_item_from_detail(mut ext: ExtensionDetail, cache_dir: &Path) -> ExtensionItem {
    if let Some(permissions) = &mut ext.permissions {
        resolve_permission_env_vars(permissions, &ext.package_name, cache_dir);
    }
    ExtensionItem::from(ext)
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn extension_item_from_manifest(mut ext: ExtensionManifest, cache_dir: &Path) -> ExtensionItem {
    if let Some(permissions) = &mut ext.permissions {
        resolve_permission_env_vars(permissions, &ext.package_name, cache_dir);
    }
    ExtensionItem::from(ext)
}

impl From<ExtensionDetail> for ExtensionItem {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(ext: ExtensionDetail) -> Self {
        let permissions = ext.permissions.unwrap_or_default();
        let error = ext.error.unwrap_or_default();
        let has_error = !error.is_empty();

        Self {
            name: ext.name.into(),
            package_name: ext.package_name.into(),
            version: ext.version.into(),
            is_installed: true,
            loading: !ext.has_started && !has_error,
            description: ext.desc.unwrap_or_default().into(),
            icon: Image::default(),
            has_started: ext.has_started,
            icon_url: ext.extension_icon.unwrap_or_default().into(),
            registry: ext.registry.unwrap_or_else(|| "local".to_string()).into(),
            author: ext.author.unwrap_or_default().into(),
            network_permissions: permissions.hosts.into_model_rc(),
            filesystem_permissions: permissions.paths.into_model_rc(),
            binaries_permissions: permissions.binaries.into_model_rc(),
            scopes: ext.scopes.into_model_rc(),
            error: error.into(),
            is_unsigned: false,
        }
    }
}

impl From<ExtensionManifest> for ExtensionItem {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(ext: ExtensionManifest) -> Self {
        let permissions = ext.permissions.unwrap_or_default();

        Self {
            name: ext.display_name.into(),
            package_name: ext.package_name.into(),
            version: ext.version.into(),
            is_installed: false,
            loading: false,
            description: ext.description.unwrap_or_default().into(),
            icon: Image::default(),
            has_started: false,
            icon_url: ext.icon.into(),
            registry: ext.registry.unwrap_or_default().into(),
            author: ext.author.unwrap_or_default().into(),
            network_permissions: permissions.hosts.into_model_rc(),
            filesystem_permissions: permissions.paths.into_model_rc(),
            binaries_permissions: permissions.binaries.into_model_rc(),
            scopes: ext.scopes.into_model_rc(),
            error: SharedString::default(),
            is_unsigned: false,
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn create_search_result(
    res: ProtoSearchResult,
    detail: Option<&ExtensionDetail>,
    has_account: bool,
    loading: bool,
    theme: &Theme<'_>,
    cache_dir: &Path,
) -> SearchResult {
    let extension = detail.map(|d| d.package_name.as_str()).unwrap_or_default();
    let extension_name = detail.map(|d| d.name.as_str()).unwrap_or_default();
    let extension_icon = resolve_extension_icon(detail, None);
    SearchResult {
        albums: ModelRc::new(LazySongVecModel::new(
            res.albums.into_iter().map(|a| (a, detail).into()).collect(),
            theme.get_cardHeight() as usize,
            theme.get_cardWidth() as usize,
            cache_dir.to_path_buf(),
            true,
        )),
        artists: ModelRc::new(LazySongVecModel::new(
            res.artists
                .into_iter()
                .map(|a| (a, detail).into())
                .collect(),
            theme.get_cardHeight() as usize,
            theme.get_cardWidth() as usize,
            cache_dir.to_path_buf(),
            true,
        )),
        genres: ModelRc::new(LazySongVecModel::new(
            res.genres.into_iter().map(GenreModel::from).collect(),
            theme.get_cardHeight() as usize,
            theme.get_cardWidth() as usize,
            cache_dir.to_path_buf(),
            true,
        )),
        playlists: ModelRc::new(LazySongVecModel::new(
            res.playlists
                .into_iter()
                .map(|p| (p, detail).into())
                .collect(),
            theme.get_cardHeight() as usize,
            theme.get_cardWidth() as usize,
            cache_dir.to_path_buf(),
            true,
        )),
        songs: ModelRc::new(LazySongVecModel::new(
            res.songs.into_iter().map(|s| (s, detail).into()).collect(),
            theme.get_songListItemHeight() as usize,
            theme.get_songListItemWidth() as usize,
            cache_dir.to_path_buf(),
            false,
        )),
        extension: extension.into(),
        extension_name: extension_name.into(),
        extension_icon,
        has_account,
        loading,
    }
}

impl From<ExtensionAccountDetail> for AccountItem {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(detail: ExtensionAccountDetail) -> Self {
        let icon = load_image(&detail.icon);

        Self {
            id: detail.id.into(),
            package_name: detail.package_name.into(),
            name: detail.name.into(),
            bg_color: parse_color(&detail.bg_color)
                .unwrap_or_else(|| Color::from_rgb_u8(30, 215, 96)),
            icon,
            logged_in: detail.logged_in,
            username: detail.username.unwrap_or_default().into(),
        }
    }
}

impl From<CoreCacheCategory> for CacheCategory {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(category: CoreCacheCategory) -> Self {
        match category {
            CoreCacheCategory::ImageCache => Self::ImageCache,
            CoreCacheCategory::BlurredCover => Self::BlurredCover,
            CoreCacheCategory::Thumbnails => Self::Thumbnails,
            CoreCacheCategory::Wasmtime | CoreCacheCategory::ExtensionDisk => Self::Wasmtime,
            CoreCacheCategory::Logs => Self::Logs,
        }
    }
}

impl From<CacheCategory> for CoreCacheCategory {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(category: CacheCategory) -> Self {
        match category {
            CacheCategory::ImageCache => Self::ImageCache,
            CacheCategory::BlurredCover => Self::BlurredCover,
            CacheCategory::Thumbnails => Self::Thumbnails,
            CacheCategory::Wasmtime => Self::Wasmtime,
            CacheCategory::Logs => Self::Logs,
        }
    }
}

pub(crate) struct OrphanCacheTarget(pub OrphanCacheKind, pub String);

impl From<&CoreOrphanKind> for OrphanCacheKind {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(kind: &CoreOrphanKind) -> Self {
        match kind {
            CoreOrphanKind::UntrackedDiskFile => Self::UntrackedDiskFile,
            CoreOrphanKind::StaleDbRecord => Self::StaleDbRecord,
            CoreOrphanKind::UninstalledExtensionDisk(_) => Self::UninstalledExtensionDisk,
            CoreOrphanKind::UninstalledExtensionDb(_) => Self::UninstalledExtensionDb,
            CoreOrphanKind::ExpiredExtensionDb => Self::ExpiredExtensionDb,
            CoreOrphanKind::OrphanLyrics => Self::OrphanLyrics,
        }
    }
}

impl From<OrphanCacheTarget> for CoreOrphanKind {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(OrphanCacheTarget(kind, package_name): OrphanCacheTarget) -> Self {
        match kind {
            OrphanCacheKind::UntrackedDiskFile => Self::UntrackedDiskFile,
            OrphanCacheKind::StaleDbRecord => Self::StaleDbRecord,
            OrphanCacheKind::UninstalledExtensionDisk => {
                Self::UninstalledExtensionDisk(package_name)
            }
            OrphanCacheKind::UninstalledExtensionDb => Self::UninstalledExtensionDb(package_name),
            OrphanCacheKind::ExpiredExtensionDb => Self::ExpiredExtensionDb,
            OrphanCacheKind::OrphanLyrics => Self::OrphanLyrics,
        }
    }
}

impl From<&AppCacheEntry> for CacheItem {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(entry: &AppCacheEntry) -> Self {
        Self {
            category: entry.category.into(),
            size_text: format_size(entry.size_bytes, BINARY).into(),
            entry_count: entry.entry_count as i32,
        }
    }
}

impl From<&ExtensionCacheEntry> for ExtensionCacheItem {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(entry: &ExtensionCacheEntry) -> Self {
        Self {
            package_name: entry.package_name.clone().into(),
            display_name: entry.display_name.clone().into(),
            size_text: format_size(entry.disk_bytes + entry.db_bytes, BINARY).into(),
            disk_size_text: format_size(entry.disk_bytes, BINARY).into(),
            db_size_text: format_size(entry.db_bytes, BINARY).into(),
            db_entries: entry.db_entries as i32,
        }
    }
}

impl From<&OrphanCacheEntry> for OrphanCacheItem {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(entry: &OrphanCacheEntry) -> Self {
        let package_name = match &entry.kind {
            CoreOrphanKind::UninstalledExtensionDisk(pkg)
            | CoreOrphanKind::UninstalledExtensionDb(pkg) => pkg.clone(),
            _ => String::new(),
        };
        Self {
            kind: (&entry.kind).into(),
            package_name: package_name.into(),
            size_text: format_size(entry.size_bytes, BINARY).into(),
            entry_count: entry.entry_count as i32,
        }
    }
}
