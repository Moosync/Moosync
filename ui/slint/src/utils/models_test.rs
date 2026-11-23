use std::collections::HashMap;

use assertables::assert_some_eq_x;
use extensions_proto::moosync::types::{
    BinaryPermission, ExtensionDetail, ExtensionManifest, ExtensionProviderScope, HostPermission,
    ManifestPermissions, PathPermission, ScopePermission,
};
use rstest::rstest;
use slint::{ComponentHandle, Image, Model};
use songs_proto::{
    duration_proto::google::protobuf::Duration,
    moosync::types::{
        Album, Artist, Genre, InnerSong, Playlist, SearchResult as ProtoSearchResult, Song,
    },
};
use tempdir::TempDir;
use tracing_test::traced_test;
use types::prelude::ScopePermissionExt;

use crate::{
    AlbumModel, ArtistModel, ExtensionItem, GenreModel, MainWindow, PlaylistModel, ProviderScope,
    SongModel, Theme,
    test_utils::main_window,
    utils::{create_search_result, extension_item_from_detail, extension_item_from_manifest},
};

#[test]
#[traced_test]
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
            album_name: Some("Album Name".to_string()),
            album_coverpath_high: Some("https://example.com/cover.jpg".to_string()),
            ..Default::default()
        }),
        artists: vec![Artist {
            artist_name: Some("Artist Name".to_string()),
            ..Default::default()
        }],
        genre: vec![Genre {
            genre_name: Some("Rock".to_string()),
            ..Default::default()
        }],
    };

    let model = SongModel::from(song);

    assert_eq!(model.id, "id123");
    assert_eq!(model.title, "Song Title");
    assert_eq!(model.album_name, "Album Name");
    assert_eq!(model.album_coverpath_high, "https://example.com/cover.jpg");
    assert_eq!(model.artists.row_count(), 1);
    assert_eq!(model.genre.row_count(), 1);
    assert_eq!(model.duration_ms, 240_000);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_song_model_to_song() {
    let model = SongModel {
        id: "id123".into(),
        title: "Song Title".into(),
        album_name: "Album Name".into(),
        album_id: "album_id".into(),
        song_cover_path_high: "https://example.com/cover.jpg".into(),
        duration_ms: 240_000,
        ..Default::default()
    };

    let song: Song = model.into();
    let inner = song.song.unwrap();

    assert_some_eq_x!(inner.id.as_deref(), "id123");
    assert_some_eq_x!(inner.title.as_deref(), "Song Title");
    assert_eq!(song.album.unwrap().album_name.unwrap(), "Album Name");
    assert_some_eq_x!(
        inner.song_cover_path_high.as_deref(),
        "https://example.com/cover.jpg"
    );
    assert_eq!(inner.duration.unwrap().seconds, 240);
}

#[test]
#[traced_test]
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
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_album_model_to_album() {
    let model = AlbumModel {
        id: "alb123".into(),
        title: "Greatest Hits".into(),
        coverPathUrl: "https://example.com/cover.jpg".into(),
        songs_count: 12,
        coverPath: Image::default(),
        extension: "".into(),
        extension_icon: Image::default(),
    };

    let album: Album = model.into();

    assert_some_eq_x!(album.album_id.as_deref(), "alb123");
    assert_some_eq_x!(album.album_name.as_deref(), "Greatest Hits");
    assert_some_eq_x!(
        album.album_coverpath_high.as_deref(),
        "https://example.com/cover.jpg"
    );
    assert_eq!(album.album_song_count, 12.0);
}

