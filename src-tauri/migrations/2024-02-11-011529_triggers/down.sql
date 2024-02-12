-- This file should undo anything in `up.sql`
DROP TRIGGER IF EXISTS increment_artist_count;

DROP TRIGGER IF EXISTS decrement_artist_count;

DROP TRIGGER IF EXISTS increment_album_count;

DROP TRIGGER IF EXISTS decrement_album_count;

DROP TRIGGER IF EXISTS increment_genre_count;

DROP TRIGGER IF EXISTS decrement_genre_count;

DROP TRIGGER IF EXISTS increment_playlist_count;

DROP TRIGGER IF EXISTS decrement_playlist_count;