use std::sync::{Arc, Mutex};
use std::time::Duration;

use bg_script_management_lib::{
    get_run_log, kill_run, list_runs, list_scripts, running_count, run_script, save_script,
    storage, AppState, ExecType, OutputEvent, RunInput, RunStatus, Script, ShellConfig, ShellKind,
};
use tauri::ipc::{Channel, InvokeResponseBody};
use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri::Manager;
use tokio::fs;

async fn temp_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("bsm-cmd-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&dir).await.unwrap();
    dir
}

fn build_app(dir: std::path::PathBuf) -> tauri::App<tauri::test::MockRuntime> {
    mock_builder()
        .manage(AppState::new(dir))
        .build(mock_context(noop_assets()))
        .unwrap()
}

fn sample_script() -> Script {
    Script {
        id: String::new(),
        name: "示例脚本".into(),
        description: "desc".into(),
        shell_id: "builtin-cmd".into(),
        exec_type: ExecType::Command,
        command: "echo hi".into(),
        cwd: None,
        env: Default::default(),
        timeout_sec: 0,
        created_at: 0,
        updated_at: 0,
    }
}

/// 返回带空 id 的 shell：`storage::save_shell` 把空 id 视为新建。
fn platform_shell() -> (ShellConfig, String) {
    #[cfg(target_os = "windows")]
    {
        (
            ShellConfig {
                id: String::new(),
                name: "CMD".into(),
                kind: ShellKind::Cmd,
                exe: "cmd.exe".into(),
                args: vec![],
                builtin: false,
            },
            "echo BSM_CMD_OK".into(),
        )
    }
    #[cfg(not(target_os = "windows"))]
    {
        (
            ShellConfig {
                id: String::new(),
                name: "sh".into(),
                kind: ShellKind::Sh,
                exe: "sh".into(),
                args: vec![],
                builtin: false,
            },
            "echo BSM_CMD_OK".into(),
        )
    }
}

fn collect_channel() -> (Channel<OutputEvent>, Arc<Mutex<Vec<OutputEvent>>>) {
    let events = Arc::new(Mutex::new(Vec::new()));
    let sink = events.clone();
    let channel: Channel<OutputEvent> = Channel::new(move |body| {
        if let InvokeResponseBody::Json(s) = body {
            if let Ok(ev) = serde_json::from_str::<OutputEvent>(&s) {
                sink.lock().unwrap().push(ev);
            }
        }
        Ok(())
    });
    (channel, events)
}

async fn wait_for_exit(events: &Arc<Mutex<Vec<OutputEvent>>>, timeout_ms: u64) -> Vec<OutputEvent> {
    let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        {
            let e = events.lock().unwrap();
            if e.iter().any(|ev| matches!(ev, OutputEvent::Exit { .. })) {
                return e.clone();
            }
        }
        if std::time::Instant::now() > deadline {
            panic!("等待 Exit 事件超时");
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

#[tokio::test]
async fn save_script_then_list_and_persist() {
    let dir = temp_dir().await;
    let app = build_app(dir.clone());
    let saved = save_script(app.state::<AppState>(), sample_script())
        .await
        .unwrap();
    assert!(!saved.id.is_empty());
    drop(app);

    // 重启后仍在（持久化生效）
    let app2 = build_app(dir.clone());
    let all = list_scripts(app2.state::<AppState>()).await.unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].name, "示例脚本");
    drop(app2);
    let _ = fs::remove_dir_all(&dir).await;
}

#[tokio::test]
async fn run_script_echo_creates_run_and_log() {
    let dir = temp_dir().await;
    let (shell, cmd) = platform_shell();
    let saved = storage::save_shell(&dir, shell).await.unwrap();
    let shell_id = saved.id.clone();
    assert!(!shell_id.is_empty());

    let app = build_app(dir.clone());
    let (channel, events) = collect_channel();
    let input = RunInput {
        script_id: None,
        script_name: "t".into(),
        shell_id,
        command: cmd,
        exec_type: ExecType::Command,
        cwd: None,
        env: Default::default(),
        timeout_sec: 0,
    };
    let record = run_script(app.state::<AppState>(), input, channel)
        .await
        .unwrap();
    assert_eq!(record.status, RunStatus::Running);

    let evs = wait_for_exit(&events, 10000).await;
    let exit = evs
        .iter()
        .find_map(|e| match e {
            OutputEvent::Exit { code, .. } => Some(*code),
            _ => None,
        })
        .unwrap();
    assert_eq!(exit, Some(0));

    let app2 = build_app(dir.clone());
    let runs = list_runs(app2.state::<AppState>()).await.unwrap();
    let rec = runs.iter().find(|r| r.id == record.id).unwrap();
    assert_eq!(rec.status, RunStatus::Success);
    let log = get_run_log(app2.state::<AppState>(), record.id.clone())
        .await
        .unwrap();
    assert!(log.contains("BSM_CMD_OK"), "日志应包含输出，实际: {log}");
    drop(app2);
    let _ = fs::remove_dir_all(&dir).await;
}

#[tokio::test]
async fn kill_run_long_task_marks_killed() {
    let dir = temp_dir().await;
    let (shell, _) = platform_shell();
    let saved = storage::save_shell(&dir, shell).await.unwrap();
    let shell_id = saved.id.clone();

    let app = build_app(dir.clone());
    let (channel, events) = collect_channel();
    #[cfg(target_os = "windows")]
    let cmd = "ping -n 60 127.0.0.1".to_string();
    #[cfg(not(target_os = "windows"))]
    let cmd = "sleep 60".to_string();
    let input = RunInput {
        script_id: None,
        script_name: "t".into(),
        shell_id,
        command: cmd,
        exec_type: ExecType::Command,
        cwd: None,
        env: Default::default(),
        timeout_sec: 0,
    };
    let record = run_script(app.state::<AppState>(), input, channel)
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(500)).await;
    kill_run(app.state::<AppState>(), record.id.clone())
        .await
        .unwrap();

    let evs = wait_for_exit(&events, 10000).await;
    let exit = evs
        .iter()
        .find_map(|e| match e {
            OutputEvent::Exit { code, .. } => Some(*code),
            _ => None,
        })
        .unwrap();
    assert_eq!(exit, None);

    let app2 = build_app(dir.clone());
    let runs = list_runs(app2.state::<AppState>()).await.unwrap();
    let rec = runs.iter().find(|r| r.id == record.id).unwrap();
    assert_eq!(rec.status, RunStatus::Killed);
    assert_eq!(running_count(app2.state::<AppState>()), 0);
    drop(app2);
    let _ = fs::remove_dir_all(&dir).await;
}