#[test]
#[traced_test]
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
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_album_model_without_cover_has_placeholder() {
    let album = Album {
        album_id: Some("alb123".to_string()),
        album_name: Some("Greatest Hits".to_string()),
        album_coverpath_high: None,
        ..Default::default()
    };

    let model: AlbumModel = album.into();

    assert_eq!(model.id, "alb123");
    assert_eq!(model.coverPath.size().width, 0);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_artist_model_without_cover_has_placeholder() {
    let artist = Artist {
        artist_id: Some("art123".to_string()),
        artist_name: Some("Queen".to_string()),
        artist_coverpath: None,
        ..Default::default()
    };

    let model = ArtistModel::from(artist);

    assert_eq!(model.id, "art123");
    assert_eq!(model.coverPath.size().width, 0);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_genre_model() {
    let genre = Genre {
        genre_id: Some("gen123".to_string()),
        genre_name: Some("Rock".to_string()),
        genre_song_count: 15.0,
    };

    let model = GenreModel::from(genre);

    assert_eq!(model.id, "gen123");
    assert_eq!(model.title, "Rock");
    assert_eq!(model.songs_count, 15);
    assert_eq!(model.coverPath.size().width, 0);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_playlist_model() {
    let playlist = Playlist {
        playlist_id: Some("pl123".to_string()),
        playlist_name: "Favorites".to_string(),
        playlist_song_count: 25.0,
        ..Default::default()
    };

    let model: PlaylistModel = playlist.into();

    assert_eq!(model.id, "pl123");
    assert_eq!(model.title, "Favorites");
    assert_eq!(model.songs_count, 25);
    assert_eq!(model.coverPath.size().width, 0);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_from_playlist_model() {
    let model = PlaylistModel {
        id: "pl123".into(),
        title: "Favorites".into(),
        songs_count: 25,
        coverPath: Image::default(),
        coverPathUrl: "https://example.com/cover.jpg".into(),
        extension: "local".into(),
        extension_icon: Image::default(),
    };

    let playlist: Playlist = model.into();

    assert_some_eq_x!(playlist.playlist_id.as_deref(), "pl123");
    assert_eq!(playlist.playlist_name, "Favorites");
    assert_some_eq_x!(
        playlist.playlist_coverpath.as_deref(),
        "https://example.com/cover.jpg"
    );
    assert_eq!(playlist.playlist_song_count, 25.0);
    assert_some_eq_x!(playlist.extension.as_deref(), "local");
}

#[test]
#[traced_test]
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
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_extension_item_with_permissions_and_scopes() {
    let detail = ExtensionDetail {
        package_name: "com.test.ext".to_string(),
        name: "Test Extension".to_string(),
        desc: Some("A test extension".to_string()),
        version: "1.0.0".to_string(),
        author: Some("Author".to_string()),
        permissions: Some(ManifestPermissions {
            hosts: vec![HostPermission {
                host: "https://api.example.com".to_string(),
                justification: "API access".to_string(),
            }],
            paths: vec![PathPermission {
                source: "/source/path".to_string(),
                target: "/target/path".to_string(),
                justification: "Music storage".to_string(),
            }],
            binaries: vec![],
        }),
        scopes: vec![
            ScopePermission::new(ExtensionProviderScope::Search, "Search provider"),
            ScopePermission::new(ExtensionProviderScope::Playlists, "Playlist provider"),
        ],
        ..Default::default()
    };

    let item = ExtensionItem::from(detail);

    assert_eq!(item.package_name, "com.test.ext");
    assert_eq!(item.name, "Test Extension");
    assert_eq!(item.description, "A test extension");
    assert_eq!(item.version, "1.0.0");
    assert_eq!(item.author, "Author");
    assert_eq!(item.network_permissions.row_count(), 1);
    assert_eq!(
        item.network_permissions.row_data(0).unwrap().host,
        "https://api.example.com"
    );
    assert_eq!(
        item.network_permissions.row_data(0).unwrap().justification,
        "API access"
    );
    assert_eq!(item.filesystem_permissions.row_count(), 1);
    assert_eq!(
        item.filesystem_permissions.row_data(0).unwrap().source,
        "/source/path"
    );
    assert_eq!(
        item.filesystem_permissions.row_data(0).unwrap().target,
        "/target/path"
    );
    assert_eq!(
        item.filesystem_permissions
            .row_data(0)
            .unwrap()
            .justification,
        "Music storage"
    );
    assert_eq!(item.scopes.row_count(), 2);
    assert_eq!(
        item.scopes.row_data(0).unwrap().scope,
        ProviderScope::Search
    );
    assert_eq!(
        item.scopes.row_data(0).unwrap().justification,
        "Search provider"
    );
    assert_eq!(
        item.scopes.row_data(1).unwrap().scope,
        ProviderScope::Playlists
    );
    assert_eq!(
        item.scopes.row_data(1).unwrap().justification,
        "Playlist provider"
    );
}

#[rstest]
#[case(ExtensionProviderScope::Search, ProviderScope::Search)]
#[case(ExtensionProviderScope::Playlists, ProviderScope::Playlists)]
#[case(ExtensionProviderScope::PlaylistSongs, ProviderScope::PlaylistSongs)]
#[case(ExtensionProviderScope::ArtistSongs, ProviderScope::ArtistSongs)]
#[case(ExtensionProviderScope::AlbumSongs, ProviderScope::AlbumSongs)]
#[case(
    ExtensionProviderScope::Recommendations,
    ProviderScope::Recommendations
)]
#[case(ExtensionProviderScope::Scrobble, ProviderScope::Scrobble)]
#[case(
    ExtensionProviderScope::PlaybackDetails,
    ProviderScope::PlaybackDetails
)]
#[case(ExtensionProviderScope::Lyrics, ProviderScope::Lyrics)]
#[case(
    ExtensionProviderScope::SongContextMenu,
    ProviderScope::SongContextMenu
)]
#[case(
    ExtensionProviderScope::PlaylistContextMenu,
    ProviderScope::PlaylistContextMenu
)]
#[case(ExtensionProviderScope::Accounts, ProviderScope::Accounts)]
#[case(
    ExtensionProviderScope::DatabaseSongEvents,
    ProviderScope::DatabaseSongEvents
)]
#[case(
    ExtensionProviderScope::DatabasePlaylistEvents,
    ProviderScope::DatabasePlaylistEvents
)]
#[case(ExtensionProviderScope::PlayerUiEvents, ProviderScope::PlayerUiEvents)]
#[case(
    ExtensionProviderScope::PlayerDataEvents,
    ProviderScope::PlayerDataEvents
)]
#[case(ExtensionProviderScope::PlayerProvider, ProviderScope::PlayerProvider)]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_provider_scope_from_proto(
    #[case] proto_scope: ExtensionProviderScope,
    #[case] expected: ProviderScope,
) {
    let result = ProviderScope::from(proto_scope);

    assert_eq!(result, expected);
}

