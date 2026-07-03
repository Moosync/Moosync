// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::{env::temp_dir, fs, path::PathBuf, time::Duration};

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use database::Database;
use songs_proto::moosync::types::{
    Album, Artist, Genre, GetEntityOptions, GetSongOptions, InnerSong, Playlist, SearchableSong,
    Song, SongType,
};
use uuid::Uuid;

fn get_test_db_path() -> PathBuf {
    let file_name = format!("moosync_bench_{}.db", Uuid::new_v4());
    temp_dir().join(file_name)
}

fn cleanup(db_path: &PathBuf) {
    let base_path = db_path.to_string_lossy().to_string();
    let _ = fs::remove_file(db_path);
    let _ = fs::remove_file(format!("{}-shm", base_path));
    let _ = fs::remove_file(format!("{}-wal", base_path));
}

fn generate_complex_songs(count: usize) -> Vec<Song> {
    let mut songs = Vec::with_capacity(count);
    let song_types = [
        SongType::Local,
        SongType::Url,
        SongType::Spotify,
        SongType::Dash,
        SongType::Hls,
    ];

    for i in 0..count {
        let song_type = song_types[i % song_types.len()];
        let mut artists = vec![Artist {
            artist_id: None,
            artist_name: Some(format!("Primary Artist {}", i % 10)),
            ..Default::default()
        }];
        if i % 2 == 0 {
            artists.push(Artist {
                artist_id: None,
                artist_name: Some(format!("Featuring Artist {}", i % 5)),
                ..Default::default()
            });
        }

        let mut genres = vec![Genre {
            genre_id: None,
            genre_name: Some(format!("Genre {}", i % 4)),
            ..Default::default()
        }];
        if i % 3 == 0 {
            genres.push(Genre {
                genre_id: None,
                genre_name: Some(format!("Sub-Genre {}", i % 3)),
                ..Default::default()
            });
        }

        songs.push(Song {
            song: Some(InnerSong {
                id: None,
                title: Some(format!("Complex Song {}", i)),
                path: Some(format!("/path/to/complex_song_{}.mp3", i)),
                song_cover_path_high: Some(format!("http://coverpath.com/high_{}.jpg", i)),
                song_cover_path_low: Some(format!("http://coverpath.com/low_{}.jpg", i)),
                date_added: Some(i as i64),
                size: Some(1024.0 * i as f64),
                bitrate: Some(320.0),
                codec: Some("flac".to_string()),
                duration: Some(songs_proto::duration_proto::google::protobuf::Duration {
                    seconds: 180 + (i % 60) as i64,
                    nanos: 0,
                }),
                sample_rate: Some(48000.0),
                lyrics: Some(format!("These are lyrics for song {}", i)),
                r#type: song_type.into(),
                track_no: Some(i as f64),
                ..Default::default()
            }),
            album: Some(Album {
                album_id: None,
                album_name: Some(format!("Complex Album {}", i % 15)),
                album_artist: Some(format!("Primary Artist {}", i % 10)),
                album_coverpath_high: Some(format!(
                    "http://coverpath.com/album_high_{}.jpg",
                    i % 15
                )),
                year: Some(format!("{}", 2000 + (i % 25))),
                ..Default::default()
            }),
            artists,
            genre: genres,
        });
    }
    songs
}

fn generate_playlists(count: usize) -> Vec<Playlist> {
    let mut playlists = Vec::with_capacity(count);
    for i in 0..count {
        playlists.push(Playlist {
            playlist_id: Some(format!("playlist-id-{}", i)),
            playlist_name: format!("Bench Playlist {}", i),
            playlist_desc: Some(format!("Description for playlist {}", i)),
            playlist_coverpath: Some(format!("http://coverpath.com/playlist_{}.jpg", i)),
            playlist_song_count: 0.0,
            playlist_path: Some(format!("/path/to/playlist_{}", i)),
            extension: Some("local_extension".to_string()),
            icon: Some("playlist_icon".to_string()),
            library_item: Some(true),
            ..Default::default()
        });
    }
    playlists
}

