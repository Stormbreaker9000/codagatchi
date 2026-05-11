# Codagatchi

A desktop Tamagotchi built with Tauri 2. It lives as a small floating widget and a system tray icon. Hatch ASCII creatures with a rarity system, keep them alive by feeding and playing, and build up a Codex of every species you've discovered.

---

## Features

- **Floating widget** — 220×280px, borderless, always-on-top, draggable to any screen position
- **Expanded panel** — expands in-place to a tabbed view with Codex, Collection, and Settings
- **ASCII creatures** — 8 species across 4 rarity tiers, each with idle/happy/hungry art
- **Stat system** — hunger, happiness, and energy decay/regen each tick; feed and play to keep your creature healthy
- **XP & milestones** — earn XP over time and through interactions; milestones award new eggs
- **Egg inventory** — hatch eggs to discover new species (max 3 unhatched eggs at once)
- **Codex** — tracks every species: discovered entries show art and flavor text; undiscovered entries show a silhouette
- **OS notifications** — starvation warnings, death notices, and new egg alerts via native OS notification
- **System tray** — creature name and mood always visible; show/hide the widget from the tray menu
- **Configurable tick interval** — default 60s, adjustable in settings

---

## Species

| Name | Rarity | Notes |
|------|--------|-------|
| Blob | Common | Amorphous, wobbly |
| Kitten | Common | Small ASCII cat |
| Sparrow | Common | Tiny round bird |
| Bearcub | Common | Perpetually sleepy |
| Drakling | Uncommon | Small dragon |
| Specter | Uncommon | Floating ghost |
| Phoenix | Rare | Flame bird |
| Voidling | Legendary | Cosmic entity |

Hatch weights: Common 60% / Uncommon 25% / Rare 12% / Legendary 3%.

---

## Game Mechanics

**Per-tick stat changes:**

| Stat | Per Tick |
|------|----------|
| Hunger | −2 |
| Happiness | −1 |
| Energy | +1 |

**Interactions:**

| Action | Effect |
|--------|--------|
| Feed | Hunger +20, Energy −5, XP +2 |
| Play | Happiness +15, Energy −10, XP +3 |

**Lifecycle events:**

- Hunger ≤ 20 → starvation warning notification
- Hunger = 0 for 3 consecutive ticks → creature dies
- XP milestones (50 / 200 / 500 / …) → new egg added to inventory

---

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri 2 |
| Frontend | React 19, TypeScript, Vite |
| Backend | Rust |
| Database | SQLite (rusqlite, bundled) |
| Notifications | tauri-plugin-notification |
| Game loop | Tokio background task |

---

## Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [Node.js](https://nodejs.org/) 18+
- Tauri system dependencies for your OS — see [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/)

---

## Development

```bash
npm install
npm run tauri dev
```

This starts the Vite dev server and the Tauri app together with hot reload.

---

## Building

```bash
npm run tauri build
```

Produces a platform-native installer in `src-tauri/target/release/bundle/`.

---

## Architecture

Rust owns all game state and business logic. React is a display layer — it calls Tauri commands and listens for `game-tick` events. It never mutates state directly.

```
src-tauri/src/
  main.rs         — Tauri builder, plugin registration
  lib.rs          — app setup, database init, starter egg seeding
  game_loop.rs    — background tick task
  commands.rs     — Tauri commands exposed to the frontend
  creatures.rs    — species definitions, stat logic, lifecycle rules
  db.rs           — SQLite migrations and query helpers
  tray.rs         — system tray construction and event handling

src/
  components/
    Widget/       — CreatureDisplay, StatBars, ActionButtons
    Expanded/     — Codex, CreatureManager, Settings
  hooks/
    useGameState  — subscribes to game-tick, holds CreatureState
    useCommands   — typed wrappers around Tauri invoke() calls
  App.tsx         — widget/expanded mode toggle, layout root
```

---

## Roadmap

- **APM integration** — the game loop already accepts an `ActivitySnapshot` stub. A future keyboard monitor module will slow happiness decay during active coding sessions and award bonus XP during sustained high-APM periods. No frontend changes will be required.
- Creature aging with distinct visual stages
- Sound effects

---

## License

See [LICENSE](LICENSE).
