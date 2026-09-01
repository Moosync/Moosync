use extensions_proto::moosync::types::{ExtensionDetail, FetchedExtensionManifest};
use slint::{ComponentHandle, Model};
use songs_proto::{
    duration_proto::google::protobuf::Duration,
    moosync::types::{
        Album, Artist, Genre, InnerSong, Playlist, SearchResult as ProtoSearchResult, Song,
    },
};
use tempdir::TempDir;

use super::{default_empty_icon, default_entity_cover};
use crate::{
    AlbumModel, ArtistModel, ExtensionItem, GenreModel, MainWindow, PlaylistModel, SearchResult,
    SongModel, Theme, test_utils::run_test,
};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_song_model() {
    let song = Song {
        song: Some(InnerSong {
            id: Some("id123".to_string()),
            title: Some("Song Title".to_string()),
            path: Some("/music/test.mp3".to_string()),
            duration: Some(Duration {
                seconds: 240,
                nanos: 0,
            }),
            ..Default::default()
        }),
        album: Some(Album {
            album_id: Some("alb1".to_string()),
            album_name: Some("Album Name".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let model = SongModel::from(song);

    assert_eq!(model.id, "id123");
    assert_eq!(model.title, "Song Title");
    assert_eq!(model.album_name, "Album Name");
    assert_eq!(model.coverPathHigh.size().width, 0);
    assert_eq!(model.coverPathLow.size().width, 0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_song_model_to_song() {
    let original = Song {
        song: Some(InnerSong {
            id: Some("id123".to_string()),
            title: Some("Song Title".to_string()),
            path: Some("/music/test.mp3".to_string()),
            ..Default::default()
        }),
        album: Some(Album {
            album_id: Some("alb1".to_string()),
            album_name: Some("Album Name".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let model = SongModel::from(original);

    let reconstructed = Song::from(model);

    assert_eq!(
        reconstructed.song.as_ref().unwrap().id.as_deref(),
        Some("id123")
    );
    assert_eq!(
        reconstructed.song.as_ref().unwrap().title.as_deref(),
        Some("Song Title")
    );
    assert_eq!(
        reconstructed.song.as_ref().unwrap().path.as_deref(),
        Some("/music/test.mp3")
    );
    assert_eq!(
        reconstructed.album.as_ref().unwrap().album_id.as_deref(),
        Some("alb1")
    );
    assert_eq!(
        reconstructed.album.as_ref().unwrap().album_name.as_deref(),
        Some("Album Name")
    );
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_album_model() {
    let album = Album {
        album_id: Some("alb123".to_string()),
        album_name: Some("Greatest Hits".to_string()),
        album_coverpath_high: Some("https://example.com/cover.jpg".to_string()),
        album_song_count: 12.0,
        ..Default::default()
    };

    let model: AlbumModel = album.into();

    assert_eq!(model.id, "alb123");
    assert_eq!(model.title, "Greatest Hits");
    assert_eq!(model.coverPathUrl, "https://example.com/cover.jpg");
    assert_eq!(model.songs_count, 12);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_album_model_to_album() {
    let model = AlbumModel {
        id: "alb123".into(),
        title: "Greatest Hits".into(),
        coverPathUrl: "https://example.com/cover.jpg".into(),
        songs_count: 12,
        coverPath: default_entity_cover(),
        extension: "".into(),
        extension_icon: default_empty_icon(),
    };

    let album: Album = model.into();

    assert_eq!(album.album_id.as_deref(), Some("alb123"));
    assert_eq!(album.album_name.as_deref(), Some("Greatest Hits"));
    assert_eq!(
        album.album_coverpath_high.as_deref(),
        Some("https://example.com/cover.jpg")
    );
    assert_eq!(album.album_song_count, 12.0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_artist_model() {
    let artist = Artist {
        artist_id: Some("art123".to_string()),
        artist_name: Some("Queen".to_string()),
        artist_song_count: 50.0,
        ..Default::default()
    };

    let model = ArtistModel::from(artist);

    assert_eq!(model.id, "art123");
    assert_eq!(model.title, "Queen");
    assert_eq!(model.songs_count, 50);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_album_model_without_cover_has_placeholder() {
    let album = Album {
        album_id: Some("alb123".to_string()),
        album_name: Some("Greatest Hits".to_string()),
        album_coverpath_high: None,
        ..Default::default()
    };
    let expected = default_entity_cover();

    let model: AlbumModel = album.into();

    assert_eq!(model.id, "alb123");
    assert_eq!(model.coverPath.size(), expected.size());
    assert_ne!(model.coverPath.size().width, 0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_artist_model_without_cover_has_placeholder() {
    let artist = Artist {
        artist_id: Some("art123".to_string()),
        artist_name: Some("Queen".to_string()),
        artist_coverpath: None,
        ..Default::default()
    };
    let expected = default_entity_cover();

    let model = ArtistModel::from(artist);

    assert_eq!(model.id, "art123");
    assert_eq!(model.coverPath.size(), expected.size());
    assert_ne!(model.coverPath.size().width, 0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_genre_model() {
    let genre = Genre {
        genre_id: Some("gen123".to_string()),
        genre_name: Some("Rock".to_string()),
        genre_song_count: 15.0,
    };
    let expected = default_entity_cover();

    let model = GenreModel::from(genre);

    assert_eq!(model.id, "gen123");
    assert_eq!(model.title, "Rock");
    assert_eq!(model.songs_count, 15);
    assert_eq!(model.coverPath.size(), expected.size());
    assert_ne!(model.coverPath.size().width, 0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_playlist_model() {
    let playlist = Playlist {
        playlist_id: Some("pl123".to_string()),
        playlist_name: "Favorites".to_string(),
        playlist_song_count: 25.0,
        ..Default::default()
    };
    let expected = default_entity_cover();

    let model: PlaylistModel = playlist.into();

    assert_eq!(model.id, "pl123");
    assert_eq!(model.title, "Favorites");
    assert_eq!(model.songs_count, 25);
    assert_eq!(model.coverPath.size(), expected.size());
    assert_ne!(model.coverPath.size().width, 0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_from_playlist_model() {
    let model = PlaylistModel {
        id: "pl123".into(),
        title: "Favorites".into(),
        songs_count: 25,
        coverPath: default_entity_cover(),
        coverPathUrl: "https://example.com/cover.jpg".into(),
        extension: "local".into(),
        extension_icon: default_empty_icon(),
    };

    let playlist: Playlist = model.into();

    assert_eq!(playlist.playlist_id.as_deref(), Some("pl123"));
    assert_eq!(playlist.playlist_name, "Favorites");
    assert_eq!(
        playlist.playlist_coverpath.as_deref(),
        Some("https://example.com/cover.jpg")
    );
    assert_eq!(playlist.playlist_song_count, 25.0);
    assert_eq!(playlist.extension.as_deref(), Some("local"));
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_extension_item() {
    let detail = ExtensionDetail {
        package_name: "com.test.ext".to_string(),
        name: "Test Extension".to_string(),
        desc: Some("A test extension".to_string()),
        version: "1.0.0".to_string(),
        author: Some("Author".to_string()),
        ..Default::default()
    };

    let item = ExtensionItem::from(detail);

    assert_eq!(item.package_name, "com.test.ext");
    assert_eq!(item.name, "Test Extension");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_fetched_extension_item() {
    let manifest = FetchedExtensionManifest {
        package_name: "com.fetched.ext".to_string(),
        name: "Fetched Ext".to_string(),
        description: Some("Fetched description".to_string()),
        version: "2.0.0".to_string(),
        ..Default::default()
    };

    let fetched_item = ExtensionItem::from(manifest);

    assert_eq!(fetched_item.package_name, "com.fetched.ext");
    assert_eq!(fetched_item.name, "Fetched Ext");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_search_result() {
    run_test(|| {
        let main_window = MainWindow::new().unwrap();
        let theme = main_window.global::<Theme>();
        let proto_res = ProtoSearchResult {
            songs: vec![Song {
                song: Some(InnerSong {
                    id: Some("s1".to_string()),
                    title: Some("Song 1".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            albums: vec![Album {
                album_id: Some("a1".to_string()),
                album_name: Some("Album 1".to_string()),
                ..Default::default()
            }],
            artists: vec![Artist {
                artist_id: Some("ar1".to_string()),
                artist_name: Some("Artist 1".to_string()),
                ..Default::default()
            }],
            playlists: vec![Playlist {
                playlist_id: Some("p1".to_string()),
                playlist_name: "Playlist 1".to_string(),
                ..Default::default()
            }],
            genres: vec![Genre {
                genre_id: Some("g1".to_string()),
                genre_name: Some("Genre 1".to_string()),
                ..Default::default()
            }],
        };
        let tmp = TempDir::new("moosync_search_utils_test").unwrap();

        let result =
            SearchResult::from((proto_res, None, default_empty_icon(), &theme, tmp.path()));

        assert_eq!(result.extension, "");
        assert_eq!(result.songs.row_count(), 1);
        assert_eq!(result.albums.row_count(), 1);
        assert_eq!(result.artists.row_count(), 1);
        assert_eq!(result.playlists.row_count(), 1);
        assert_eq!(result.genres.row_count(), 1);
    });
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_from_extension_detail_default_icon() {
    let detail = ExtensionDetail {
        name: "Test Extension".to_string(),
        package_name: "test.ext".to_string(),
        version: "1.0.0".to_string(),
        active: true,
        has_started: true,
        desc: Some("A test extension".to_string()),
        extension_icon: None,
        registry: Some("local".to_string()),
        ..Default::default()
    };

    let item = ExtensionItem::from(detail);

    assert_eq!(item.name, "Test Extension");
    assert_eq!(item.icon.size().width, 0);
    assert_eq!(item.icon.size().height, 0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_from_fetched_extension_manifest_default_icon() {
    let manifest = FetchedExtensionManifest {
        name: "Remote Extension".to_string(),
        package_name: "remote.ext".to_string(),
        version: "2.0.0".to_string(),
        description: Some("Remote description".to_string()),
        logo: None,
        url: "https://example.com/ext.msox".to_string(),
        registry: Some("Community".to_string()),
    };

    let item = ExtensionItem::from(manifest);

    assert_eq!(item.name, "Remote Extension");
    assert_eq!(item.icon.size().width, 0);
    assert_eq!(item.icon.size().height, 0);
}
