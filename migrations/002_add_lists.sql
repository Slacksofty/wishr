CREATE TABLE IF NOT EXISTS item_lists (
    id         INTEGER PRIMARY KEY AUTOINCREMENT,
    name       TEXT    NOT NULL,
    created_at TEXT    NOT NULL
);

-- Seed a default list for all existing items
INSERT OR IGNORE INTO item_lists (id, name, created_at)
    VALUES (1, 'General', datetime('now'));

-- Add list_id to wish_items; existing rows fall back to list 1
-- Note: SQLite disallows REFERENCES on ALTER TABLE ADD COLUMN with a non-NULL default
ALTER TABLE wish_items
    ADD COLUMN list_id INTEGER NOT NULL DEFAULT 1;
