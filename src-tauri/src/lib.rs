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
            let app_data = app.path().app_data_dir()
                .expect("could not resolve app data dir");
            std::fs::create_dir_all(&app_data)?;
            let db_path = app_data.join("codagatchi.db");

            // Seeding connection — dropped at end of this block
            {
                let conn = db::open(&db_path).expect("failed to open database");
                let all_species = creatures::all_species();
                db::seed_species(&conn, &all_species).expect("failed to seed species");

                // Auto-seed a starter egg on first run (no creatures at all)
                let egg_count = db::count_eggs(&conn).unwrap_or(0);
                let creature_count: i32 = conn.query_row(
                    "SELECT COUNT(*) FROM creatures WHERE status != 'egg'",
                    [],
                    |r| r.get(0),
                ).unwrap_or(0);
                if egg_count == 0 && creature_count == 0 {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;
                    db::add_egg(&conn, now).expect("failed to add starter egg");
                }
            }

            // Commands connection
            app.manage(AppState {
                db: Mutex::new(db::open(&db_path).expect("failed to open db for commands")),
            });

            // Game loop connection
            let loop_db = Arc::new(Mutex::new(
                db::open(&db_path).expect("failed to open db for game loop"),
            ));
            game_loop::start(app.handle().clone(), loop_db);

            tray::build(app)?;

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
