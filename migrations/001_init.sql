CREATE TABLE IF NOT EXISTS wish_items (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    name            TEXT    NOT NULL,
    description     TEXT,
    estimated_cost  REAL,
    want_level      INTEGER NOT NULL DEFAULT 3,
    need_level      INTEGER NOT NULL DEFAULT 3,
    where_to_buy    TEXT,
    category        TEXT,
    notes           TEXT,
    status          TEXT    NOT NULL DEFAULT 'active',
    created_at      TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS purchase_records (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id       INTEGER NOT NULL REFERENCES wish_items(id),
    actual_cost   REAL,
    purchased_at  TEXT    NOT NULL,
    notes         TEXT
);
