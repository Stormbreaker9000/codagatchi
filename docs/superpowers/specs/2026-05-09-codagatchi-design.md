# Codagatchi — Design Spec

**Date:** 2026-05-09
**Status:** Approved

---

## Overview

Codagatchi is a lightweight desktop tamagotchi application built with Tauri 2. It lives as a small floating widget and system tray icon. Users hatch ASCII creatures with a rarity system, care for them via simple interactions, and build a Codex of discovered species over time. A future APM (actions-per-minute) integration will tie creature mood to keyboard activity.

---

## Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri 2 |
| Frontend | React 18 + Vite + TypeScript |
| Backend | Rust (Tauri backend) |
| Persistence | SQLite via `tauri-plugin-sql` |
| Notifications | `tauri-plugin-notification` (native OS) |
| Game loop | Tokio background task |

---

## Architecture

### Core Boundary

Rust owns all game state and business logic. React is a display layer only — it calls Tauri commands and listens for Tauri events. It never mutates state directly. This boundary keeps the future APM integration clean: the Rust background task gains a new data source (keyboard activity) without any frontend changes.

### Window Model

A single Tauri window with two modes:

- **Widget mode** — 220×280px, borderless, optionally always-on-top, positioned at a user-chosen screen corner. Shows creature, stat bars, and action buttons.
- **Expanded mode** — grows in-place to ~420×560px via `window.setSize()` with a CSS transition. Reveals tabbed panel: Creature management, Codex, Settings. Collapses back on outside click or close button.

### System Tray

Always present regardless of widget visibility. Tooltip shows creature name and current mood. Menu items:
- Show / Hide widget
- Hunger and happiness at a glance (read-only)
- Quit

### Game Loop

A `tokio::spawn` background task started at app launch. Every tick (default 60s, configurable):

1. Load active creature stats from SQLite
2. Apply stat decay / regen
3. Check lifecycle events (starvation warning, death, milestone unlock)
4. Write updated stats back to SQLite
5. Emit `game-tick` Tauri event to frontend with updated `CreatureState` as JSON

The tick interval is read from `settings` at startup and reloaded on change. When the widget window is hidden, `window.hide()` is called (not minimize) to suspend WebView rendering while the game loop continues uninterrupted.

**APM hook point:** `game_loop.rs` accepts an optional `ActivitySnapshot` (keyboard APM value). In v1 this is always `None`. When the APM module is added, it populates this value each tick to influence happiness decay rate — no other changes required.

---

## Rust Backend

### Module Structure

```
src-tauri/src/
  main.rs          — Tauri builder, plugin registration, tray setup
  game_loop.rs     — background tick task, ActivitySnapshot stub
  commands.rs      — all Tauri commands exposed to frontend
  creatures.rs     — species definitions, stat logic, lifecycle rules
  db.rs            — SQLite migrations and query helpers
  tray.rs          — system tray construction and event handling
```

### Tauri Commands

```
get_active_creature()        → CreatureState
get_codex()                  → Vec<CodexEntry>
get_settings()               → Settings
feed(creature_id)            → CreatureStats
play(creature_id)            → CreatureStats
hatch_egg()                  → Creature          // only callable if egg is available
set_active_creature(id)      → ()
update_settings(patch)       → Settings
```

### Species Definitions

Static `Vec<Species>` in `creatures.rs`, seeded into the `species` SQLite table on first run. Adding a new species later means adding one struct entry and a migration — no external config files.

---

## Data Model

### `species` (static, seeded at startup)

| Column | Type | Notes |
|--------|------|-------|
| id | TEXT PK | slug, e.g. `"blob"` |
| name | TEXT | display name |
| rarity_tier | TEXT | `common \| uncommon \| rare \| legendary` |
| ascii_idle | TEXT | multiline string, ~8×6 chars |
| ascii_happy | TEXT | |
| ascii_hungry | TEXT | |
| ascii_special | TEXT | nullable, Legendary only |
| description | TEXT | short flavor text |
| codex_flavor_text | TEXT | shown in Codex on discovery |

