-- Your SQL goes here
CREATE TABLE `artist_bridge`(
	`id` INTEGER PRIMARY KEY,
	`song` TEXT,
	`artist` TEXT,
	FOREIGN KEY (`song`) REFERENCES `allsongs`(`_id`),
	FOREIGN KEY (`artist`) REFERENCES `artists`(`artist_id`)
);

CREATE TABLE `playlist_bridge`(
	`id` INTEGER PRIMARY KEY,
	`song` TEXT,
	`playlist` TEXT,
	FOREIGN KEY (`song`) REFERENCES `allsongs`(`_id`),
	FOREIGN KEY (`playlist`) REFERENCES `playlists`(`playlist_id`)
);

CREATE TABLE `artists`(
	`artist_id` TEXT PRIMARY KEY,
	`artist_mbid` TEXT,
	`artist_name` TEXT,
	`artist_coverpath` TEXT,
	`artist_song_count` DOUBLE NOT NULL,
	`artist_extra_info` TEXT,
	`sanitized_artist_name` TEXT
);

CREATE TABLE `album_bridge`(
	`id` INTEGER PRIMARY KEY,
	`song` TEXT,
	`album` TEXT,
	FOREIGN KEY (`song`) REFERENCES `allsongs`(`_id`),
	FOREIGN KEY (`album`) REFERENCES `albums`(`album_id`)
);

CREATE TABLE `genres`(
	`genre_id` TEXT PRIMARY KEY,
	`genre_name` TEXT,
	`genre_song_count` DOUBLE NOT NULL
);

CREATE TABLE `playlists`(
	`playlist_id` TEXT PRIMARY KEY,
	`playlist_name` TEXT NOT NULL,
	`playlist_coverpath` TEXT,
	`playlist_song_count` DOUBLE NOT NULL,
	`playlist_desc` TEXT,
	`playlist_path` TEXT,
	`extension` TEXT,
	`icon` TEXT
);

CREATE TABLE `allsongs`(
	`_id` TEXT PRIMARY KEY,
	`path` TEXT,
	`size` DOUBLE,
	`inode` TEXT,
	`deviceno` TEXT,
	`title` TEXT,
	`date` TEXT,
	`year` TEXT,
	`lyrics` TEXT,
	`releasetype` TEXT,
	`bitrate` DOUBLE,
	`codec` TEXT,
	`container` TEXT,
	`duration` DOUBLE,
	`samplerate` DOUBLE,
	`hash` TEXT,
	`type` TEXT NOT NULL,
	`url` TEXT,
	`song_coverpath_high` TEXT,
	`playbackurl` TEXT,
	`song_coverpath_low` TEXT,
	`date_added` UNSIGNED BIG INT,
	`provider_extension` TEXT,
	`icon` TEXT,
	`show_in_library` BOOL,
	`track_no` DOUBLE
);

CREATE TABLE `genre_bridge`(
	`id` INTEGER PRIMARY KEY,
	`song` TEXT,
	`genre` TEXT,
	FOREIGN KEY (`song`) REFERENCES `allsongs`(`_id`),
	FOREIGN KEY (`genre`) REFERENCES `genres`(`genre_id`)
);

CREATE TABLE `albums`(
	`album_id` TEXT PRIMARY KEY,
	`album_name` TEXT,
	`album_artist` TEXT,
	`album_coverpath_high` TEXT,
	`album_song_count` DOUBLE NOT NULL,
	`year` TEXT,
	`album_coverpath_low` TEXT,
	`album_extra_info` TEXT
);

CREATE TABLE `analytics`(
	`id` TEXT PRIMARY KEY,
	`song_id` TEXT,
	`play_count` INTEGER,
	`play_time` DOUBLE
);