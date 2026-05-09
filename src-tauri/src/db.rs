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

/// Updates stats only if the creature is still alive. Returns false if the creature
/// died between the caller's read and this write (game_loop TOCTOU guard).
pub fn update_stats_if_alive(conn: &Connection, stats: &CreatureStats) -> Result<bool> {
    let rows = conn.execute(
        "UPDATE creature_stats SET hunger=?1, happiness=?2, energy=?3,
         xp=?4, milestone_count=?5, starving_ticks=?6
         WHERE creature_id=?7
           AND EXISTS (SELECT 1 FROM creatures WHERE id=?7 AND status='alive')",
        params![
            stats.hunger, stats.happiness, stats.energy,
            stats.xp, stats.milestone_count, stats.starving_ticks,
            stats.creature_id
        ],
    )?;
    Ok(rows > 0)
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
    conn.execute(
        "INSERT INTO creatures (species_id, hatched_at, is_active, status)
         VALUES ('', ?1, 0, 'egg')",
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
