-- Clean-slate STRICT SQLite schema for Moosync (sqlx)

CREATE TABLE albums (
    album_id TEXT PRIMARY KEY,
    album_name TEXT,
    album_artist TEXT,
    album_coverpath_high TEXT,
    album_song_count INTEGER NOT NULL DEFAULT 0,
    year TEXT,
    album_coverpath_low TEXT,
    album_extra_info TEXT
) STRICT;

CREATE TABLE artists (
    artist_id TEXT PRIMARY KEY,
    artist_mbid TEXT,
    artist_name TEXT,
    artist_coverpath TEXT,
    artist_song_count INTEGER NOT NULL DEFAULT 0,
    artist_extra_info TEXT,
    sanitized_artist_name TEXT
) STRICT;

CREATE TABLE genres (
    genre_id TEXT PRIMARY KEY,
    genre_name TEXT,
    genre_song_count INTEGER NOT NULL DEFAULT 0
) STRICT;

CREATE TABLE playlists (
    playlist_id TEXT PRIMARY KEY,
    playlist_name TEXT NOT NULL,
    playlist_coverpath TEXT,
    playlist_song_count INTEGER NOT NULL DEFAULT 0,
    playlist_desc TEXT,
    playlist_path TEXT,
    extension TEXT,
    icon TEXT,
    library_item INTEGER NOT NULL DEFAULT 1
) STRICT;

CREATE TABLE allsongs (
    _id TEXT PRIMARY KEY,
    album_id TEXT REFERENCES albums(album_id) ON DELETE SET NULL,
    path TEXT,
    size REAL,
    inode TEXT,
    deviceno TEXT,
    title TEXT,
    date TEXT,
    year TEXT,
    releasetype TEXT,
    bitrate REAL,
    codec TEXT,
    container TEXT,
    duration INTEGER,
    samplerate REAL,
    hash TEXT,
    type TEXT NOT NULL,
    url TEXT,
    song_coverpath_high TEXT,
    playbackurl BLOB,
    song_coverpath_low TEXT,
    date_added INTEGER,
    provider_extension TEXT,
    icon TEXT,
    show_in_library INTEGER,
    track_no REAL,
    library_item INTEGER NOT NULL DEFAULT 1
) STRICT;

CREATE TABLE song_lyrics (
    song_id TEXT PRIMARY KEY,
    lyrics BLOB NOT NULL
) STRICT;

CREATE TABLE artist_bridge (
    song TEXT NOT NULL REFERENCES allsongs(_id) ON DELETE CASCADE,
    artist TEXT NOT NULL REFERENCES artists(artist_id) ON DELETE CASCADE,
    PRIMARY KEY (song, artist)
) WITHOUT ROWID, STRICT;

CREATE TABLE genre_bridge (
    song TEXT NOT NULL REFERENCES allsongs(_id) ON DELETE CASCADE,
    genre TEXT NOT NULL REFERENCES genres(genre_id) ON DELETE CASCADE,
    PRIMARY KEY (song, genre)
) WITHOUT ROWID, STRICT;

CREATE TABLE playlist_bridge (
    playlist TEXT NOT NULL REFERENCES playlists(playlist_id) ON DELETE CASCADE,
    song TEXT NOT NULL REFERENCES allsongs(_id) ON DELETE CASCADE,
    position INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (playlist, song)
) WITHOUT ROWID, STRICT;

CREATE TABLE analytics (
    song_id TEXT PRIMARY KEY REFERENCES allsongs(_id) ON DELETE CASCADE,
    play_count INTEGER NOT NULL DEFAULT 0,
    play_time REAL NOT NULL DEFAULT 0.0
) STRICT;

CREATE TABLE extension_cache (
    package_name TEXT NOT NULL,
    key TEXT NOT NULL,
    data BLOB NOT NULL,
    size INTEGER NOT NULL,
    expiry_timestamp INTEGER NOT NULL,
    PRIMARY KEY (package_name, key)
) STRICT;

