-- Your SQL goes here
CREATE TRIGGER increment_artist_count
AFTER
INSERT
    ON artist_bridge BEGIN
UPDATE
    artists
SET
    artist_song_count = artist_song_count + 1
WHERE
    artist_id = NEW.artist;

END;

CREATE TRIGGER decrement_artist_count
AFTER
    DELETE ON artist_bridge BEGIN
UPDATE
    artists
SET
    artist_song_count = artist_song_count - 1
WHERE
    artist_id = OLD.artist;

DELETE FROM
    artists
WHERE
    artist_song_count = 0;

END;

CREATE TRIGGER increment_album_count
AFTER
INSERT
    ON album_bridge BEGIN
UPDATE
    albums
SET
    album_song_count = album_song_count + 1
WHERE
    album_id = NEW.album;

END;

CREATE TRIGGER decrement_album_count
AFTER
    DELETE ON album_bridge BEGIN
UPDATE
    albums
SET
    album_song_count = album_song_count - 1
WHERE
    album_id = OLD.album;

DELETE FROM
    albums
WHERE
    album_song_count = 0;

END;

CREATE TRIGGER increment_genre_count
AFTER
INSERT
    ON genre_bridge BEGIN
UPDATE
    genres
SET
    genre_song_count = genre_song_count + 1
WHERE
    genre_id = NEW.genre;

END;

CREATE TRIGGER decrement_genre_count
AFTER
    DELETE ON genre_bridge BEGIN
UPDATE
    genres
SET
    genre_song_count = genre_song_count - 1
WHERE
    genre_id = OLD.genre;

DELETE FROM
    genres
WHERE
    genre_song_count = 0;

END;

CREATE TRIGGER increment_playlist_count
AFTER
INSERT
    ON playlist_bridge BEGIN
UPDATE
    playlists
SET
    playlist_song_count = playlist_song_count + 1
WHERE
    playlist_id = NEW.playlist;

END;

CREATE TRIGGER decrement_playlist_count
AFTER
    DELETE ON playlist_bridge BEGIN
UPDATE
    playlists
SET
    playlist_song_count = playlist_song_count - 1
WHERE
    playlist_id = OLD.playlist;

END;