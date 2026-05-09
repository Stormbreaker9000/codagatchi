# Codagatchi Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a lightweight desktop tamagotchi app with ASCII creatures, a rarity/hatching system, and a Codex — implemented as a Tauri 2 floating widget with a Rust game engine and React frontend.

**Architecture:** Rust owns all game state via SQLite (rusqlite). A tokio background task ticks the game loop every 60s and emits Tauri events to React. React is a display-only layer that calls Tauri commands and listens for `game-tick` events. No state library needed on the frontend.

**Tech Stack:** Tauri 2, React 18 + Vite + TypeScript, Rust, rusqlite (bundled), tauri-plugin-notification, tokio (via Tauri runtime), rand 0.8

---

## File Map

```
src-tauri/
  Cargo.toml
  tauri.conf.json
  capabilities/default.json
  migrations/001_initial.sql
  src/
    main.rs          — windows_subsystem + calls lib::run()
    lib.rs           — Tauri builder, plugin registration, setup
    types.rs         — all shared Rust types (Species, Creature, CreatureStats, etc.)
    creatures.rs     — species definitions (ASCII art), game logic, rarity roll
    db.rs            — Connection init, migrations, all SQL query helpers
    commands.rs      — #[tauri::command] functions
    game_loop.rs     — background tick task, ActivitySnapshot stub
    tray.rs          — system tray construction and event handling

src/
  main.tsx
  App.tsx            — widget/expanded mode toggle, layout root
  types.ts           — TypeScript types mirroring Rust types
  hooks/
    useGameState.ts  — subscribes to game-tick event
    useCommands.ts   — typed invoke() wrappers
  components/
    Widget/
      CreatureDisplay.tsx   — ASCII renderer + frame animation
      StatBars.tsx          — hunger / happiness / energy bars
      ActionButtons.tsx     — feed, play, hatch (conditional)
    Expanded/
      Codex.tsx             — species grid, discovered vs undiscovered
      CreatureManager.tsx   — collection list, set active creature
      Settings.tsx          — always-on-top, tick interval, window position
  index.css

src-tauri/tests/
  creatures_test.rs  — unit tests for game logic
  db_test.rs         — unit tests for query helpers
```

---

## Task 1: Scaffold Tauri 2 + React project

**Files:**
- Create: all scaffold files in `/home/storm/wsl-projects/codagatchi/`

- [ ] **Step 1: Scaffold the project**

From the **parent** directory (`/home/storm/wsl-projects`), scaffold into a temporary directory then merge:

```bash
cd /home/storm/wsl-projects
npm create tauri-app@latest codagatchi-scaffold -- --template react-ts --manager npm --identifier com.codagatchi.app
```

When prompted for project name, enter `codagatchi-scaffold`. Then merge into the existing repo:

```bash
cp -r codagatchi-scaffold/src codagatchi/
cp -r codagatchi-scaffold/src-tauri codagatchi/
cp codagatchi-scaffold/package.json codagatchi/
cp codagatchi-scaffold/tsconfig.json codagatchi/
cp codagatchi-scaffold/tsconfig.node.json codagatchi/
cp codagatchi-scaffold/vite.config.ts codagatchi/
cp codagatchi-scaffold/index.html codagatchi/
cp codagatchi-scaffold/.gitignore codagatchi/ 2>/dev/null || true
rm -rf codagatchi-scaffold
cd codagatchi
npm install
```

- [ ] **Step 2: Update `src-tauri/Cargo.toml` dependencies**

Replace the `[dependencies]` section in `src-tauri/Cargo.toml`:

```toml
[package]
name = "codagatchi"
version = "0.1.0"
edition = "2021"

[lib]
name = "codagatchi_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2", features = [] }

[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
tauri-plugin-notification = "2"
rusqlite = { version = "0.31", features = ["bundled"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rand = "0.8"
tokio = { version = "1", features = ["time"] }

[features]
default = []
bench = []
```

- [ ] **Step 3: Update `src-tauri/tauri.conf.json`**

Replace the file contents:

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Codagatchi",
  "version": "0.1.0",
  "identifier": "com.codagatchi.app",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "withGlobalTauri": false,
    "windows": [
      {
        "label": "main",
        "title": "Codagatchi",
        "width": 220,
        "height": 280,
        "resizable": false,
        "decorations": false,
        "alwaysOnTop": true,
        "transparent": false,
        "visible": true
      }
    ]
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

- [ ] **Step 4: Update capabilities**

Create/replace `src-tauri/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capabilities",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "notification:default"
  ]
}
```

- [ ] **Step 5: Verify the project builds**

```bash
npm run tauri dev
```

Expected: Tauri dev window opens (220×280, borderless). React scaffold renders. No compilation errors.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: scaffold Tauri 2 + React project"
```

---

## Task 2: Database migrations

**Files:**
- Create: `src-tauri/migrations/001_initial.sql`
- Create: `src-tauri/src/db.rs`

- [ ] **Step 1: Write the migration SQL**

Create `src-tauri/migrations/001_initial.sql`:

```sql
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
    species_id  TEXT    NOT NULL REFERENCES species(id),
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
```

- [ ] **Step 2: Write `db.rs`**

Create `src-tauri/src/db.rs`:

```rust
use rusqlite::{Connection, Result, params};
use std::path::Path;
use crate::types::*;

const MIGRATION_SQL: &str = include_str!("../migrations/001_initial.sql");

pub fn open(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
    run_migrations(&conn)?;
    Ok(conn)
}

pub fn open_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    run_migrations(&conn)?;
    Ok(conn)
}

fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(MIGRATION_SQL)
}

pub fn get_active_creature(conn: &Connection) -> Result<Option<(Creature, CreatureStats)>> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.species_id, c.nickname, c.hatched_at, c.retired_at,
                c.is_active, c.age_ticks, c.status,
                cs.hunger, cs.happiness, cs.energy, cs.xp, cs.milestone_count, cs.starving_ticks
         FROM creatures c
         JOIN creature_stats cs ON cs.creature_id = c.id
         WHERE c.is_active = 1 AND c.status = 'alive'
         LIMIT 1"
    )?;

    let result = stmt.query_row([], |row| {
        Ok((
            Creature {
                id: row.get(0)?,
                species_id: row.get(1)?,
                nickname: row.get(2)?,
                hatched_at: row.get(3)?,
                retired_at: row.get(4)?,
                is_active: row.get::<_, i32>(5)? != 0,
                age_ticks: row.get(6)?,
                status: CreatureStatus::from_str(&row.get::<_, String>(7)?),
            },
            CreatureStats {
                creature_id: row.get(0)?,
                hunger: row.get(8)?,
                happiness: row.get(9)?,
                energy: row.get(10)?,
                xp: row.get(11)?,
                milestone_count: row.get(12)?,
                starving_ticks: row.get(13)?,
            },
        ))
    });

    match result {
        Ok(v) => Ok(Some(v)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn get_species(conn: &Connection, id: &str) -> Result<Species> {
    conn.query_row(
        "SELECT id, name, rarity_tier, ascii_idle, ascii_happy, ascii_hungry,
                ascii_special, description, codex_flavor_text
         FROM species WHERE id = ?1",
        params![id],
        |row| Ok(Species {
            id: row.get(0)?,
            name: row.get(1)?,
            rarity_tier: RarityTier::from_str(&row.get::<_, String>(2)?),
            ascii_idle: row.get(3)?,
            ascii_happy: row.get(4)?,
            ascii_hungry: row.get(5)?,
            ascii_special: row.get(6)?,
            description: row.get(7)?,
            codex_flavor_text: row.get(8)?,
        }),
    )
}

pub fn get_all_species(conn: &Connection) -> Result<Vec<Species>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, rarity_tier, ascii_idle, ascii_happy, ascii_hungry,
                ascii_special, description, codex_flavor_text FROM species"
    )?;
    let rows = stmt.query_map([], |row| Ok(Species {
        id: row.get(0)?,
        name: row.get(1)?,
        rarity_tier: RarityTier::from_str(&row.get::<_, String>(2)?),
        ascii_idle: row.get(3)?,
        ascii_happy: row.get(4)?,
        ascii_hungry: row.get(5)?,
        ascii_special: row.get(6)?,
        description: row.get(7)?,
        codex_flavor_text: row.get(8)?,
    }))?;
    rows.collect()
}

pub fn update_stats(conn: &Connection, stats: &CreatureStats) -> Result<()> {
    conn.execute(
        "UPDATE creature_stats SET hunger=?1, happiness=?2, energy=?3,
         xp=?4, milestone_count=?5, starving_ticks=?6
         WHERE creature_id=?7",
        params![
            stats.hunger, stats.happiness, stats.energy,
            stats.xp, stats.milestone_count, stats.starving_ticks,
            stats.creature_id
        ],
    )?;
    Ok(())
}

pub fn mark_creature_dead(conn: &Connection, creature_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE creatures SET status='dead', is_active=0 WHERE id=?1",
        params![creature_id],
    )?;
    Ok(())
}

pub fn increment_age(conn: &Connection, creature_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE creatures SET age_ticks = age_ticks + 1 WHERE id=?1",
        params![creature_id],
    )?;
    Ok(())
}

pub fn count_eggs(conn: &Connection) -> Result<i32> {
    conn.query_row(
        "SELECT COUNT(*) FROM creatures WHERE status='egg'",
        [],
        |row| row.get(0),
    )
}