-- Unique & covering indexes
CREATE UNIQUE INDEX path_uq ON allsongs(path);
CREATE INDEX idx_allsongs_album_id ON allsongs(album_id);
CREATE UNIQUE INDEX sanitized_artist_name_uq ON artists(sanitized_artist_name);
CREATE UNIQUE INDEX album_name_uq ON albums(album_name);
CREATE UNIQUE INDEX genre_name_uq ON genres(genre_name);
CREATE INDEX idx_artist_bridge_artist ON artist_bridge(artist, song);
CREATE INDEX idx_genre_bridge_genre ON genre_bridge(genre, song);
CREATE INDEX idx_playlist_bridge_song ON playlist_bridge(song, playlist);
CREATE INDEX idx_cache_expiry ON extension_cache(expiry_timestamp);

-- Song count maintenance triggers
CREATE TRIGGER increment_album_count_insert
AFTER INSERT ON allsongs
WHEN NEW.album_id IS NOT NULL
BEGIN
    UPDATE albums SET album_song_count = album_song_count + 1 WHERE album_id = NEW.album_id;
END;

CREATE TRIGGER decrement_album_count_delete
AFTER DELETE ON allsongs
WHEN OLD.album_id IS NOT NULL
BEGIN
    UPDATE albums SET album_song_count = album_song_count - 1 WHERE album_id = OLD.album_id;
    DELETE FROM albums WHERE album_id = OLD.album_id AND album_song_count <= 0;
END;

CREATE TRIGGER update_album_count_change
AFTER UPDATE OF album_id ON allsongs
WHEN OLD.album_id IS NOT NEW.album_id
BEGIN
    UPDATE albums SET album_song_count = album_song_count - 1 WHERE album_id = OLD.album_id;
    DELETE FROM albums WHERE album_id = OLD.album_id AND album_song_count <= 0;
    UPDATE albums SET album_song_count = album_song_count + 1 WHERE album_id = NEW.album_id;
END;

CREATE TRIGGER increment_artist_count
AFTER INSERT ON artist_bridge
BEGIN
    UPDATE artists SET artist_song_count = artist_song_count + 1 WHERE artist_id = NEW.artist;
END;

CREATE TRIGGER decrement_artist_count
AFTER DELETE ON artist_bridge
BEGIN
    UPDATE artists SET artist_song_count = artist_song_count - 1 WHERE artist_id = OLD.artist;
    DELETE FROM artists WHERE artist_id = OLD.artist AND artist_song_count <= 0;
END;

CREATE TRIGGER increment_genre_count
AFTER INSERT ON genre_bridge
BEGIN
    UPDATE genres SET genre_song_count = genre_song_count + 1 WHERE genre_id = NEW.genre;
END;

CREATE TRIGGER decrement_genre_count
AFTER DELETE ON genre_bridge
BEGIN
    UPDATE genres SET genre_song_count = genre_song_count - 1 WHERE genre_id = OLD.genre;
    DELETE FROM genres WHERE genre_id = OLD.genre AND genre_song_count <= 0;
END;

CREATE TRIGGER increment_playlist_count
AFTER INSERT ON playlist_bridge
BEGIN
    UPDATE playlists SET playlist_song_count = playlist_song_count + 1 WHERE playlist_id = NEW.playlist;
END;

CREATE TRIGGER decrement_playlist_count
AFTER DELETE ON playlist_bridge
BEGIN
    UPDATE playlists SET playlist_song_count = playlist_song_count - 1 WHERE playlist_id = OLD.playlist;
END;

-- FTS5 full-text search index & sync triggers
CREATE VIRTUAL TABLE search_fts USING fts5(
    entity_type UNINDEXED,
    entity_id UNINDEXED,
    title
);

CREATE TRIGGER fts_song_insert AFTER INSERT ON allsongs BEGIN
    INSERT INTO search_fts (entity_type, entity_id, title)
    VALUES ('song', NEW._id, COALESCE(NEW.title, '') || ' ' || COALESCE(NEW.path, ''));