### `creatures` (the player's collection)

| Column | Type | Notes |
|--------|------|-------|
| id | INTEGER PK | |
| species_id | TEXT FK | |
| nickname | TEXT | nullable |
| hatched_at | INTEGER | Unix timestamp |
| retired_at | INTEGER | nullable |
| is_active | BOOLEAN | only one row true at a time |
| age_ticks | INTEGER | incremented each game tick |
| status | TEXT | `egg \| alive \| retired \| dead` |

### `creature_stats`

| Column | Type | Notes |
|--------|------|-------|
| creature_id | INTEGER FK | |
| hunger | INTEGER | 0–100 |
| happiness | INTEGER | 0–100 |
| energy | INTEGER | 0–100 |
| xp | INTEGER | cumulative |
| milestone_count | INTEGER | eggs awarded so far |
| starving_ticks | INTEGER | consecutive ticks at hunger = 0; resets to 0 when hunger > 0 |

Kept separate from `creatures` so historical records stay clean.

### `codex`

| Column | Type | Notes |
|--------|------|-------|
| species_id | TEXT FK | one row per species |
| discovered | BOOLEAN | |
| first_hatched_at | INTEGER | nullable |
| times_hatched | INTEGER | |

### `settings`

| Column | Type | Default |
|--------|------|---------|
| always_on_top | BOOLEAN | true |
| window_x | INTEGER | screen-dependent |
| window_y | INTEGER | screen-dependent |
| tick_interval_secs | INTEGER | 60 |
| show_tray_tooltip | BOOLEAN | true |

---

## Stat Mechanics

### Decay / Regen (per tick, tunable constants in Rust)

| Stat | Per Tick |
|------|---------|
| Hunger | −2 |
| Happiness | −1 |
| Energy | +1 (passive regen) |

### Interactions

| Action | Effect |
|--------|--------|
| Feed | Hunger +20, Energy −5, XP +2 |
| Play | Happiness +15, Energy −10, XP +3 |

### Lifecycle Events

- **Starvation warning:** hunger ≤ 20 for 1 tick → tray icon changes, OS notification: `"[Name] is starving!"`
- **Death:** hunger = 0 for 3 consecutive ticks → creature status set to `dead`, OS notification: `"[Name] has died."` with "Hatch a new egg?" action if egg is available
- **Milestone unlock:** XP thresholds 50 / 200 / 500 (and multiples of 500 thereafter) → `milestone_count++`, egg added to inventory, OS notification: `"🥚 A new egg is ready!"`

### Egg Inventory

- Maximum 3 unhatched eggs at once (prevents hoarding)
- Eggs persist in `creatures` table with `status = 'egg'`

---

## Rarity & Hatching System

### Tiers and Weights

| Tier | Hatch Weight | Starting Species Count |
|------|-------------|----------------------|
| Common | 60% | 4 |
| Uncommon | 25% | 2 |
| Rare | 12% | 1 |
| Legendary | 3% | 1 |

### Hatch Flow

1. Player taps "Hatch" button in widget (only visible when an egg is in inventory)
2. Rust rolls rarity tier (weighted random), then picks a random undiscovered species within that tier if any remain; otherwise picks any species in that tier
3. New row inserted into `creatures` with `status = 'alive'`
4. `codex` entry updated: `discovered = true`, `first_hatched_at` set on first discovery, `times_hatched` incremented
5. If player has no active creature, new creature becomes active immediately; otherwise it goes to collection and player is notified

### Starting Roster (8 species)

| Name | Tier | Concept |
|------|------|---------|
| Blob | Common | amorphous, round, simple |
| Kitten | Common | small ASCII cat |
| Sparrow | Common | tiny round bird |
| Bearcub | Common | sleepy round bear |
| Drakling | Uncommon | small dragon |
| Specter | Uncommon | floating ghost |
| Phoenix | Rare | flame bird |
| Voidling | Legendary | cosmic entity, 4 animation frames |

