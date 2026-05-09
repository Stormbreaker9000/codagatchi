use tauri::State;
use std::sync::Mutex;
use rusqlite::Connection;
use rand::Rng;
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

    let (final_creature, final_stats) = if creature.id == creature_id {
        (creature, stats)
    } else {
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