END;
CREATE TRIGGER fts_song_update AFTER UPDATE OF title, path ON allsongs BEGIN
    DELETE FROM search_fts WHERE entity_type = 'song' AND entity_id = OLD._id;
    INSERT INTO search_fts (entity_type, entity_id, title)
    VALUES ('song', NEW._id, COALESCE(NEW.title, '') || ' ' || COALESCE(NEW.path, ''));
END;
CREATE TRIGGER fts_song_delete AFTER DELETE ON allsongs BEGIN
    DELETE FROM search_fts WHERE entity_type = 'song' AND entity_id = OLD._id;
END;

CREATE TRIGGER fts_album_insert AFTER INSERT ON albums BEGIN
    INSERT INTO search_fts (entity_type, entity_id, title)
    VALUES ('album', NEW.album_id, COALESCE(NEW.album_name, ''));
END;
CREATE TRIGGER fts_album_update AFTER UPDATE OF album_name ON albums BEGIN
    DELETE FROM search_fts WHERE entity_type = 'album' AND entity_id = OLD.album_id;
    INSERT INTO search_fts (entity_type, entity_id, title)
    VALUES ('album', NEW.album_id, COALESCE(NEW.album_name, ''));
END;
CREATE TRIGGER fts_album_delete AFTER DELETE ON albums BEGIN
    DELETE FROM search_fts WHERE entity_type = 'album' AND entity_id = OLD.album_id;
END;

CREATE TRIGGER fts_artist_insert AFTER INSERT ON artists BEGIN
    INSERT INTO search_fts (entity_type, entity_id, title)
    VALUES ('artist', NEW.artist_id, COALESCE(NEW.artist_name, ''));
END;
CREATE TRIGGER fts_artist_update AFTER UPDATE OF artist_name ON artists BEGIN
    DELETE FROM search_fts WHERE entity_type = 'artist' AND entity_id = OLD.artist_id;
    INSERT INTO search_fts (entity_type, entity_id, title)
    VALUES ('artist', NEW.artist_id, COALESCE(NEW.artist_name, ''));
END;
CREATE TRIGGER fts_artist_delete AFTER DELETE ON artists BEGIN
    DELETE FROM search_fts WHERE entity_type = 'artist' AND entity_id = OLD.artist_id;
END;

CREATE TRIGGER fts_genre_insert AFTER INSERT ON genres BEGIN
    INSERT INTO search_fts (entity_type, entity_id, title)
    VALUES ('genre', NEW.genre_id, COALESCE(NEW.genre_name, ''));
END;
CREATE TRIGGER fts_genre_update AFTER UPDATE OF genre_name ON genres BEGIN
    DELETE FROM search_fts WHERE entity_type = 'genre' AND entity_id = OLD.genre_id;
    INSERT INTO search_fts (entity_type, entity_id, title)
    VALUES ('genre', NEW.genre_id, COALESCE(NEW.genre_name, ''));
END;
CREATE TRIGGER fts_genre_delete AFTER DELETE ON genres BEGIN
    DELETE FROM search_fts WHERE entity_type = 'genre' AND entity_id = OLD.genre_id;
END;

CREATE TRIGGER fts_playlist_insert AFTER INSERT ON playlists BEGIN
    INSERT INTO search_fts (entity_type, entity_id, title)
    VALUES ('playlist', NEW.playlist_id, NEW.playlist_name);
END;
CREATE TRIGGER fts_playlist_update AFTER UPDATE OF playlist_name ON playlists BEGIN
    DELETE FROM search_fts WHERE entity_type = 'playlist' AND entity_id = OLD.playlist_id;
    INSERT INTO search_fts (entity_type, entity_id, title)
    VALUES ('playlist', NEW.playlist_id, NEW.playlist_name);
END;
CREATE TRIGGER fts_playlist_delete AFTER DELETE ON playlists BEGIN
    DELETE FROM search_fts WHERE entity_type = 'playlist' AND entity_id = OLD.playlist_id;
END;
