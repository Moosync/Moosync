-- Your SQL goes here
CREATE TABLE `cache`(
    `id` INTEGER PRIMARY KEY AUTOINCREMENT,
    `url` TEXT NOT NULL,
    `blob` BLOB NOT NULL,
    `expires` INTEGER NOT NULL
);

CREATE UNIQUE INDEX `cache_url_idx` on `cache`(`url`);