mod detect;
mod models;
mod runner;
mod state;
mod storage;

use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let detected = detect::detect_shells();
            if let Err(e) =
                tauri::async_runtime::block_on(storage::merge_detected(&data_dir, &detected))
            {
                eprintln!("[storage] merge detected shells failed: {e}");
            }
            if let Err(e) = tauri::async_runtime::block_on(storage::migrate_running_on_boot(&data_dir))
            {
                eprintln!("[storage] migrate running records failed: {e}");
            }
            app.manage(state::AppState::new(data_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
