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
