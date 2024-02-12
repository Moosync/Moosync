-- Your SQL goes here
CREATE UNIQUE INDEX path_uq ON allsongs(path);

CREATE UNIQUE INDEX sanitized_artist_name_uq ON artists(sanitized_artist_name);

CREATE UNIQUE INDEX album_name_uq ON albums(album_name);

CREATE UNIQUE INDEX genre_name_uq ON genres(genre_name);

CREATE UNIQUE INDEX artist_bridge_uq ON artist_bridge(song, artist);

CREATE UNIQUE INDEX album_bridge_uq ON album_bridge(song, album);

CREATE UNIQUE INDEX genre_bridge_uq ON genre_bridge(song, genre);