CREATE TABLE IF NOT EXISTS species (
    id               TEXT    PRIMARY KEY,
    name             TEXT    NOT NULL,
    rarity_tier      TEXT    NOT NULL,
    ascii_idle       TEXT    NOT NULL,
    ascii_happy      TEXT    NOT NULL,
    ascii_hungry     TEXT    NOT NULL,
    ascii_special    TEXT,
    description      TEXT    NOT NULL,
    codex_flavor_text TEXT   NOT NULL
);

CREATE TABLE IF NOT EXISTS creatures (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    species_id  TEXT    NOT NULL DEFAULT '',
    nickname    TEXT,
    hatched_at  INTEGER NOT NULL,
    retired_at  INTEGER,
    is_active   INTEGER NOT NULL DEFAULT 0,
    age_ticks   INTEGER NOT NULL DEFAULT 0,
    status      TEXT    NOT NULL DEFAULT 'egg'
);

CREATE TABLE IF NOT EXISTS creature_stats (
    creature_id     INTEGER PRIMARY KEY REFERENCES creatures(id),
    hunger          INTEGER NOT NULL DEFAULT 100,
    happiness       INTEGER NOT NULL DEFAULT 100,
    energy          INTEGER NOT NULL DEFAULT 100,
    xp              INTEGER NOT NULL DEFAULT 0,
    milestone_count INTEGER NOT NULL DEFAULT 0,
    starving_ticks  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS codex (
    species_id       TEXT    PRIMARY KEY REFERENCES species(id),
    discovered       INTEGER NOT NULL DEFAULT 0,
    first_hatched_at INTEGER,
    times_hatched    INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS settings (
    id               INTEGER PRIMARY KEY CHECK (id = 1),
    always_on_top    INTEGER NOT NULL DEFAULT 1,
    window_x         INTEGER,
    window_y         INTEGER,
    tick_interval_secs INTEGER NOT NULL DEFAULT 60,
    show_tray_tooltip INTEGER NOT NULL DEFAULT 1
);

INSERT OR IGNORE INTO settings (id) VALUES (1);
