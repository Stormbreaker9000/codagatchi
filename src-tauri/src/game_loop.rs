use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use rusqlite::Connection;
use crate::{creatures, db, types::*};

#[cfg(feature = "bench")]
use std::time::Instant;

pub fn start(app: AppHandle, db: Arc<Mutex<Connection>>) {
    tauri::async_runtime::spawn(async move {
        loop {
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
            let egg_count = db::count_eggs(&conn).unwrap_or(0);
            let _ = app.emit("game-tick", TickEvent { creature_state: None, egg_count });
            return;
        }
    };

    let (creature, stats) = row;
    let (updated_stats, events) = creatures::apply_tick(stats, None);

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
        send_notification(app, "Oh no!", &format!("{} has died.", creature_name(&creature)));
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
            send_notification(app, "New egg!", "A new egg is ready to hatch!");
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
