CREATE TABLE disk_cache (
    path TEXT PRIMARY KEY,
    category TEXT NOT NULL,
    owner TEXT NOT NULL,
    size INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    last_accessed INTEGER NOT NULL
) STRICT;

CREATE INDEX idx_disk_cache_category ON disk_cache(category);
CREATE INDEX idx_disk_cache_owner ON disk_cache(owner);