---

## React Frontend

### Component Structure

```
src/
  components/
    Widget/
      CreatureDisplay.tsx    — ASCII renderer + frame animation
      StatBars.tsx           — hunger, happiness, energy bars
      ActionButtons.tsx      — feed, play, hatch (conditional on egg)
    Expanded/
      Codex.tsx              — species grid, discovered vs undiscovered
      CreatureManager.tsx    — collection list, set active creature
      Settings.tsx           — always-on-top, tick interval, window position
  hooks/
    useGameState.ts          — subscribes to game-tick event, exposes CreatureState
    useCommands.ts           — typed wrappers around all Tauri invoke() calls
  App.tsx                    — widget/expanded mode toggle, layout root
```

### State Management

No external state library. `useGameState` subscribes to the `game-tick` Tauri event and stores the latest `CreatureState` in a single `useState`. User actions call `useCommands`, which invokes Tauri commands and applies optimistic local updates while Rust confirms. Source of truth is always the Rust backend.

### Widget ↔ Expanded Transition

CSS `transition` on window dimensions via `window.setSize()`, combined with a React layout state flag. Same DOM tree — no second window, no portal. Expanded panel renders below/beside the creature view and is hidden via CSS when in widget mode (not unmounted, to avoid re-fetch on every open).

### ASCII Renderer

`CreatureDisplay` cycles through the species' frame strings on a 500ms `setInterval`. Frames are multiline strings returned from Rust in the species payload. State-aware: idle frames normally, happy frame on interaction, hungry frame when hunger ≤ 30.

### Codex Display

Grid of all 8 species slots. Discovered entries show ASCII idle frame, name, rarity tier badge, flavor text, and times hatched. Undiscovered entries show a `░`-block silhouette (same dimensions as real creature), `???` for the name, and the rarity tier — giving players something to chase without spoiling the design.

---

## Notifications

Uses `tauri-plugin-notification` for native OS notifications (Windows Toast / macOS Notification Center / Linux libnotify).

| Event | Message | Action Button |
|-------|---------|--------------|
| Egg ready | `"🥚 A new egg is ready!"` | "Hatch now" (opens widget) |
| Starvation warning | `"[Name] is starving!"` | "Feed now" (opens widget) |
| Death | `"[Name] has died."` | "Hatch a new egg?" (if egg available) |

Notification permission is requested once on first launch via Tauri's automatic OS prompt.

---

## Resource Footprint Targets

| Metric | Target |
|--------|--------|
| Idle CPU (widget visible) | < 0.5% |
| Idle memory | < 60MB |
| Tick CPU spike duration | < 50ms |
| Background-only (window hidden) | < 20MB working set |

### Testing Approach

- `bench` Cargo feature that logs tick duration and memory usage to a file
- Manual profiling on Windows (Task Manager) and Linux (`htop`) at each milestone
- Stress test: tick interval set to 1s, monitor resource usage under sustained load
- Resource footprint is an explicit acceptance criterion — idle CPU above 1% is investigated before merge

---

## Future: APM Integration

The game loop's `ActivitySnapshot` stub in v1 accepts `None`. When APM is added:

- A new `keyboard_monitor` module in `game_loop.rs` samples keystrokes per minute on a rolling 60s window
- High APM (active coding session) → happiness decay slows (creature is happy you're working)
- Low APM (idle) → happiness decays at normal rate
- Very high APM sustained → creature gains bonus XP per tick

No frontend changes are required. The Rust tick function signature already accommodates it.

---

## Out of Scope for v1

- Multiple simultaneous active creatures
- Creature breeding or trading
- Cloud sync or cross-device state
- APM monitoring (stubbed, not implemented)
- Creature aging into different visual stages
- Sound effects