#[rstest]
#[case(0, ProviderScope::Search)]
#[case(1, ProviderScope::Playlists)]
#[case(999, ProviderScope::Search)]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_provider_scope_from_i32(#[case] val: i32, #[case] expected: ProviderScope) {
    let result = ProviderScope::from(val);

    assert_eq!(result, expected);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_fetched_extension_item() {
    let manifest = ExtensionManifest {
        moosync_extension: false,
        display_name: "Fetched Ext".to_string(),
        extension_entry: "".to_string(),
        author: None,
        name: "com.fetched.ext".to_string(),
        version: "2.0.0".to_string(),
        icon: "".to_string(),
        permissions: None,
        description: Some("Fetched description".to_string()),
        url: None,
        registry: None,
        ..Default::default()
    };

    let fetched_item = ExtensionItem::from(manifest);

    assert_eq!(fetched_item.package_name, "com.fetched.ext");
    assert_eq!(fetched_item.name, "Fetched Ext");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_search_result(main_window: MainWindow) {
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

    let result = create_search_result(proto_res, None, false, false, &theme, tmp.path());

    assert_eq!(result.extension, "");
    assert_eq!(result.extension_name, "");
    assert!(!result.has_account);
    assert!(!result.loading);
    assert_eq!(result.songs.row_count(), 1);
    assert_eq!(result.albums.row_count(), 1);
    assert_eq!(result.artists.row_count(), 1);
    assert_eq!(result.playlists.row_count(), 1);
    assert_eq!(result.genres.row_count(), 1);

    let detail = ExtensionDetail {
        name: "Test Extension Display Name".to_string(),
        package_name: "test.ext.package".to_string(),
        ..Default::default()
    };
    let ext_result = create_search_result(
        ProtoSearchResult::default(),
        Some(&detail),
        false,
        false,
        &theme,
        tmp.path(),
    );
    assert_eq!(ext_result.extension, "test.ext.package");
    assert_eq!(ext_result.extension_name, "Test Extension Display Name");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_from_extension_detail_default_icon() {
    let detail = ExtensionDetail {
        name: "Test Extension".to_string(),
        package_name: "test.ext".to_string(),
        version: "1.0.0".to_string(),
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
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_from_fetched_extension_manifest_default_icon() {
    let manifest = ExtensionManifest {
        moosync_extension: false,
        display_name: "Remote Extension".to_string(),
        extension_entry: "".to_string(),
        author: None,
        name: "remote.ext".to_string(),
        version: "2.0.0".to_string(),
        icon: "".to_string(),
        permissions: None,
        description: Some("Remote description".to_string()),
        url: Some("https://example.com/ext.msox".to_string()),
        registry: Some("Community".to_string()),
        ..Default::default()
    };

    let item = ExtensionItem::from(manifest);

    assert_eq!(item.name, "Remote Extension");
    assert_eq!(item.icon.size().width, 0);
    assert_eq!(item.icon.size().height, 0);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_items_default_extension_icon() {
    let detail = ExtensionDetail {
        name: "Test Ext".to_string(),
        package_name: "test.ext".to_string(),
        version: "1.0.0".to_string(),
        extension_icon: None,
        ..Default::default()
    };

    let song = Song {
        song: Some(InnerSong {
            id: Some("s1".to_string()),
            title: Some("Song 1".to_string()),
            extension: Some("test.ext".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let song_model = SongModel::from((song, Some(&detail)));
    assert_eq!(song_model.extension, "test.ext");
    assert_eq!(song_model.extension_icon.size().width, 0);

    let album = Album {
        album_id: Some("a1".to_string()),
        album_name: Some("Album 1".to_string()),
        extension: Some("test.ext".to_string()),
        ..Default::default()
    };
    let album_model = AlbumModel::from((album, Some(&detail)));
    assert_eq!(album_model.extension, "test.ext");
    assert_eq!(album_model.extension_icon.size().width, 0);

    let artist = Artist {
        artist_id: Some("ar1".to_string()),
        artist_name: Some("Artist 1".to_string()),
        extension: Some("test.ext".to_string()),
        ..Default::default()
    };
    let artist_model = ArtistModel::from((artist, Some(&detail)));
    assert_eq!(artist_model.extension, "test.ext");
    assert_eq!(artist_model.extension_icon.size().width, 0);

    let playlist = Playlist {
        playlist_id: Some("p1".to_string()),
        playlist_name: "Playlist 1".to_string(),
        extension: Some("test.ext".to_string()),
        ..Default::default()
    };
    let playlist_model = PlaylistModel::from((playlist, Some(&detail)));
    assert_eq!(playlist_model.extension, "test.ext");
    assert_eq!(playlist_model.extension_icon.size().width, 0);

    let local_song = Song {
        song: Some(InnerSong {
            id: Some("ls1".to_string()),
            title: Some("Local Song".to_string()),
            extension: None,
            ..Default::default()
        }),
        ..Default::default()
    };
    let local_song_model = SongModel::from(local_song);
    assert_eq!(local_song_model.extension, "");
    assert_eq!(local_song_model.extension_icon.size().width, 0);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_item_from_manifest_permissions_mapping() {
    let mut hashes = HashMap::new();
    hashes.insert("sha256".to_string(), "abc256".to_string());
    hashes.insert("sha512".to_string(), "def512".to_string());
    hashes.insert("md5".to_string(), "ghi012".to_string());

    let manifest = ExtensionManifest {
        name: "test.pkg".to_string(),
        display_name: "Test Display".to_string(),
        version: "1.2.3".to_string(),
        description: Some("Test description".to_string()),
        author: Some("Test Author".to_string()),
        icon: "icon.svg".to_string(),
        permissions: Some(ManifestPermissions {
            hosts: vec![HostPermission {
                host: "api.example.com".to_string(),
                justification: "Connect to API".to_string(),
            }],
            paths: vec![PathPermission {
                source: "/host/cache".to_string(),
                target: "/guest/cache".to_string(),
                justification: "Store cache".to_string(),
            }],
            binaries: vec![BinaryPermission {
                name: "/host/cache/bin".to_string(),
                hashes,
                justification: "Run helper".to_string(),
            }],
        }),
        scopes: vec![ScopePermission::new(
            ExtensionProviderScope::Search,
            "Search catalog",
        )],
        ..Default::default()
    };

    let item = ExtensionItem::from(manifest);

    assert_eq!(item.name, "Test Display");
    assert_eq!(item.package_name, "test.pkg");
    assert_eq!(item.version, "1.2.3");
    assert_eq!(item.network_permissions.row_count(), 1);
    let net = item.network_permissions.row_data(0).unwrap();
    assert_eq!(net.host, "api.example.com");
    assert_eq!(net.justification, "Connect to API");

    assert_eq!(item.filesystem_permissions.row_count(), 1);
    let path = item.filesystem_permissions.row_data(0).unwrap();
    assert_eq!(path.source, "/host/cache");
    assert_eq!(path.target, "/guest/cache");
    assert_eq!(path.justification, "Store cache");

    assert_eq!(item.binaries_permissions.row_count(), 1);
    let bin = item.binaries_permissions.row_data(0).unwrap();
    assert_eq!(bin.name, "/host/cache/bin");
    assert!(bin.hashes.contains("sha256: abc256"));
    assert!(bin.hashes.contains("sha512: def512"));
    assert!(bin.hashes.contains("md5: ghi012"));
    assert_eq!(bin.justification, "Run helper");

    assert_eq!(item.scopes.row_count(), 1);
    let sc = item.scopes.row_data(0).unwrap();
    assert_eq!(sc.scope, ProviderScope::Search);
    assert_eq!(sc.justification, "Search catalog");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_item_from_detail_permissions_mapping() {
    let mut hashes = HashMap::new();
    hashes.insert("sha256".to_string(), "samplehash".to_string());

    let detail = ExtensionDetail {
        name: "Installed Ext".to_string(),
        package_name: "installed.ext".to_string(),
        version: "2.0.0".to_string(),
        has_started: true,
        permissions: Some(ManifestPermissions {
            hosts: vec![HostPermission {
                host: "api.moosync.app".to_string(),
                justification: "Sync account".to_string(),
            }],
            paths: vec![PathPermission {
                source: "/data".to_string(),
                target: "/media".to_string(),
                justification: "Access media".to_string(),
            }],
            binaries: vec![BinaryPermission {
                name: "/bin/helper".to_string(),
                hashes,
                justification: "Execute audio helper".to_string(),
            }],
        }),
        scopes: vec![ScopePermission::new(
            ExtensionProviderScope::Playlists,
            "Playlist provider",
        )],
        ..Default::default()
    };

    let item = ExtensionItem::from(detail);

    assert_eq!(item.name, "Installed Ext");
    assert_eq!(item.package_name, "installed.ext");
    assert!(item.is_installed);
    assert!(item.has_started);
    assert_eq!(item.network_permissions.row_count(), 1);
    assert_eq!(
        item.network_permissions.row_data(0).unwrap().justification,
        "Sync account"
    );
    assert_eq!(item.filesystem_permissions.row_count(), 1);
    assert_eq!(
        item.filesystem_permissions
            .row_data(0)
            .unwrap()
            .justification,
        "Access media"
    );
    assert_eq!(item.binaries_permissions.row_count(), 1);
    assert_eq!(
        item.binaries_permissions.row_data(0).unwrap().justification,
        "Execute audio helper"
    );
    assert_eq!(item.scopes.row_count(), 1);
    assert_eq!(
        item.scopes.row_data(0).unwrap().justification,
        "Playlist provider"
    );
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_item_resolves_cache_dir_and_env_vars() {
    let temp_dir = TempDir::new("moosync_models_env_test").unwrap();
    let cache_dir = temp_dir.path();
    let mut hashes = HashMap::new();
    hashes.insert("sha256".to_string(), "deadbeef".to_string());

    let manifest = ExtensionManifest {
        name: "env.test.ext".to_string(),
        display_name: "Env Ext".to_string(),
        permissions: Some(ManifestPermissions {
            paths: vec![PathPermission {
                source: "{CACHE_DIR}/bin".to_string(),
                target: "/bin".to_string(),
                justification: "Cache binaries".to_string(),
            }],
            binaries: vec![BinaryPermission {
                name: "{CACHE_DIR}/bin/yt-dlp".to_string(),
                hashes,
                justification: "Download".to_string(),
            }],
            ..Default::default()
        }),
        ..Default::default()
    };

    let detail = ExtensionDetail {
        package_name: "env.test.ext".to_string(),
        name: "Env Ext".to_string(),
        permissions: manifest.permissions.clone(),
        ..Default::default()
    };

    let expected_cache = cache_dir.join("extensions").join("env.test.ext");
    let expected_bin_dir = format!("{}/bin", expected_cache.to_string_lossy());
    let expected_bin_path = format!("{}/bin/yt-dlp", expected_cache.to_string_lossy());

    let manifest_item = extension_item_from_manifest(manifest, cache_dir);
    assert_eq!(
        manifest_item
            .filesystem_permissions
            .row_data(0)
            .unwrap()
            .source,
        expected_bin_dir
    );
    assert_eq!(
        manifest_item.binaries_permissions.row_data(0).unwrap().name,
        expected_bin_path
    );

    let detail_item = extension_item_from_detail(detail, cache_dir);
    assert_eq!(
        detail_item
            .filesystem_permissions
            .row_data(0)
            .unwrap()
            .source,
        expected_bin_dir
    );
    assert_eq!(
        detail_item.binaries_permissions.row_data(0).unwrap().name,
        expected_bin_path
    );
}
