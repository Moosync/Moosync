use std::{path::Path, time::Duration};

use extensions_proto::moosync::types::{ExtensionDetail, FetchedExtensionManifest};
use slint::{Image, Model, ModelRc, SharedString, VecModel};
use songs_proto::moosync::types::{
    Album, Artist, Genre, InnerSong, Playlist, SearchResult as ProtoSearchResult, Song,
};
use types::prelude::{SongsExt, core_to_proto_duration};

use super::{get_extension_icon, lazy_model::LazySongVecModel, load_icon};
use crate::{
    AlbumModel, ArtistModel, ExtensionItem, GenreModel, PlaylistModel, SearchResult, SongModel,
    Theme,
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
        let duration = core_to_proto_duration(Duration::from_secs(model.duration_s as u64));

        let inner_song = InnerSong {
            id: (!model.id.is_empty()).then(|| model.id.to_string()),
            path: (!model.path.is_empty()).then(|| model.path.to_string()),
            size: (model.size != 0.0).then_some(model.size as f64),
            title: (!model.title.is_empty()).then(|| model.title.to_string()),
            date: (!model.date.is_empty()).then(|| model.date.to_string()),
            year: (!model.year.is_empty()).then(|| model.year.to_string()),
            lyrics: (!model.lyrics.is_empty()).then(|| model.lyrics.to_string()),
            release_type: (!model.release_type.is_empty()).then(|| model.release_type.to_string()),
            bitrate: (model.bitrate != 0.0).then_some(model.bitrate as f64),
            codec: (!model.codec.is_empty()).then(|| model.codec.to_string()),
            container: (!model.container.is_empty()).then(|| model.container.to_string()),
            duration: (model.duration_s != 0).then_some(duration),
            sample_rate: (model.sample_rate != 0.0).then_some(model.sample_rate as f64),
            hash: (!model.hash.is_empty()).then(|| model.hash.to_string()),
            r#type: model.r#type,
            url: (!model.url.is_empty()).then(|| model.url.to_string()),
            song_cover_path_high: (!model.song_cover_path_high.is_empty())
                .then(|| model.song_cover_path_high.to_string()),
            playback_url: (!model.playback_url.is_empty()).then(|| model.playback_url.to_string()),
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

#[tracing::instrument(level = "debug", skip_all)]
pub fn resolve_extension_icon(extension: &str, detail: Option<&ExtensionDetail>) -> Image {
    if !extension.is_empty() {
        get_extension_icon(detail)
    } else {
        Image::default()
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
        let extension_icon = resolve_extension_icon(&extension, detail);
        let raw_duration = song.get_duration_or_default();
        let duration_s = raw_duration.as_secs() as i32;
        let duration_str = song.format_duration();
        let cover_url_high: SharedString =
            song.get_cover_high().as_deref().unwrap_or_default().into();
        let cover_url_low: SharedString =
            song.get_cover_low().as_deref().unwrap_or_default().into();

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
            lyrics: inner
                .as_ref()
                .and_then(|s| s.lyrics.as_deref())
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
            duration_s,
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
                .and_then(|s| s.playback_url.as_deref())
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
        let extension_icon = resolve_extension_icon(extension, detail);
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
        let extension_icon = resolve_extension_icon(extension, detail);
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
            detail
                .map(|d| get_extension_icon(Some(d)))
                .unwrap_or_else(|| {
                    playlist
                        .icon
                        .as_deref()
                        .filter(|p| !p.is_empty())
                        .map(load_icon)
                        .unwrap_or_default()
                })
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

impl From<ExtensionDetail> for ExtensionItem {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(ext: ExtensionDetail) -> Self {
        Self {
            name: ext.name.into(),
            package_name: ext.package_name.into(),
            version: ext.version.into(),
            active: ext.active,
            is_installed: true,
            loading: ext.active && !ext.has_started,
            description: ext.desc.unwrap_or_default().into(),
            icon: Image::default(),
            has_started: ext.has_started,
            icon_url: ext.extension_icon.unwrap_or_default().into(),
            registry: ext.registry.unwrap_or_else(|| "local".to_string()).into(),
        }
    }
}

impl From<FetchedExtensionManifest> for ExtensionItem {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(ext: FetchedExtensionManifest) -> Self {
        Self {
            name: ext.name.into(),
            package_name: ext.package_name.into(),
            version: ext.version.into(),
            active: false,
            is_installed: false,
            loading: false,
            description: ext.description.unwrap_or_default().into(),
            icon: Image::default(),
            has_started: false,
            icon_url: ext.logo.unwrap_or_default().into(),
            registry: ext.registry.unwrap_or_default().into(),
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn create_search_result(
    res: ProtoSearchResult,
    detail: Option<&ExtensionDetail>,
    theme: &Theme<'_>,
    cache_dir: &Path,
) -> SearchResult {
    let extension = detail.map(|d| d.package_name.as_str()).unwrap_or_default();
    let extension_icon = get_extension_icon(detail);
    SearchResult {
        albums: ModelRc::new(LazySongVecModel::new(
            res.albums.into_iter().map(|a| (a, detail).into()).collect(),
            theme.get_cardHeight() as usize,
            theme.get_cardWidth() as usize,
            cache_dir.to_path_buf(),
        )),
        artists: ModelRc::new(LazySongVecModel::new(
            res.artists
                .into_iter()
                .map(|a| (a, detail).into())
                .collect(),
            theme.get_cardHeight() as usize,
            theme.get_cardWidth() as usize,
            cache_dir.to_path_buf(),
        )),
        genres: ModelRc::new(LazySongVecModel::new(
            res.genres.into_iter().map(GenreModel::from).collect(),
            theme.get_cardHeight() as usize,
            theme.get_cardWidth() as usize,
            cache_dir.to_path_buf(),
        )),
        playlists: ModelRc::new(LazySongVecModel::new(
            res.playlists
                .into_iter()
                .map(|p| (p, detail).into())
                .collect(),
            theme.get_cardHeight() as usize,
            theme.get_cardWidth() as usize,
            cache_dir.to_path_buf(),
        )),
        songs: ModelRc::new(LazySongVecModel::new(
            res.songs.into_iter().map(|s| (s, detail).into()).collect(),
            theme.get_songListItemHeight() as usize,
            theme.get_songListItemWidth() as usize,
            cache_dir.to_path_buf(),
        )),
        extension: extension.into(),
        extension_icon,
    }
}