fn bench_db_ops(c: &mut Criterion) {
    // 1. Setup a shared pre-populated database for read-only benchmarks
    let shared_db_path = get_test_db_path();
    let shared_db = Database::new(shared_db_path.clone());
    let prep_songs = generate_complex_songs(1000);
    let inserted_huge = shared_db.insert_songs(prep_songs).unwrap();

    // Create 50 playlists in the shared DB
    let playlists = generate_playlists(50);
    for pl in playlists {
        shared_db.create_playlist(pl).unwrap();
    }
    // Associate 500 songs to a playlist in the shared DB
    let playlist_id = "playlist-id-5".to_string();
    shared_db
        .add_to_playlist(&playlist_id, &inserted_huge[0..500])
        .unwrap();

    // Group for Database Reads
    let mut read_group = c.benchmark_group("database_reads");
    read_group.measurement_time(Duration::from_secs(1));
    read_group.sample_size(10);

    read_group.bench_function("query_by_title", |b| {
        b.iter(|| {
            let _ = shared_db
                .get_songs_by_options(GetSongOptions {
                    song: Some(SearchableSong {
                        title: Some("%Complex Song 500%".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .unwrap();
        });
    });

    read_group.bench_function("query_by_path", |b| {
        b.iter(|| {
            let _ = shared_db
                .get_songs_by_options(GetSongOptions {
                    song: Some(SearchableSong {
                        path: Some("/path/to/complex_song_250.mp3".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .unwrap();
        });
    });

    read_group.bench_function("query_by_album", |b| {
        b.iter(|| {
            let _ = shared_db
                .get_songs_by_options(GetSongOptions {
                    album: Some(Album {
                        album_name: Some("Complex Album 5".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .unwrap();
        });
    });

    read_group.bench_function("query_by_artist", |b| {
        b.iter(|| {
            let _ = shared_db
                .get_songs_by_options(GetSongOptions {
                    artist: Some(Artist {
                        artist_name: Some("Primary Artist 3".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .unwrap();
        });
    });

    read_group.bench_function("query_by_genre", |b| {
        b.iter(|| {
            let _ = shared_db
                .get_songs_by_options(GetSongOptions {
                    genre: Some(Genre {
                        genre_name: Some("Genre 2".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .unwrap();
        });
    });

    read_group.bench_function("query_by_type", |b| {
        b.iter(|| {
            let _ = shared_db
                .get_songs_by_options(GetSongOptions {
                    song: Some(SearchableSong {
                        r#type: Some(SongType::Spotify.into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .unwrap();
        });
    });

    read_group.bench_function("query_combined", |b| {
        b.iter(|| {
            let _ = shared_db
                .get_songs_by_options(GetSongOptions {
                    song: Some(SearchableSong {
                        title: Some("%Complex Song %".to_string()),
                        r#type: Some(SongType::Spotify.into()),
                        ..Default::default()
                    }),
                    inclusive: Some(true),
                    ..Default::default()
                })
                .unwrap();
        });
    });

    read_group.bench_function("get_album_songs", |b| {
        let mut conn = shared_db.pool.get().unwrap();
        let test_album = Album {
            album_name: Some("Complex Album 5".to_string()),
            ..Default::default()
        };
        b.iter(|| {
            let _ = shared_db
                .get_album_songs(test_album.clone(), true, &mut conn)
                .unwrap();
        });
    });

    read_group.bench_function("get_artist_songs", |b| {
        let mut conn = shared_db.pool.get().unwrap();
        let test_artist = Artist {
            artist_name: Some("Primary Artist 3".to_string()),
            ..Default::default()
        };
        b.iter(|| {
            let _ = shared_db
                .get_artist_songs(test_artist.clone(), true, &mut conn)
                .unwrap();
        });
    });

    read_group.bench_function("get_genre_songs", |b| {
        let mut conn = shared_db.pool.get().unwrap();
        let test_genre = Genre {
            genre_name: Some("Genre 2".to_string()),
            ..Default::default()
        };
        b.iter(|| {
            let _ = shared_db
                .get_genre_songs(test_genre.clone(), true, &mut conn)
                .unwrap();
        });
    });

    read_group.bench_function("get_entity_albums", |b| {
        b.iter(|| {
            let _ = shared_db
                .get_entity_by_options(GetEntityOptions {
                    album: Some(Album {
                        album_name: Some("%Complex Album%".to_string()),
                        ..Default::default()
                    }),
                    inclusive: Some(true),
                    ..Default::default()
                })
                .unwrap();
        });
    });

    read_group.bench_function("get_entity_artists", |b| {
        b.iter(|| {
            let _ = shared_db
                .get_entity_by_options(GetEntityOptions {
                    artist: Some(Artist {
                        artist_name: Some("%Primary Artist%".to_string()),
                        ..Default::default()
                    }),
                    inclusive: Some(true),
                    ..Default::default()
                })
                .unwrap();
        });
    });

    read_group.bench_function("search_all", |b| {
        b.iter(|| {
            let _ = shared_db.search_all("Complex").unwrap();
        });
    });

    read_group.bench_function("get_playlist_songs_500", |b| {
        b.iter(|| {
            let _ = shared_db
                .get_songs_by_options(GetSongOptions {
                    playlist: Some(Playlist {
                        playlist_id: Some("playlist-id-5".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .unwrap();
        });
    });

    read_group.bench_function("export_playlist", |b| {
        b.iter(|| {
            let _ = shared_db.export_playlist("playlist-id-5").unwrap();
        });
    });

    read_group.finish();

    // Group for Database Updates
    let mut update_group = c.benchmark_group("database_updates");
    update_group.measurement_time(Duration::from_secs(1));
    update_group.sample_size(10);

    update_group.bench_function("update_song_single", |b| {
        let mut song_to_update = inserted_huge[0].song.clone().unwrap();
        song_to_update.title = Some("Updated Title for Bench".to_string());
        b.iter(|| {
            let _ = shared_db.update_song(&song_to_update).unwrap();
        });
    });

    update_group.bench_function("update_songs_100", |b| {
        let mut songs_to_update = Vec::new();
        for s in inserted_huge[1..101].iter() {
            let mut inner = s.song.clone().unwrap();
            inner.title = Some(format!(
                "Bulk Updated Title {}",
                inner.track_no.unwrap_or(0.0)
            ));
            songs_to_update.push(Song {
                song: Some(inner),
                album: None,
                artists: vec![],
                genre: vec![],
            });
        }
        b.iter(|| {
            let _ = shared_db.update_songs(songs_to_update.clone()).unwrap();
        });
    });

    update_group.bench_function("update_lyrics", |b| {
        let target_id = inserted_huge[0].song.as_ref().unwrap().id.clone().unwrap();
        b.iter(|| {
            let _ = shared_db
                .update_lyrics(
                    target_id.clone(),
                    "New lyrics for the benchmark".to_string(),
                )
                .unwrap();
        });
    });

    update_group.bench_function("increment_play_count", |b| {
        let target_id = inserted_huge[0].song.as_ref().unwrap().id.clone().unwrap();
        b.iter(|| {
            let _ = shared_db.increment_play_count(&target_id).unwrap();
        });
    });

    update_group.bench_function("increment_play_time", |b| {
        let target_id = inserted_huge[0].song.as_ref().unwrap().id.clone().unwrap();
        b.iter(|| {
            let _ = shared_db.increment_play_time(&target_id, 245.5).unwrap();
        });
    });

    update_group.bench_function("get_top_listened", |b| {
        b.iter(|| {
            let _ = shared_db.get_top_listened_songs().unwrap();
        });
    });

    update_group.finish();

    // Group for Database Writes (using setup/cleanup batches to avoid side-effects)
    let mut write_group = c.benchmark_group("database_writes");
    write_group.measurement_time(Duration::from_secs(1));
    write_group.sample_size(10);

    write_group.bench_function("insert_10_songs", |b| {
        b.iter_batched(
            || {
                let db_path = get_test_db_path();
                let db = Database::new(db_path.clone());
                let songs = generate_complex_songs(10);
                (db, songs, db_path)
            },
            |(db, songs, db_path)| {
                let _ = db.insert_songs(songs).unwrap();
                cleanup(&db_path);
            },
            BatchSize::SmallInput,
        );
    });

    write_group.bench_function("insert_1000_songs", |b| {
        b.iter_batched(
            || {
                let db_path = get_test_db_path();
                let db = Database::new(db_path.clone());
                let songs = generate_complex_songs(1000);
                (db, songs, db_path)
            },
            |(db, songs, db_path)| {
                let _ = db.insert_songs(songs).unwrap();
                cleanup(&db_path);
            },
            BatchSize::SmallInput,
        );
    });

    write_group.bench_function("insert_100000_songs", |b| {
        b.iter_batched(
            || {
                let db_path = get_test_db_path();
                let db = Database::new(db_path.clone());
                let songs = generate_complex_songs(100000);
                (db, songs, db_path)
            },
            |(db, songs, db_path)| {
                let _ = db.insert_songs(songs).unwrap();
                cleanup(&db_path);
            },
            BatchSize::LargeInput,
        );
    });

    write_group.bench_function("insert_1000_duplicate_songs", |b| {
        b.iter_batched(
            || {
                let db_path = get_test_db_path();
                let db = Database::new(db_path.clone());
                let songs1 = generate_complex_songs(1000);
                let _ = db.insert_songs(songs1).unwrap();
                let songs2 = generate_complex_songs(1000);
                (db, songs2, db_path)
            },
            |(db, songs, db_path)| {
                let _ = db.insert_songs(songs).unwrap();
                cleanup(&db_path);
            },
            BatchSize::SmallInput,
        );
    });

    write_group.bench_function("remove_500_songs", |b| {
        b.iter_batched(
            || {
                let db_path = get_test_db_path();
                let db = Database::new(db_path.clone());
                let songs = generate_complex_songs(1000);
                let inserted = db.insert_songs(songs).unwrap();
                let ids: Vec<String> = inserted[100..600]
                    .iter()
                    .map(|s| s.song.as_ref().unwrap().id.clone().unwrap())
                    .collect();
                (db, ids, db_path)
            },
            |(db, ids, db_path)| {
                let _ = db.remove_songs(&ids).unwrap();
                cleanup(&db_path);
            },
            BatchSize::SmallInput,
        );
    });

    write_group.finish();

    cleanup(&shared_db_path);
}

criterion_group!(benches, bench_db_ops);
criterion_main!(benches);
