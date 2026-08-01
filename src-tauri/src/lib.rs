mod detect;
mod models;
mod runner;
mod state;
pub mod storage;

use tauri::Manager;

pub use models::{
    ExecType, OutputEvent, RunInput, RunRecord, RunStatus, Script, ShellConfig, ShellKind,
};
pub use state::AppState;

// 命令函数放进私有子模块：`#[tauri::command]` 对 `pub fn` 会同时生成
// `#[macro_export]` 的 `__cmd__X`/`__tauri_command_name_X` 宏（导出到 crate 根）
// 和模块内的 `pub use {__cmd__X, ...}`。若函数与 run() 同在 crate 根，
// 二者落在同一模块 → E0255 重复定义。子模块把两者分隔开，且
// `generate_handler![commands::list_scripts, ...]` 经路径解析 `commands::__cmd__X`。
mod commands {
    use super::runner;
    use super::storage;
    use super::{AppState, OutputEvent, RunInput, RunRecord, Script, ShellConfig};
    use tauri::ipc::Channel;
    use tauri::State;

    // ---- scripts ----

    #[tauri::command]
    pub async fn list_scripts(state: State<'_, AppState>) -> Result<Vec<Script>, String> {
        Ok(storage::load_scripts(&state.data_dir).await)
    }

    #[tauri::command]
    pub async fn save_script(state: State<'_, AppState>, script: Script) -> Result<Script, String> {
        storage::save_script(&state.data_dir, script).await
    }

    #[tauri::command]
    pub async fn delete_script(state: State<'_, AppState>, id: String) -> Result<(), String> {
        storage::delete_script(&state.data_dir, &id).await
    }

    // ---- shells ----

    #[tauri::command]
    pub async fn list_shells(state: State<'_, AppState>) -> Result<Vec<ShellConfig>, String> {
        Ok(storage::load_shells(&state.data_dir).await)
    }

    #[tauri::command]
    pub async fn save_shell(
        state: State<'_, AppState>,
        shell: ShellConfig,
    ) -> Result<ShellConfig, String> {
        storage::save_shell(&state.data_dir, shell).await
    }

    #[tauri::command]
    pub async fn delete_shell(state: State<'_, AppState>, id: String) -> Result<(), String> {
        storage::delete_shell(&state.data_dir, &id).await
    }

    // ---- runs ----

    #[tauri::command]
    pub async fn run_script(
        state: State<'_, AppState>,
        input: RunInput,
        channel: Channel<OutputEvent>,
    ) -> Result<RunRecord, String> {
        runner::spawn(&state, input, channel).await
    }

    #[tauri::command]
    pub async fn kill_run(state: State<'_, AppState>, run_id: String) -> Result<(), String> {
        runner::kill(&state, &run_id).await
    }

    #[tauri::command]
    pub async fn list_runs(state: State<'_, AppState>) -> Result<Vec<RunRecord>, String> {
        Ok(storage::load_runs(&state.data_dir).await)
    }

    #[tauri::command]
    pub async fn get_run_log(state: State<'_, AppState>, run_id: String) -> Result<String, String> {
        storage::read_log(&state.data_dir, &run_id).await
    }

    #[tauri::command]
    pub async fn clear_history(
        state: State<'_, AppState>,
        script_id: Option<String>,
    ) -> Result<(), String> {
        storage::clear_history(&state.data_dir, script_id.as_deref()).await
    }

    #[tauri::command]
    pub fn running_count(state: State<'_, AppState>) -> u32 {
        state.running_count() as u32
    }
}

// 供集成测试（tests/commands.rs）经 library crate 直接调用各命令函数。
pub use commands::{
    clear_history, delete_script, delete_shell, get_run_log, kill_run, list_runs, list_scripts,
    list_shells, running_count, run_script, save_script, save_shell,
};

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
            app.manage(AppState::new(data_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_scripts, commands::save_script, commands::delete_script,
            commands::list_shells, commands::save_shell, commands::delete_shell,
            commands::run_script, commands::kill_run,
            commands::list_runs, commands::get_run_log, commands::clear_history,
            commands::running_count,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
