CREATE TABLE IF NOT EXISTS water
(
    id integer primary key autoincrement,
    timestamp integer
);

CREATE INDEX IF NOT EXISTS idx_items_timestamp on water (timestamp);