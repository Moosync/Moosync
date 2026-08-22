-- This file should undo anything in `up.sql`
ALTER TABLE playlists
DROP COLUMN library_item;