pub fn add_egg(conn: &Connection, now: i64) -> Result<()> {
    // Egg has no species until hatched — use placeholder species_id "egg"
    conn.execute(
        "INSERT INTO creatures (species_id, hatched_at, is_active, status)
         VALUES ('__egg__', ?1, 0, 'egg')",
        params![now],
    )?;
    Ok(())
}

pub fn get_next_egg_id(conn: &Connection) -> Result<Option<i64>> {
    let result = conn.query_row(
        "SELECT id FROM creatures WHERE status='egg' ORDER BY hatched_at ASC LIMIT 1",
        [],
        |row| row.get(0),
    );
    match result {
        Ok(id) => Ok(Some(id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn hatch_creature(
    conn: &Connection,
    egg_id: i64,
    species_id: &str,
    now: i64,
) -> Result<i64> {
    // Has active creature already?
    let has_active: bool = conn.query_row(
        "SELECT COUNT(*) > 0 FROM creatures WHERE is_active=1 AND status='alive'",
        [],
        |row| row.get(0),
    )?;

    conn.execute(
        "UPDATE creatures SET species_id=?1, hatched_at=?2, status='alive', is_active=?3
         WHERE id=?4",
        params![species_id, now, if has_active { 0 } else { 1 }, egg_id],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO creature_stats
         (creature_id, hunger, happiness, energy, xp, milestone_count, starving_ticks)
         VALUES (?1, 100, 100, 100, 0, 0, 0)",
        params![egg_id],
    )?;

    // Update codex
    conn.execute(
        "INSERT INTO codex (species_id, discovered, first_hatched_at, times_hatched)
         VALUES (?1, 1, ?2, 1)
         ON CONFLICT(species_id) DO UPDATE SET
           discovered=1,
           first_hatched_at=COALESCE(first_hatched_at, ?2),
           times_hatched=times_hatched+1",
        params![species_id, now],
    )?;

    Ok(egg_id)
}

pub fn get_codex(conn: &Connection) -> Result<Vec<CodexEntry>> {
    let mut stmt = conn.prepare(
        "SELECT s.id, s.name, s.rarity_tier, s.ascii_idle, s.ascii_happy,
                s.ascii_hungry, s.ascii_special, s.description, s.codex_flavor_text,
                COALESCE(co.discovered, 0),
                co.first_hatched_at,
                COALESCE(co.times_hatched, 0)
         FROM species s
         LEFT JOIN codex co ON co.species_id = s.id
         ORDER BY s.rarity_tier, s.name"
    )?;
    let rows = stmt.query_map([], |row| Ok(CodexEntry {
        species: Species {
            id: row.get(0)?,
            name: row.get(1)?,
            rarity_tier: RarityTier::from_str(&row.get::<_, String>(2)?),
            ascii_idle: row.get(3)?,
            ascii_happy: row.get(4)?,
            ascii_hungry: row.get(5)?,
            ascii_special: row.get(6)?,
            description: row.get(7)?,
            codex_flavor_text: row.get(8)?,
        },
        discovered: row.get::<_, i32>(9)? != 0,
        first_hatched_at: row.get(10)?,
        times_hatched: row.get(11)?,
    }))?;
    rows.collect()
}

pub fn get_settings(conn: &Connection) -> Result<Settings> {
    conn.query_row(
        "SELECT always_on_top, window_x, window_y, tick_interval_secs, show_tray_tooltip
         FROM settings WHERE id=1",
        [],
        |row| Ok(Settings {
            always_on_top: row.get::<_, i32>(0)? != 0,
            window_x: row.get(1)?,
            window_y: row.get(2)?,
            tick_interval_secs: row.get::<_, i64>(3)? as u64,
            show_tray_tooltip: row.get::<_, i32>(4)? != 0,
        }),
    )
}

pub fn update_settings(conn: &Connection, patch: &SettingsPatch) -> Result<Settings> {
    if let Some(v) = patch.always_on_top {
        conn.execute("UPDATE settings SET always_on_top=?1 WHERE id=1", params![v as i32])?;
    }
    if let Some(v) = patch.window_x {
        conn.execute("UPDATE settings SET window_x=?1 WHERE id=1", params![v])?;
    }
    if let Some(v) = patch.window_y {
        conn.execute("UPDATE settings SET window_y=?1 WHERE id=1", params![v])?;
    }
    if let Some(v) = patch.tick_interval_secs {
        conn.execute("UPDATE settings SET tick_interval_secs=?1 WHERE id=1", params![v as i64])?;
    }
    if let Some(v) = patch.show_tray_tooltip {
        conn.execute("UPDATE settings SET show_tray_tooltip=?1 WHERE id=1", params![v as i32])?;
    }
    get_settings(conn)
}

pub fn get_collection(conn: &Connection) -> Result<Vec<(Creature, Species)>> {
    let mut stmt = conn.prepare(
        "SELECT c.id, c.species_id, c.nickname, c.hatched_at, c.retired_at,
                c.is_active, c.age_ticks, c.status,
                s.id, s.name, s.rarity_tier, s.ascii_idle, s.ascii_happy,
                s.ascii_hungry, s.ascii_special, s.description, s.codex_flavor_text
         FROM creatures c
         JOIN species s ON s.id = c.species_id
         WHERE c.status IN ('alive', 'retired')
         ORDER BY c.hatched_at DESC"
    )?;
    let rows = stmt.query_map([], |row| Ok((
        Creature {
            id: row.get(0)?,
            species_id: row.get(1)?,
            nickname: row.get(2)?,
            hatched_at: row.get(3)?,
            retired_at: row.get(4)?,
            is_active: row.get::<_, i32>(5)? != 0,
            age_ticks: row.get(6)?,
            status: CreatureStatus::from_str(&row.get::<_, String>(7)?),
        },
        Species {
            id: row.get(8)?,
            name: row.get(9)?,
            rarity_tier: RarityTier::from_str(&row.get::<_, String>(10)?),
            ascii_idle: row.get(11)?,
            ascii_happy: row.get(12)?,
            ascii_hungry: row.get(13)?,
            ascii_special: row.get(14)?,
            description: row.get(15)?,
            codex_flavor_text: row.get(16)?,
        },
    )))?;
    rows.collect()
}

pub fn set_active_creature(conn: &Connection, creature_id: i64) -> Result<()> {
    conn.execute("UPDATE creatures SET is_active=0 WHERE is_active=1", [])?;
    conn.execute(
        "UPDATE creatures SET is_active=1 WHERE id=?1 AND status='alive'",
        params![creature_id],
    )?;
    Ok(())
}

pub fn seed_species(conn: &Connection, all_species: &[Species]) -> Result<()> {
    for s in all_species {
        conn.execute(
            "INSERT OR IGNORE INTO species
             (id, name, rarity_tier, ascii_idle, ascii_happy, ascii_hungry,
              ascii_special, description, codex_flavor_text)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![
                s.id, s.name, s.rarity_tier.as_str(),
                s.ascii_idle, s.ascii_happy, s.ascii_hungry,
                s.ascii_special, s.description, s.codex_flavor_text
            ],
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO codex (species_id) VALUES (?1)",
            params![s.id],
        )?;
    }
    Ok(())
}
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/migrations/ src-tauri/src/db.rs
git commit -m "feat: database migrations and query helpers"
```

---

## Task 3: Core types + species definitions

**Files:**
- Create: `src-tauri/src/types.rs`
- Create: `src-tauri/src/creatures.rs`

- [ ] **Step 1: Write `types.rs`**

Create `src-tauri/src/types.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RarityTier {
    Common,
    Uncommon,
    Rare,
    Legendary,
}

impl RarityTier {
    pub fn from_str(s: &str) -> Self {
        match s {
            "uncommon" => Self::Uncommon,
            "rare" => Self::Rare,
            "legendary" => Self::Legendary,
            _ => Self::Common,
        }
    }
    pub fn as_str(&self) -> &str {
        match self {
            Self::Common => "common",
            Self::Uncommon => "uncommon",
            Self::Rare => "rare",
            Self::Legendary => "legendary",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CreatureStatus {
    Egg,
    Alive,
    Retired,
    Dead,
}

impl CreatureStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "egg" => Self::Egg,
            "retired" => Self::Retired,
            "dead" => Self::Dead,
            _ => Self::Alive,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Species {
    pub id: String,
    pub name: String,
    pub rarity_tier: RarityTier,
    pub ascii_idle: String,
    pub ascii_happy: String,
    pub ascii_hungry: String,
    pub ascii_special: Option<String>,
    pub description: String,
    pub codex_flavor_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Creature {
    pub id: i64,
    pub species_id: String,
    pub nickname: Option<String>,
    pub hatched_at: i64,
    pub retired_at: Option<i64>,
    pub is_active: bool,
    pub age_ticks: i64,
    pub status: CreatureStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureStats {
    pub creature_id: i64,
    pub hunger: i32,
    pub happiness: i32,
    pub energy: i32,
    pub xp: i64,
    pub milestone_count: i32,
    pub starving_ticks: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureState {
    pub creature: Creature,
    pub species: Species,
    pub stats: CreatureStats,
    pub egg_count: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexEntry {
    pub species: Species,
    pub discovered: bool,
    pub first_hatched_at: Option<i64>,
    pub times_hatched: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub always_on_top: bool,
    pub window_x: Option<i32>,
    pub window_y: Option<i32>,
    pub tick_interval_secs: u64,
    pub show_tray_tooltip: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SettingsPatch {
    pub always_on_top: Option<bool>,
    pub window_x: Option<Option<i32>>,
    pub window_y: Option<Option<i32>>,
    pub tick_interval_secs: Option<u64>,
    pub show_tray_tooltip: Option<bool>,
}

/// Stub for future APM integration. v1 always passes None.
#[derive(Debug, Clone)]
pub struct ActivitySnapshot {
    pub apm: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickEvent {
    pub creature_state: Option<CreatureState>,
    pub egg_count: i32,
}
```

- [ ] **Step 2: Write `creatures.rs` — species data**

Create `src-tauri/src/creatures.rs`:

```rust
use crate::types::*;
use rand::Rng;

// ── Stat constants ──────────────────────────────────────────────────────────
pub const HUNGER_DECAY: i32 = 2;
pub const HAPPINESS_DECAY: i32 = 1;
pub const ENERGY_REGEN: i32 = 1;

pub const FEED_HUNGER: i32 = 20;
pub const FEED_ENERGY: i32 = -5;
pub const FEED_XP: i64 = 2;

pub const PLAY_HAPPINESS: i32 = 15;
pub const PLAY_ENERGY: i32 = -10;
pub const PLAY_XP: i64 = 3;

pub const STARVATION_WARNING: i32 = 20;
pub const STARVATION_DEATH_TICKS: i32 = 3;
pub const MAX_EGGS: i32 = 3;
pub const TICK_XP: i64 = 1;

// XP thresholds: index 0 = first milestone, index 1 = second, etc.
// After index 2, every 500 XP beyond 500 is a milestone.
pub fn next_milestone_xp(milestone_count: i32) -> i64 {
    match milestone_count {
        0 => 50,
        1 => 200,
        n => 500 * (n as i64 - 1),
    }
}

// ── Rarity weights ──────────────────────────────────────────────────────────
pub fn roll_rarity_tier(rng: &mut impl Rng) -> RarityTier {
    let roll: u32 = rng.gen_range(0..100);
    match roll {
        0..=59 => RarityTier::Common,
        60..=84 => RarityTier::Uncommon,
        85..=96 => RarityTier::Rare,
        _ => RarityTier::Legendary,
    }
}

// ── Game logic ──────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub enum LifecycleEvent {
    StarvationWarning,
    Died,
    MilestoneUnlocked,
}

/// Returns updated stats and any lifecycle events that fired.
pub fn apply_tick(
    mut stats: CreatureStats,
    _activity: Option<&ActivitySnapshot>,
) -> (CreatureStats, Vec<LifecycleEvent>) {
    let mut events = Vec::new();

    // Decay / regen
    stats.hunger = (stats.hunger - HUNGER_DECAY).max(0);
    stats.happiness = (stats.happiness - HAPPINESS_DECAY).max(0);
    stats.energy = (stats.energy + ENERGY_REGEN).min(100);
    stats.xp += TICK_XP;

    // Starving ticks
    if stats.hunger == 0 {
        stats.starving_ticks += 1;
    } else {
        stats.starving_ticks = 0;
    }

    // Lifecycle checks
    if stats.starving_ticks >= STARVATION_DEATH_TICKS {
        events.push(LifecycleEvent::Died);
    } else if stats.hunger <= STARVATION_WARNING {
        events.push(LifecycleEvent::StarvationWarning);
    }

    // Milestone check
    let threshold = next_milestone_xp(stats.milestone_count);
    if stats.xp >= threshold {
        events.push(LifecycleEvent::MilestoneUnlocked);
        stats.milestone_count += 1;
    }

    (stats, events)
}

pub fn apply_feed(mut stats: CreatureStats) -> CreatureStats {
    stats.hunger = (stats.hunger + FEED_HUNGER).min(100);
    stats.energy = (stats.energy + FEED_ENERGY).max(0);
    stats.xp += FEED_XP;
    stats
}

pub fn apply_play(mut stats: CreatureStats) -> CreatureStats {
    stats.happiness = (stats.happiness + PLAY_HAPPINESS).min(100);
    stats.energy = (stats.energy + PLAY_ENERGY).max(0);
    stats.xp += PLAY_XP;
    stats
}

// ── Species definitions ─────────────────────────────────────────────────────

pub fn all_species() -> Vec<Species> {
    vec![
        // ── Common ──
        Species {
            id: "blob".into(),
            name: "Blob".into(),
            rarity_tier: RarityTier::Common,
            ascii_idle:   "  ___  \n /o o\\ \n| --- |\n \\___/ ".into(),
            ascii_happy:  "  ___  \n /^ ^\\ \n| ~~~ |\n \\___/ ".into(),
            ascii_hungry: "  ___  \n /- -\\ \n| ... |\n \\___/ ".into(),
            ascii_special: None,
            description: "A wobbly, amorphous companion.".into(),
            codex_flavor_text: "No one knows what Blob is made of, and Blob doesn't care.".into(),
        },
        Species {
            id: "kitten".into(),
            name: "Kitten".into(),
            rarity_tier: RarityTier::Common,
            ascii_idle:   "/\\ /\\\n(o . o)\n > w <\n(_____)\n      ".into(),
            ascii_happy:  "/\\ /\\\n(^ . ^)\n > U <\n(_____)\n      ".into(),
            ascii_hungry: "/\\ /\\\n(; . ;)\n > _ <\n(_____)\n      ".into(),
            ascii_special: None,
            description: "A tiny ASCII cat with very strong opinions.".into(),
            codex_flavor_text: "Reportedly domesticated. Evidence is inconclusive.".into(),
        },
        Species {
            id: "sparrow".into(),
            name: "Sparrow".into(),
            rarity_tier: RarityTier::Common,
            ascii_idle:   "  __  \n (oo) \n-/||\\ \n  \\/  ".into(),
            ascii_happy:  "  __  \n (^^) \n-/||\\ \n  \\/  ".into(),
            ascii_hungry: "  __  \n (..) \n-/||\\ \n  \\/  ".into(),
            ascii_special: None,
            description: "Chirpy and round.".into(),
            codex_flavor_text: "Eats approximately its own weight in seeds daily.".into(),
        },
        Species {
            id: "bearcub".into(),
            name: "Bearcub".into(),
            rarity_tier: RarityTier::Common,
            ascii_idle:   "  _   _\n (eYe)\n /|_|\\ \n  \\_/  ".into(),
            ascii_happy:  "  _   _\n (^Y^)\n /|_|\\ \n  \\_/  ".into(),
            ascii_hungry: "  _   _\n (;Y;)\n /|_|\\ \n  \\_/  ".into(),
            ascii_special: None,
            description: "Perpetually sleepy.".into(),
            codex_flavor_text: "Hibernates 14 hours a day. Judgemental about the other 10.".into(),
        },
        // ── Uncommon ──
        Species {
            id: "drakling".into(),
            name: "Drakling".into(),
            rarity_tier: RarityTier::Uncommon,
            ascii_idle:   "^ ^  \n(o.o)\n)|||(\n  V  ".into(),
            ascii_happy:  "^ ^  \n(^.^)\n)|||(\n  V  ".into(),
            ascii_hungry: "^ ^  \n(-.-)\n)|||(\n  V  ".into(),
            ascii_special: None,
            description: "A small dragon, mostly harmless.".into(),
            codex_flavor_text: "Can technically breathe fire. Mostly breathes heavy sighs.".into(),
        },
        Species {
            id: "specter".into(),
            name: "Specter".into(),
            rarity_tier: RarityTier::Uncommon,
            ascii_idle:   ".~~~.\n/o   o\\\n| ^ |\n \\   /\n~~\\/~~".into(),
            ascii_happy:  ".~~~.\n/^ ^ ^\\\n| w |\n \\   /\n~~\\/~~".into(),
            ascii_hungry: ".~~~.\n/. . .\\\n| _ |\n \\   /\n~~\\/~~".into(),
            ascii_special: None,
            description: "Floats. Occasionally vanishes mid-conversation.".into(),
            codex_flavor_text: "Technically not alive. Doesn't let that slow it down.".into(),
        },
        // ── Rare ──
        Species {
            id: "phoenix".into(),
            name: "Phoenix".into(),
            rarity_tier: RarityTier::Rare,
            ascii_idle:   " /\\ \n/oo\\\n|~~|\n\\^^/\n >>> ".into(),
            ascii_happy:  " /\\ \n/^^\\\n|~~|\n\\^^/\n >>>".into(),
            ascii_hungry: " /\\ \n/..\\\n|--|\n\\--/\n ... ".into(),
            ascii_special: None,
            description: "A flame bird. Self-renewing. A bit smug about it.".into(),
            codex_flavor_text: "Has died 0 times. (This figure is contested.)".into(),
        },
        // ── Legendary ──
        Species {
            id: "voidling".into(),
            name: "Voidling".into(),
            rarity_tier: RarityTier::Legendary,
            ascii_idle:   "*  *  *\n* \\|/ *\n (o_o)\n* /|\\ *\n*  *  *".into(),
            ascii_happy:  "*  *  *\n* \\|/ *\n (^_^)\n* /|\\ *\n*  *  *".into(),
            ascii_hungry: "*  *  *\n* \\|/ *\n (O_O)\n* /|\\ *\n*  *  *".into(),
            ascii_special: Some("+ . + .\n. \\|/ .\n(>^_^<)\n. /|\\ .\n+ . + .".into()),
            description: "A being of pure cosmic energy. Handle with care.".into(),
            codex_flavor_text: "Exists in all states simultaneously. Prefers not to discuss it.".into(),
        },
    ]
}

pub fn species_by_tier(all: &[Species], tier: &RarityTier) -> Vec<&Species> {
    all.iter().filter(|s| &s.rarity_tier == tier).collect()
}
```

- [ ] **Step 3: Add modules to `lib.rs`**

Edit `src-tauri/src/lib.rs` to declare the modules (full lib.rs comes in Task 10, just add module declarations for now so `cargo check` passes):

```rust
mod types;
mod creatures;
mod db;

pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: Verify it compiles**

```bash
cd src-tauri && cargo check
```

Expected: no errors.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/types.rs src-tauri/src/creatures.rs src-tauri/src/lib.rs
git commit -m "feat: core types and species definitions"
```

---

## Task 4: Game logic tests

**Files:**
- Create: `src-tauri/src/creatures.rs` (tests block, appended to existing file)

- [ ] **Step 1: Write failing tests for `apply_tick`**

Append to the bottom of `src-tauri/src/creatures.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn base_stats() -> CreatureStats {
        CreatureStats {
            creature_id: 1,
            hunger: 50,
            happiness: 50,
            energy: 50,
            xp: 0,
            milestone_count: 0,
            starving_ticks: 0,
        }
    }

    #[test]
    fn apply_tick_decays_stats() {
        let stats = base_stats();
        let (result, events) = apply_tick(stats, None);
        assert_eq!(result.hunger, 48);
        assert_eq!(result.happiness, 49);
        assert_eq!(result.energy, 51);
        assert_eq!(result.xp, 1);
        assert!(events.is_empty());
    }

    #[test]
    fn apply_tick_clamps_at_zero() {
        let mut stats = base_stats();
        stats.hunger = 1;
        stats.happiness = 0;
        stats.energy = 99;
        let (result, _) = apply_tick(stats, None);
        assert_eq!(result.hunger, 0);
        assert_eq!(result.happiness, 0);
        assert_eq!(result.energy, 100);
    }

    #[test]
    fn apply_tick_fires_starvation_warning_at_threshold() {
        let mut stats = base_stats();
        stats.hunger = 22; // will decay to 20
        let (_, events) = apply_tick(stats, None);
        assert!(events.contains(&LifecycleEvent::StarvationWarning));
    }

    #[test]
    fn apply_tick_increments_starving_ticks() {
        let mut stats = base_stats();
        stats.hunger = 2; // decays to 0
        let (result, _) = apply_tick(stats, None);
        assert_eq!(result.starving_ticks, 1);
    }

    #[test]
    fn apply_tick_resets_starving_ticks_when_hungry_recovers() {
        let mut stats = base_stats();
        stats.hunger = 0;
        stats.starving_ticks = 1;
        // hunger decays to 0 again
        let (result, _) = apply_tick(stats.clone(), None);
        assert_eq!(result.starving_ticks, 2);

        // Now recover hunger
        let mut recovered = result;
        recovered.hunger = 50;
        recovered.starving_ticks = 2;
        let (final_stats, _) = apply_tick(recovered, None);
        assert_eq!(final_stats.starving_ticks, 0);
    }

    #[test]
    fn apply_tick_fires_died_after_three_starving_ticks() {
        let mut stats = base_stats();
        stats.hunger = 0;
        stats.starving_ticks = STARVATION_DEATH_TICKS - 1;
        let (_, events) = apply_tick(stats, None);
        assert!(events.contains(&LifecycleEvent::Died));
    }

    #[test]
    fn apply_tick_fires_milestone_at_50_xp() {
        let mut stats = base_stats();
        stats.xp = 49; // one tick puts it to 50
        let (result, events) = apply_tick(stats, None);
        assert_eq!(result.xp, 50);
        assert!(events.contains(&LifecycleEvent::MilestoneUnlocked));
        assert_eq!(result.milestone_count, 1);
    }

    #[test]
    fn next_milestone_xp_thresholds_are_correct() {
        assert_eq!(next_milestone_xp(0), 50);
        assert_eq!(next_milestone_xp(1), 200);
        assert_eq!(next_milestone_xp(2), 500);
        assert_eq!(next_milestone_xp(3), 1000);
        assert_eq!(next_milestone_xp(4), 1500);
    }

    #[test]
    fn apply_feed_clamps_at_100() {
        let mut stats = base_stats();
        stats.hunger = 90;
        let result = apply_feed(stats);
        assert_eq!(result.hunger, 100);
        assert_eq!(result.energy, 45);
        assert_eq!(result.xp, FEED_XP);
    }

    #[test]
    fn apply_play_clamps_energy_at_zero() {
        let mut stats = base_stats();
        stats.energy = 5;
        let result = apply_play(stats);
        assert_eq!(result.energy, 0);
        assert_eq!(result.happiness, 65);
    }

    #[test]
    fn roll_rarity_tier_roughly_correct_distribution() {
        use rand::SeedableRng;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(42);
        let mut common = 0u32;
        let mut uncommon = 0u32;
        let mut rare = 0u32;
        let mut legendary = 0u32;
        for _ in 0..10_000 {
            match roll_rarity_tier(&mut rng) {
                RarityTier::Common => common += 1,
                RarityTier::Uncommon => uncommon += 1,
                RarityTier::Rare => rare += 1,
                RarityTier::Legendary => legendary += 1,
            }
        }
        // Allow ±5% tolerance
        assert!(common > 5500 && common < 6500, "common={common}");
        assert!(uncommon > 2000 && uncommon < 3000, "uncommon={uncommon}");
        assert!(rare > 700 && rare < 1700, "rare={rare}");
        assert!(legendary > 0 && legendary < 800, "legendary={legendary}");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd src-tauri && cargo test
```

Expected: compilation error or test failures (the functions exist but tests are being run for the first time).

- [ ] **Step 3: Run tests to verify they pass**

After the creatures.rs implementation from Task 3 is in place, all tests should pass:

```bash
cd src-tauri && cargo test creatures
```

Expected:

```
test creatures::tests::apply_feed_clamps_at_100 ... ok
test creatures::tests::apply_play_clamps_energy_at_zero ... ok
test creatures::tests::apply_tick_clamps_at_zero ... ok
test creatures::tests::apply_tick_decays_stats ... ok
test creatures::tests::apply_tick_fires_died_after_three_starving_ticks ... ok
test creatures::tests::apply_tick_fires_milestone_at_50_xp ... ok
test creatures::tests::apply_tick_fires_starvation_warning_at_threshold ... ok
test creatures::tests::apply_tick_increments_starving_ticks ... ok
test creatures::tests::apply_tick_resets_starving_ticks_when_hungry_recovers ... ok
test creatures::tests::next_milestone_xp_thresholds_are_correct ... ok
test creatures::tests::roll_rarity_tier_roughly_correct_distribution ... ok
```

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/creatures.rs
git commit -m "test: game logic unit tests — all passing"
```

---

## Task 5: Tauri commands

**Files:**
- Create: `src-tauri/src/commands.rs`

- [ ] **Step 1: Write `commands.rs`**

Create `src-tauri/src/commands.rs`:

```rust
use tauri::State;
use std::sync::Mutex;
use rusqlite::Connection;
use crate::{types::*, creatures, db};

pub struct AppState {
    pub db: Mutex<Connection>,
}

#[tauri::command]
pub fn get_active_creature(state: State<'_, AppState>) -> Result<Option<CreatureState>, String> {
    let conn = state.db.lock().unwrap();
    let row = db::get_active_creature(&conn).map_err(|e| e.to_string())?;
    match row {
        None => Ok(None),
        Some((creature, stats)) => {
            let species = db::get_species(&conn, &creature.species_id)
                .map_err(|e| e.to_string())?;
            let egg_count = db::count_eggs(&conn).map_err(|e| e.to_string())?;
            Ok(Some(CreatureState { creature, species, stats, egg_count }))
        }
    }
}

#[tauri::command]
pub fn get_codex(state: State<'_, AppState>) -> Result<Vec<CodexEntry>, String> {
    let conn = state.db.lock().unwrap();
    db::get_codex(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let conn = state.db.lock().unwrap();
    db::get_settings(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn feed(state: State<'_, AppState>, creature_id: i64) -> Result<CreatureStats, String> {
    let conn = state.db.lock().unwrap();
    let row = db::get_active_creature(&conn).map_err(|e| e.to_string())?;
    match row {
        None => Err("No active creature".into()),
        Some((creature, stats)) => {
            if creature.id != creature_id {
                return Err("Creature ID mismatch".into());
            }
            let updated = creatures::apply_feed(stats);
            db::update_stats(&conn, &updated).map_err(|e| e.to_string())?;
            Ok(updated)
        }
    }
}

#[tauri::command]
pub fn play(state: State<'_, AppState>, creature_id: i64) -> Result<CreatureStats, String> {
    let conn = state.db.lock().unwrap();
    let row = db::get_active_creature(&conn).map_err(|e| e.to_string())?;
    match row {
        None => Err("No active creature".into()),
        Some((creature, stats)) => {
            if creature.id != creature_id {
                return Err("Creature ID mismatch".into());
            }
            let updated = creatures::apply_play(stats);
            db::update_stats(&conn, &updated).map_err(|e| e.to_string())?;
            Ok(updated)
        }
    }
}

#[tauri::command]
pub fn hatch_egg(state: State<'_, AppState>) -> Result<CreatureState, String> {
    let conn = state.db.lock().unwrap();

    let egg_id = db::get_next_egg_id(&conn)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "No eggs available".to_string())?;

    let all_species = db::get_all_species(&conn).map_err(|e| e.to_string())?;
    let codex = db::get_codex(&conn).map_err(|e| e.to_string())?;

    let mut rng = rand::thread_rng();
    let tier = creatures::roll_rarity_tier(&mut rng);

    // Prefer undiscovered species in the rolled tier
    let discovered_ids: std::collections::HashSet<&str> = codex.iter()
        .filter(|e| e.discovered)
        .map(|e| e.species.id.as_str())
        .collect();

    let candidates: Vec<&Species> = all_species.iter()
        .filter(|s| s.rarity_tier == tier)
        .collect();

    let undiscovered: Vec<&Species> = candidates.iter()
        .copied()
        .filter(|s| !discovered_ids.contains(s.id.as_str()))
        .collect();

    let pool = if undiscovered.is_empty() { &candidates } else { &undiscovered };
    let idx = rng.gen_range(0..pool.len());
    let chosen_species = pool[idx];

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let creature_id = db::hatch_creature(&conn, egg_id, &chosen_species.id, now)
        .map_err(|e| e.to_string())?;

    let (creature, stats) = db::get_active_creature(&conn)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Failed to load hatched creature".to_string())?;

    // If newly hatched creature is not the active one (player had one already),
    // load the newly hatched creature directly
    let (final_creature, final_stats) = if creature.id == creature_id {
        (creature, stats)
    } else {
        // Load the newly hatched creature's stats
        conn.query_row(
            "SELECT c.id, c.species_id, c.nickname, c.hatched_at, c.retired_at,
                    c.is_active, c.age_ticks, c.status,
                    cs.hunger, cs.happiness, cs.energy, cs.xp, cs.milestone_count, cs.starving_ticks
             FROM creatures c JOIN creature_stats cs ON cs.creature_id = c.id
             WHERE c.id = ?1",
            rusqlite::params![creature_id],
            |row| Ok((
                Creature {
                    id: row.get(0)?,
                    species_id: row.get(1)?,
                    nickname: row.get(2)?,
                    hatched_at: row.get(3)?,
                    retired_at: row.get(4)?,
                    is_active: row.get::<_, i32>(5)? != 0,
                    age_ticks: row.get(6)?,
                    status: CreatureStatus::from_str(&row.get::<_, String>(7)?),
                },
                CreatureStats {
                    creature_id: row.get(0)?,
                    hunger: row.get(8)?,
                    happiness: row.get(9)?,
                    energy: row.get(10)?,
                    xp: row.get(11)?,
                    milestone_count: row.get(12)?,
                    starving_ticks: row.get(13)?,
                },
            )),
        ).map_err(|e| e.to_string())?
    };

    let egg_count = db::count_eggs(&conn).map_err(|e| e.to_string())?;
    Ok(CreatureState {
        creature: final_creature,
        species: chosen_species.clone(),
        stats: final_stats,
        egg_count,
    })
}

#[tauri::command]
pub fn set_active_creature(state: State<'_, AppState>, creature_id: i64) -> Result<(), String> {
    let conn = state.db.lock().unwrap();
    db::set_active_creature(&conn, creature_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_settings(
    state: State<'_, AppState>,
    patch: SettingsPatch,
    window: tauri::WebviewWindow,
) -> Result<Settings, String> {
    let conn = state.db.lock().unwrap();
    let settings = db::update_settings(&conn, &patch).map_err(|e| e.to_string())?;

    // Apply always_on_top immediately
    if patch.always_on_top.is_some() {
        window.set_always_on_top(settings.always_on_top)
            .map_err(|e| e.to_string())?;
    }

    Ok(settings)
}

#[tauri::command]
pub fn get_collection(state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, String> {
    let conn = state.db.lock().unwrap();
    let rows = db::get_collection(&conn).map_err(|e| e.to_string())?;
    let result: Vec<serde_json::Value> = rows.into_iter().map(|(creature, species)| {
        serde_json::json!({ "creature": creature, "species": species })
    }).collect();
    Ok(result)
}
```

- [ ] **Step 2: Add `use rand::Rng` import to commands.rs**

Add at the top of `commands.rs` (already included above, verify it's present):

```rust
use rand::Rng;
```

- [ ] **Step 3: Verify compilation**

```bash
cd src-tauri && cargo check
```

Expected: no errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands.rs
git commit -m "feat: tauri commands — get_active_creature, feed, play, hatch_egg, etc."
```

---

## Task 6: Game loop

**Files:**
- Create: `src-tauri/src/game_loop.rs`

- [ ] **Step 1: Write `game_loop.rs`**

Create `src-tauri/src/game_loop.rs`:

```rust
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use rusqlite::Connection;
use crate::{creatures, db, types::*};

#[cfg(feature = "bench")]
use std::time::Instant;

pub fn start(app: AppHandle, db: Arc<Mutex<Connection>>) {
    tauri::async_runtime::spawn(async move {
        loop {
            // Read tick interval from settings each loop (allows live changes)
            let interval_secs = {
                let conn = db.lock().unwrap();
                db::get_settings(&conn)
                    .map(|s| s.tick_interval_secs)
                    .unwrap_or(60)
            };

            tokio::time::sleep(Duration::from_secs(interval_secs)).await;

            #[cfg(feature = "bench")]
            let tick_start = Instant::now();

            tick(&app, &db);

            #[cfg(feature = "bench")]
            {
                let elapsed = tick_start.elapsed();
                eprintln!("[bench] tick took {:?}", elapsed);
            }
        }
    });
}

fn tick(app: &AppHandle, db: &Arc<Mutex<Connection>>) {
    let conn = db.lock().unwrap();

    let row = match db::get_active_creature(&conn) {
        Ok(Some(r)) => r,
        _ => {
            // No active creature — emit egg count anyway
            let egg_count = db::count_eggs(&conn).unwrap_or(0);
            let _ = app.emit("game-tick", TickEvent { creature_state: None, egg_count });
            return;
        }
    };

    let (creature, stats) = row;
    let (updated_stats, events) = creatures::apply_tick(stats, None /* v1: no APM */);

    // Handle lifecycle events
    let mut died = false;
    let mut milestone_unlocked = false;

    for event in &events {
        match event {
            creatures::LifecycleEvent::Died => died = true,
            creatures::LifecycleEvent::MilestoneUnlocked => milestone_unlocked = true,
            creatures::LifecycleEvent::StarvationWarning => {
                send_notification(app, "Hungry!", &format!("{} is starving!", creature_name(&creature)));
            }
        }
    }

    if died {
        let _ = db::mark_creature_dead(&conn, creature.id);
        let egg_count = db::count_eggs(&conn).unwrap_or(0);
        let action_label = if egg_count > 0 { "Hatch a new egg?" } else { "" };
        send_notification(app, "Oh no!", &format!(
            "{} has died.{}",
            creature_name(&creature),
            if action_label.is_empty() { String::new() } else { format!(" {action_label}") }
        ));
        let _ = app.emit("game-tick", TickEvent { creature_state: None, egg_count });
        return;
    }

    let _ = db::update_stats(&conn, &updated_stats);
    let _ = db::increment_age(&conn, creature.id);

    if milestone_unlocked {
        let current_egg_count = db::count_eggs(&conn).unwrap_or(0);
        if current_egg_count < creatures::MAX_EGGS {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            let _ = db::add_egg(&conn, now);
            send_notification(app, "New egg!", "🥚 A new egg is ready to hatch!");
        }
    }

    let species = match db::get_species(&conn, &creature.species_id) {
        Ok(s) => s,
        Err(_) => return,
    };
    let egg_count = db::count_eggs(&conn).unwrap_or(0);

    let state = CreatureState {
        creature,
        species,
        stats: updated_stats,
        egg_count,
    };

    let _ = app.emit("game-tick", TickEvent { creature_state: Some(state), egg_count });
}

fn creature_name(creature: &Creature) -> String {
    creature.nickname.clone().unwrap_or_else(|| "Your creature".into())
}

fn send_notification(app: &AppHandle, title: &str, body: &str) {
    use tauri_plugin_notification::NotificationExt;
    let _ = app.notification()
        .builder()
        .title(title)
        .body(body)
        .show();
}
```

- [ ] **Step 2: Verify compilation**

```bash
cd src-tauri && cargo check
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/game_loop.rs
git commit -m "feat: tokio background game loop with lifecycle event handling"
```

---

## Task 7: System tray

**Files:**
- Create: `src-tauri/src/tray.rs`

- [ ] **Step 1: Write `tray.rs`**

Create `src-tauri/src/tray.rs`:

```rust
use tauri::{
    App, Manager,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

pub fn build(app: &mut App) -> tauri::Result<()> {
    let show_hide = MenuItem::with_id(app, "show_hide", "Show / Hide", true, None::<&str>)?;
    let separator = tauri::menu::PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Codagatchi", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_hide, &separator, &quit])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .menu_on_left_click(false)
        .tooltip("Codagatchi")
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "show_hide" => {
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
                "quit" => app.exit(0),
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}
```

- [ ] **Step 2: Verify compilation**

```bash
cd src-tauri && cargo check
```

Expected: no errors.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/tray.rs
git commit -m "feat: system tray with show/hide and quit"
```

---

## Task 8: Wire up lib.rs

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Write `main.rs`**

Replace `src-tauri/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    codagatchi_lib::run();
}
```

- [ ] **Step 2: Write the full `lib.rs`**

Replace `src-tauri/src/lib.rs`:

```rust
mod commands;
mod creatures;
mod db;
mod game_loop;
mod tray;
mod types;

use commands::AppState;
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            // Resolve database path in app data directory
            let app_data = app.path().app_data_dir()
                .expect("could not resolve app data dir");
            std::fs::create_dir_all(&app_data)?;
            let db_path = app_data.join("codagatchi.db");

            let conn = db::open(&db_path)
                .expect("failed to open database");

            // Seed species on first run
            let all_species = creatures::all_species();
            db::seed_species(&conn, &all_species)
                .expect("failed to seed species");

            let db_arc = Arc::new(Mutex::new(conn));

            // Register shared state
            app.manage(AppState { db: Mutex::new(
                // We open a second connection for commands (separate from game loop)
                db::open(&db_path).expect("failed to open db for commands")
            )});

            // Start game loop with its own connection
            let loop_conn = db::open(&db_path)
                .expect("failed to open db for game loop");
            let loop_db = Arc::new(Mutex::new(loop_conn));
            game_loop::start(app.handle().clone(), loop_db);

            // Build tray
            tray::build(app)?;

            // Request notification permission
            use tauri_plugin_notification::NotificationExt;
            let _ = app.notification().request_permission();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_active_creature,
            commands::get_codex,
            commands::get_settings,
            commands::feed,
            commands::play,
            commands::hatch_egg,
            commands::set_active_creature,
            commands::update_settings,
            commands::get_collection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Build and run**

```bash
npm run tauri dev
```

Expected: app launches, tray icon appears, no console errors. The window is 220×280 borderless.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/main.rs src-tauri/src/lib.rs
git commit -m "feat: wire up lib.rs — plugins, state, game loop, tray"
```

---

## Task 9: TypeScript types and hooks

**Files:**
- Create: `src/types.ts`
- Create: `src/hooks/useGameState.ts`
- Create: `src/hooks/useCommands.ts`

- [ ] **Step 1: Write `src/types.ts`**

```typescript
export type RarityTier = 'common' | 'uncommon' | 'rare' | 'legendary';
export type CreatureStatus = 'egg' | 'alive' | 'retired' | 'dead';

export interface Species {
  id: string;
  name: string;
  rarity_tier: RarityTier;
  ascii_idle: string;
  ascii_happy: string;
  ascii_hungry: string;
  ascii_special: string | null;
  description: string;
  codex_flavor_text: string;
}

export interface Creature {
  id: number;
  species_id: string;
  nickname: string | null;
  hatched_at: number;
  retired_at: number | null;
  is_active: boolean;
  age_ticks: number;
  status: CreatureStatus;
}

export interface CreatureStats {
  creature_id: number;
  hunger: number;
  happiness: number;
  energy: number;
  xp: number;
  milestone_count: number;
  starving_ticks: number;
}

export interface CreatureState {
  creature: Creature;
  species: Species;
  stats: CreatureStats;
  egg_count: number;
}

export interface CodexEntry {
  species: Species;
  discovered: boolean;
  first_hatched_at: number | null;
  times_hatched: number;
}

export interface Settings {
  always_on_top: boolean;
  window_x: number | null;
  window_y: number | null;
  tick_interval_secs: number;
  show_tray_tooltip: boolean;
}

export interface SettingsPatch {
  always_on_top?: boolean;
  window_x?: number | null;
  window_y?: number | null;
  tick_interval_secs?: number;
  show_tray_tooltip?: boolean;
}

export interface TickEvent {
  creature_state: CreatureState | null;
  egg_count: number;
}

export interface CollectionEntry {
  creature: Creature;
  species: Species;
}
```

- [ ] **Step 2: Write `src/hooks/useGameState.ts`**

```typescript
import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { CreatureState, TickEvent } from '../types';

export function useGameState() {
  const [creatureState, setCreatureState] = useState<CreatureState | null>(null);
  const [eggCount, setEggCount] = useState(0);
  const [loading, setLoading] = useState(true);

  // Load initial state
  useEffect(() => {
    invoke<CreatureState | null>('get_active_creature').then((state) => {
      setCreatureState(state);
      if (state) setEggCount(state.egg_count);
      setLoading(false);
    });
  }, []);

  // Subscribe to game tick events
  useEffect(() => {
    const unlisten = listen<TickEvent>('game-tick', (event) => {
      setCreatureState(event.payload.creature_state);
      setEggCount(event.payload.egg_count);
    });
    return () => { unlisten.then(f => f()); };
  }, []);

  return { creatureState, eggCount, loading, setCreatureState, setEggCount };
}
```

- [ ] **Step 3: Write `src/hooks/useCommands.ts`**

```typescript
import { invoke } from '@tauri-apps/api/core';
import {
  CreatureState, CreatureStats, CodexEntry,
  Settings, SettingsPatch, CollectionEntry
} from '../types';

export function useCommands() {
  const feed = (creatureId: number) =>
    invoke<CreatureStats>('feed', { creature_id: creatureId });

  const play = (creatureId: number) =>
    invoke<CreatureStats>('play', { creature_id: creatureId });

  const hatchEgg = () =>
    invoke<CreatureState>('hatch_egg');

  const setActiveCreature = (creatureId: number) =>
    invoke<void>('set_active_creature', { creature_id: creatureId });

  const getCodex = () =>
    invoke<CodexEntry[]>('get_codex');

  const getSettings = () =>
    invoke<Settings>('get_settings');

  const updateSettings = (patch: SettingsPatch) =>
    invoke<Settings>('update_settings', { patch });

  const getCollection = () =>
    invoke<CollectionEntry[]>('get_collection');

  return { feed, play, hatchEgg, setActiveCreature, getCodex, getSettings, updateSettings, getCollection };
}
```

- [ ] **Step 4: Install Tauri API package (if not already present)**

```bash
npm install @tauri-apps/api
```

- [ ] **Step 5: Commit**

```bash
git add src/types.ts src/hooks/
git commit -m "feat: TypeScript types and React hooks"
```

---

## Task 10: Widget components

**Files:**
- Create: `src/components/Widget/CreatureDisplay.tsx`
- Create: `src/components/Widget/StatBars.tsx`
- Create: `src/components/Widget/ActionButtons.tsx`

- [ ] **Step 1: Write `CreatureDisplay.tsx`**

```tsx
import { useEffect, useRef, useState } from 'react';
import { CreatureState } from '../../types';

interface Props {
  state: CreatureState;
}

function getFrames(state: CreatureState): string[] {
  const { species, stats } = state;
  if (stats.hunger <= 30) return [species.ascii_hungry];
  if (stats.happiness >= 80 && species.ascii_special) {
    return [species.ascii_happy, species.ascii_special];
  }
  return [species.ascii_idle, species.ascii_happy];
}

export function CreatureDisplay({ state }: Props) {
  const [frameIdx, setFrameIdx] = useState(0);
  const frames = getFrames(state);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    if (intervalRef.current) clearInterval(intervalRef.current);
    setFrameIdx(0);
    intervalRef.current = setInterval(() => {
      setFrameIdx(i => (i + 1) % frames.length);
    }, 500);
    return () => { if (intervalRef.current) clearInterval(intervalRef.current); };
  }, [state.creature.id, state.stats.hunger, state.stats.happiness]);

  const name = state.creature.nickname ?? state.species.name;

  return (
    <div className="creature-display">
      <div className="creature-name">{name}</div>
      <pre className="creature-ascii">{frames[frameIdx]}</pre>
      <div className="creature-age">Age: {state.creature.age_ticks} ticks</div>
    </div>
  );
}
```

- [ ] **Step 2: Write `StatBars.tsx`**

```tsx
import { CreatureStats } from '../../types';

interface Props {
  stats: CreatureStats;
}

function StatBar({ label, value, color }: { label: string; value: number; color: string }) {
  return (
    <div className="stat-row">
      <span className="stat-label">{label}</span>
      <div className="stat-bar-track">
        <div
          className="stat-bar-fill"
          style={{ width: `${value}%`, background: color }}
        />
      </div>
      <span className="stat-value">{value}</span>
    </div>
  );
}

export function StatBars({ stats }: Props) {
  return (
    <div className="stat-bars">
      <StatBar label="Hunger"    value={stats.hunger}    color="#e88" />
      <StatBar label="Happiness" value={stats.happiness} color="#8e8" />
      <StatBar label="Energy"    value={stats.energy}    color="#88e" />
    </div>
  );
}
```

- [ ] **Step 3: Write `ActionButtons.tsx`**

```tsx
import { CreatureState, CreatureStats } from '../../types';

interface Props {
  state: CreatureState;
  onFeed: () => void;
  onPlay: () => void;
  onHatch: () => void;
  onExpand: () => void;
  disabled: boolean;
}

export function ActionButtons({ state, onFeed, onPlay, onHatch, onExpand, disabled }: Props) {
  return (
    <div className="action-buttons">
      <button onClick={onFeed} disabled={disabled || state.stats.hunger >= 100}>
        Feed
      </button>
      <button onClick={onPlay} disabled={disabled || state.stats.energy < 10}>
        Play
      </button>
      {state.egg_count > 0 && (
        <button onClick={onHatch} disabled={disabled} className="hatch-btn">
          🥚 Hatch ({state.egg_count})
        </button>
      )}
      <button onClick={onExpand} className="expand-btn">⋯</button>
    </div>
  );
}
```

- [ ] **Step 4: Commit**

```bash
git add src/components/Widget/
git commit -m "feat: Widget components — CreatureDisplay, StatBars, ActionButtons"
```

---

## Task 11: Expanded panel components

**Files:**
- Create: `src/components/Expanded/Codex.tsx`
- Create: `src/components/Expanded/CreatureManager.tsx`
- Create: `src/components/Expanded/Settings.tsx`

- [ ] **Step 1: Write `Codex.tsx`**

```tsx
import { useEffect, useState } from 'react';
import { CodexEntry, RarityTier } from '../../types';
import { useCommands } from '../../hooks/useCommands';

const RARITY_COLORS: Record<RarityTier, string> = {
  common: '#aaa',
  uncommon: '#4af',
  rare: '#a4f',
  legendary: '#fa4',
};

function silhouette(ascii: string): string {
  return ascii.replace(/[^\n]/g, '░');
}

export function Codex() {
  const { getCodex } = useCommands();
  const [entries, setEntries] = useState<CodexEntry[]>([]);

  useEffect(() => {
    getCodex().then(setEntries);
  }, []);

  return (
    <div className="codex">
      <h2>Codex</h2>
      <div className="codex-grid">
        {entries.map(entry => (
          <div key={entry.species.id} className="codex-card">
            <pre className="codex-ascii">
              {entry.discovered ? entry.species.ascii_idle : silhouette(entry.species.ascii_idle)}
            </pre>
            <div className="codex-name">
              {entry.discovered ? entry.species.name : '???'}
            </div>
            <div className="codex-tier" style={{ color: RARITY_COLORS[entry.species.rarity_tier] }}>
              {entry.species.rarity_tier}
            </div>
            {entry.discovered && (
              <div className="codex-detail">
                <div>{entry.species.codex_flavor_text}</div>
                <div>Hatched {entry.times_hatched}×</div>
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
```

- [ ] **Step 2: Write `CreatureManager.tsx`**

```tsx
import { useEffect, useState } from 'react';
import { CollectionEntry } from '../../types';
import { useCommands } from '../../hooks/useCommands';

interface Props {
  activeCreatureId: number | null;
  onSwitched: () => void;
}

export function CreatureManager({ activeCreatureId, onSwitched }: Props) {
  const { getCollection, setActiveCreature } = useCommands();
  const [collection, setCollection] = useState<CollectionEntry[]>([]);

  useEffect(() => {
    getCollection().then(setCollection);
  }, [activeCreatureId]);

  const handleSwitch = async (id: number) => {
    await setActiveCreature(id);
    onSwitched();
  };

  return (
    <div className="creature-manager">
      <h2>Collection</h2>
      {collection.length === 0 && <p>No creatures yet. Hatch your first egg!</p>}
      {collection.map(({ creature, species }) => (
        <div key={creature.id} className={`collection-entry ${creature.is_active ? 'active' : ''}`}>
          <pre className="collection-ascii">{species.ascii_idle}</pre>
          <div className="collection-info">
            <div className="collection-name">{creature.nickname ?? species.name}</div>
            <div className="collection-species">{species.name} · {species.rarity_tier}</div>
            <div className="collection-age">Age: {creature.age_ticks} ticks</div>
          </div>
          {!creature.is_active && creature.status === 'alive' && (
            <button onClick={() => handleSwitch(creature.id)}>Set Active</button>
          )}
          {creature.is_active && <span className="active-badge">Active</span>}
        </div>
      ))}
    </div>
  );
}
```

- [ ] **Step 3: Write `Settings.tsx`**

```tsx
import { useEffect, useState } from 'react';
import { Settings as SettingsType } from '../../types';
import { useCommands } from '../../hooks/useCommands';

interface Props {
  onChanged: (settings: SettingsType) => void;
}

export function Settings({ onChanged }: Props) {
  const { getSettings, updateSettings } = useCommands();
  const [settings, setSettings] = useState<SettingsType | null>(null);

  useEffect(() => {
    getSettings().then(setSettings);
  }, []);

  if (!settings) return <div>Loading settings...</div>;

  const patch = async (changes: Partial<SettingsType>) => {
    const updated = await updateSettings(changes);
    setSettings(updated);
    onChanged(updated);
  };

  return (
    <div className="settings">
      <h2>Settings</h2>

      <label className="setting-row">
        <span>Always on top</span>
        <input
          type="checkbox"
          checked={settings.always_on_top}
          onChange={e => patch({ always_on_top: e.target.checked })}
        />
      </label>

      <label className="setting-row">
        <span>Show tray tooltip</span>
        <input
          type="checkbox"
          checked={settings.show_tray_tooltip}
          onChange={e => patch({ show_tray_tooltip: e.target.checked })}
        />
      </label>

      <label className="setting-row">
        <span>Tick interval (seconds)</span>
        <input
          type="number"
          min={10}
          max={3600}
          value={settings.tick_interval_secs}
          onChange={e => patch({ tick_interval_secs: parseInt(e.target.value, 10) })}
        />
      </label>
    </div>
  );
}
```

- [ ] **Step 4: Commit**

```bash
git add src/components/Expanded/
git commit -m "feat: Expanded panel — Codex, CreatureManager, Settings"
```

---

## Task 12: App layout + mode switching

**Files:**
- Modify: `src/App.tsx`

- [ ] **Step 1: Write `App.tsx`**

Replace `src/App.tsx`:

```tsx
import { useState, useEffect, useRef } from 'react';
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window';
import { useGameState } from './hooks/useGameState';
import { useCommands } from './hooks/useCommands';
import { CreatureDisplay } from './components/Widget/CreatureDisplay';
import { StatBars } from './components/Widget/StatBars';
import { ActionButtons } from './components/Widget/ActionButtons';
import { Codex } from './components/Expanded/Codex';
import { CreatureManager } from './components/Expanded/CreatureManager';
import { Settings } from './components/Expanded/Settings';
import { CreatureStats } from './types';

const WIDGET_SIZE  = new LogicalSize(220, 280);
const EXPANDED_SIZE = new LogicalSize(420, 560);

type Tab = 'codex' | 'collection' | 'settings';

export default function App() {
  const [expanded, setExpanded] = useState(false);
  const [activeTab, setActiveTab] = useState<Tab>('codex');
  const [busy, setBusy] = useState(false);
  const { creatureState, eggCount, loading, setCreatureState, setEggCount } = useGameState();
  const { feed, play, hatchEgg } = useCommands();
  const appWindow = useRef(getCurrentWindow());

  useEffect(() => {
    const win = appWindow.current;
    win.setSize(expanded ? EXPANDED_SIZE : WIDGET_SIZE);
  }, [expanded]);

  const handleFeed = async () => {
    if (!creatureState) return;
    setBusy(true);
    try {
      const stats: CreatureStats = await feed(creatureState.creature.id);
      setCreatureState({ ...creatureState, stats });
    } finally {
      setBusy(false);
    }
  };

  const handlePlay = async () => {
    if (!creatureState) return;
    setBusy(true);
    try {
      const stats: CreatureStats = await play(creatureState.creature.id);
      setCreatureState({ ...creatureState, stats });
    } finally {
      setBusy(false);
    }
  };

  const handleHatch = async () => {
    setBusy(true);
    try {
      const newState = await hatchEgg();
      setCreatureState(newState);
      setEggCount(newState.egg_count);
    } finally {
      setBusy(false);
    }
  };

  if (loading) {
    return <div className="widget loading">...</div>;
  }

  return (
    <div className={`app ${expanded ? 'expanded' : 'widget'}`}>
      {/* Widget section — always visible */}
      <div className="widget-section">
        {creatureState ? (
          <>
            <CreatureDisplay state={creatureState} />
            <StatBars stats={creatureState.stats} />
            <ActionButtons
              state={{ ...creatureState, egg_count: eggCount }}
              onFeed={handleFeed}
              onPlay={handlePlay}
              onHatch={handleHatch}
              onExpand={() => setExpanded(e => !e)}
              disabled={busy}
            />
          </>
        ) : (
          <div className="no-creature">
            {eggCount > 0
              ? <button onClick={handleHatch} disabled={busy}>🥚 Hatch your first egg!</button>
              : <p>Keep playing to earn eggs!</p>
            }
            <button onClick={() => setExpanded(e => !e)} className="expand-btn">⋯</button>
          </div>
        )}
      </div>

      {/* Expanded section — only when expanded */}
      {expanded && (
        <div className="expanded-section">
          <div className="tab-bar">
            {(['codex', 'collection', 'settings'] as Tab[]).map(tab => (
              <button
                key={tab}
                className={activeTab === tab ? 'tab active' : 'tab'}
                onClick={() => setActiveTab(tab)}
              >
                {tab.charAt(0).toUpperCase() + tab.slice(1)}
              </button>
            ))}
          </div>
          <div className="tab-content">
            {activeTab === 'codex'      && <Codex />}
            {activeTab === 'collection' && (
              <CreatureManager
                activeCreatureId={creatureState?.creature.id ?? null}
                onSwitched={() => {}} // game-tick event will update state
              />
            )}
            {activeTab === 'settings'   && <Settings onChanged={() => {}} />}
          </div>
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 2: Install `@tauri-apps/api/window` (already in @tauri-apps/api)**

Verify the import works — `LogicalSize` and `getCurrentWindow` are exported from `@tauri-apps/api/window`.

```bash
npm run build 2>&1 | head -20
```

Expected: no TypeScript errors for the imports.

- [ ] **Step 3: Commit**

```bash
git add src/App.tsx
git commit -m "feat: App layout with widget/expanded mode switching"
```

---

## Task 13: CSS styling

**Files:**
- Modify: `src/index.css`

- [ ] **Step 1: Write `src/index.css`**

Replace the contents of `src/index.css`:

```css
*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

:root {
  --bg: #1a1a2e;
  --surface: #16213e;
  --border: #0f3460;
  --accent: #e94560;
  --text: #eee;
  --text-muted: #888;
  --font-mono: 'Cascadia Code', 'Fira Mono', 'Consolas', monospace;
}

html, body, #root { height: 100%; width: 100%; overflow: hidden; }

body {
  background: var(--bg);
  color: var(--text);
  font-family: var(--font-mono);
  font-size: 12px;
  user-select: none;
  -webkit-user-select: none;
}

/* ── Layout ── */
.app { display: flex; flex-direction: column; height: 100vh; }
.widget-section { padding: 12px; display: flex; flex-direction: column; gap: 8px; }
.expanded-section { flex: 1; display: flex; flex-direction: column; overflow: hidden; border-top: 1px solid var(--border); }

/* ── Creature display ── */
.creature-display { text-align: center; }
.creature-name { font-size: 13px; font-weight: bold; margin-bottom: 4px; }
.creature-ascii { font-size: 11px; line-height: 1.4; color: #cce; white-space: pre; }
.creature-age { font-size: 10px; color: var(--text-muted); margin-top: 2px; }

/* ── Stat bars ── */
.stat-bars { display: flex; flex-direction: column; gap: 4px; }
.stat-row { display: flex; align-items: center; gap: 6px; }
.stat-label { width: 58px; font-size: 10px; color: var(--text-muted); }
.stat-bar-track { flex: 1; height: 6px; background: var(--border); border-radius: 3px; overflow: hidden; }
.stat-bar-fill { height: 100%; border-radius: 3px; transition: width 0.3s ease; }
.stat-value { width: 24px; text-align: right; font-size: 10px; color: var(--text-muted); }

/* ── Action buttons ── */
.action-buttons { display: flex; gap: 6px; flex-wrap: wrap; }
.action-buttons button {
  flex: 1; padding: 5px 4px; background: var(--surface);
  border: 1px solid var(--border); border-radius: 4px;
  color: var(--text); font-family: var(--font-mono); font-size: 11px;
  cursor: pointer; transition: background 0.15s;
}
.action-buttons button:hover:not(:disabled) { background: var(--border); }
.action-buttons button:disabled { opacity: 0.4; cursor: default; }
.hatch-btn { border-color: #fa4 !important; color: #fa4 !important; }
.expand-btn { flex: 0 0 28px !important; }

/* ── No creature ── */
.no-creature { text-align: center; padding: 20px; display: flex; flex-direction: column; gap: 10px; }

/* ── Loading ── */
.loading { display: flex; align-items: center; justify-content: center; height: 100vh; color: var(--text-muted); }

/* ── Tab bar ── */
.tab-bar { display: flex; border-bottom: 1px solid var(--border); }
.tab {
  flex: 1; padding: 8px; background: none; border: none;
  border-bottom: 2px solid transparent; color: var(--text-muted);
  font-family: var(--font-mono); font-size: 11px; cursor: pointer;
  transition: all 0.15s;
}
.tab.active { color: var(--text); border-bottom-color: var(--accent); }
.tab-content { flex: 1; overflow-y: auto; padding: 12px; }

/* ── Codex ── */
.codex h2 { font-size: 12px; margin-bottom: 10px; color: var(--text-muted); text-transform: uppercase; }
.codex-grid { display: grid; grid-template-columns: repeat(2, 1fr); gap: 8px; }
.codex-card {
  background: var(--surface); border: 1px solid var(--border);
  border-radius: 6px; padding: 8px; display: flex; flex-direction: column; gap: 4px;
}
.codex-ascii { font-size: 9px; line-height: 1.3; white-space: pre; color: #aac; }
.codex-name { font-size: 11px; font-weight: bold; }
.codex-tier { font-size: 9px; text-transform: uppercase; }
.codex-detail { font-size: 9px; color: var(--text-muted); line-height: 1.4; }

/* ── Collection ── */
.creature-manager h2 { font-size: 12px; margin-bottom: 10px; color: var(--text-muted); text-transform: uppercase; }
.collection-entry {
  display: flex; align-items: center; gap: 8px;
  padding: 8px; border: 1px solid var(--border); border-radius: 6px;
  margin-bottom: 6px; background: var(--surface);
}
.collection-entry.active { border-color: var(--accent); }
.collection-ascii { font-size: 9px; line-height: 1.3; white-space: pre; }
.collection-info { flex: 1; }
.collection-name { font-size: 11px; font-weight: bold; }
.collection-species { font-size: 10px; color: var(--text-muted); }
.collection-age { font-size: 10px; color: var(--text-muted); }
.active-badge { font-size: 9px; color: var(--accent); }

/* ── Settings ── */
.settings h2 { font-size: 12px; margin-bottom: 10px; color: var(--text-muted); text-transform: uppercase; }
.setting-row {
  display: flex; justify-content: space-between; align-items: center;
  padding: 8px 0; border-bottom: 1px solid var(--border);
  font-size: 11px;
}
.setting-row input[type="checkbox"] { cursor: pointer; }
.setting-row input[type="number"] {
  width: 60px; background: var(--border); border: none;
  color: var(--text); font-family: var(--font-mono); padding: 2px 4px; border-radius: 3px;
}
```

- [ ] **Step 2: Verify the app looks correct**

```bash
npm run tauri dev
```

Expected: widget renders at 220×280 with dark theme, monospace font, creature ASCII art, stat bars, and action buttons. Clicking ⋯ expands the panel with tabs.

- [ ] **Step 3: Commit**

```bash
git add src/index.css
git commit -m "feat: dark monospace CSS theme for widget and expanded panel"
```

---

## Task 14: End-to-end smoke test + bench feature

**Files:**
- Modify: `src-tauri/Cargo.toml` (bench feature already declared)

- [ ] **Step 1: Run with bench feature enabled and observe output**

```bash
cd src-tauri && cargo build --features bench
npm run tauri dev -- -- --features bench
```

Expected: on each game tick, stderr prints something like:
```
[bench] tick took 1.23ms
```

- [ ] **Step 2: Stress test — set tick interval to 1 second**

In the running app, open Settings and set tick interval to 10. Observe:
1. Stat bars update every 10 seconds
2. CPU usage stays low (check with `htop` or Task Manager)
3. Memory stays under 60MB

Target: idle CPU < 0.5%, memory < 60MB. If either is exceeded, investigate before merging.

- [ ] **Step 3: Full game loop smoke test**

Manually verify the following sequence in the running app:
1. App launches — creature shown (auto-seeded on first run if no active creature, or shows hatch prompt)
2. Click Feed — hunger bar increases immediately
3. Click Play — happiness bar increases immediately
4. Stat bars decay after one tick
5. Click ⋯ — window expands to 420×560
6. Codex tab shows all 8 species (discovered or silhouetted)
7. Settings tab — toggle always-on-top, verify window behavior changes
8. Collection tab — shows current creature
9. Collapse back to widget size
10. Tray icon visible — Show/Hide and Quit items work

- [ ] **Step 4: Final commit**

```bash
git add -A
git commit -m "feat: Codagatchi v1 — complete implementation"
```

---

## Self-Review Notes

### Spec coverage check

| Spec requirement | Covered by |
|-----------------|-----------|
| Tauri 2 + React 18 + Vite | Task 1 |
| SQLite via rusqlite | Task 2 |
| Single window, widget/expanded modes | Task 12 |
| Always-on-top toggle | Task 5 (update_settings), Task 13 (Settings.tsx) |
| System tray (show/hide, quit) | Task 7 |
| Game loop (tokio, 60s tick) | Task 6 |
| APM ActivitySnapshot stub | Task 6 (None passed) |
| Stat decay: hunger −2, happiness −1, energy +1 | Task 4 (creatures.rs) |
| Feed: hunger +20, energy −5, XP +2 | Task 4 |
| Play: happiness +15, energy −10, XP +3 | Task 4 |
| Starvation warning at hunger ≤ 20 | Task 4 + Task 6 |
| Death after 3 starving ticks | Task 4 + Task 6 |
| Milestones at XP 50/200/500/… | Task 4 + Task 6 |
| Max 3 eggs | Task 6 |
| 8 species with 4 rarity tiers | Task 3 |
| Weighted hatch roll (60/25/12/3%) | Task 5 |
| Prefer undiscovered on hatch | Task 5 |
| Codex with discovered/silhouette states | Task 11 |
| Collection / active creature switching | Task 11 |
| OS notifications (egg, starvation, death) | Task 6 |
| ASCII animation (500ms frame cycle) | Task 10 |
| Hungry frame when hunger ≤ 30 | Task 10 |
| Resource bench feature | Task 14 |
| Window hide (not minimize) when hidden | Task 7, Task 12 |
