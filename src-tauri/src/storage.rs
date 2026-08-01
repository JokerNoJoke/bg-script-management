use std::path::Path;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::models::{now_ms, RunRecord, RunStatus, Script, ShellConfig};

const MAX_HISTORY: usize = 500;
const SCRIPTS_FILE: &str = "scripts.json";
const SHELLS_FILE: &str = "shells.json";
const HISTORY_FILE: &str = "history.json";

async fn ensure_dir(dir: &Path) -> Result<(), String> {
    fs::create_dir_all(dir)
        .await
        .map_err(|e| format!("创建数据目录失败: {e}"))
}

async fn read_json<T: serde::de::DeserializeOwned>(dir: &Path, file: &str) -> Result<Vec<T>, String> {
    match fs::read_to_string(dir.join(file)).await {
        Ok(text) if text.trim().is_empty() => Ok(Vec::new()),
        Ok(text) => serde_json::from_str(&text).map_err(|e| format!("解析 {file} 失败: {e}")),
        Err(_) => Ok(Vec::new()),
    }
}

async fn write_json<T: serde::Serialize>(dir: &Path, file: &str, value: &T) -> Result<(), String> {
    let content =
        serde_json::to_string_pretty(value).map_err(|e| format!("序列化 {file} 失败: {e}"))?;
    fs::write(dir.join(file), content)
        .await
        .map_err(|e| format!("写入 {file} 失败: {e}"))
}

fn sort_scripts(scripts: &mut Vec<Script>) {
    scripts.sort_by_key(|s| s.created_at);
}

fn sort_shells(shells: &mut Vec<ShellConfig>) {
    shells.sort_by(|a, b| b.builtin.cmp(&a.builtin).then_with(|| a.name.cmp(&b.name)));
}

// ---- scripts ----

pub async fn load_scripts(dir: &Path) -> Vec<Script> {
    read_json(dir, SCRIPTS_FILE).await.unwrap_or_default()
}

pub async fn save_script(dir: &Path, mut script: Script) -> Result<Script, String> {
    ensure_dir(dir).await?;
    let mut scripts = load_scripts(dir).await;
    if script.id.is_empty() {
        script.id = uuid::Uuid::new_v4().to_string();
        script.created_at = now_ms();
        script.updated_at = script.created_at;
        scripts.push(script.clone());
    } else {
        script.updated_at = now_ms();
        let Some(existing) = scripts.iter_mut().find(|s| s.id == script.id) else {
            return Err("脚本不存在".into());
        };
        *existing = script.clone();
    }
    sort_scripts(&mut scripts);
    write_json(dir, SCRIPTS_FILE, &scripts).await?;
    Ok(script)
}

pub async fn delete_script(dir: &Path, id: &str) -> Result<(), String> {
    ensure_dir(dir).await?;
    let mut scripts = load_scripts(dir).await;
    let before = scripts.len();
    scripts.retain(|s| s.id != id);
    if scripts.len() == before {
        return Err("脚本不存在".into());
    }
    write_json(dir, SCRIPTS_FILE, &scripts).await
}

// ---- shells ----

pub async fn load_shells(dir: &Path) -> Vec<ShellConfig> {
    read_json(dir, SHELLS_FILE).await.unwrap_or_default()
}

pub async fn save_shell(dir: &Path, mut shell: ShellConfig) -> Result<ShellConfig, String> {
    shell.kind = ShellConfig::infer_kind(&shell.exe);
    ensure_dir(dir).await?;
    let mut shells = load_shells(dir).await;
    if shell.id.is_empty() {
        shell.id = uuid::Uuid::new_v4().to_string();
        shells.push(shell.clone());
    } else {
        let Some(existing) = shells.iter_mut().find(|s| s.id == shell.id) else {
            return Err("Shell 不存在".into());
        };
        *existing = shell.clone();
    }
    sort_shells(&mut shells);
    write_json(dir, SHELLS_FILE, &shells).await?;
    Ok(shell)
}

pub async fn delete_shell(dir: &Path, id: &str) -> Result<(), String> {
    ensure_dir(dir).await?;
    let mut shells = load_shells(dir).await;
    let shell = shells
        .iter()
        .find(|s| s.id == id)
        .ok_or("Shell 不存在")?;
    if shell.builtin {
        return Err("内置 Shell 不可删除".into());
    }
    let scripts = load_scripts(dir).await;
    if scripts.iter().any(|s| s.shell_id == id) {
        return Err("有脚本正在引用该 Shell，无法删除".into());
    }
    shells.retain(|s| s.id != id);
    write_json(dir, SHELLS_FILE, &shells).await
}

pub async fn merge_detected(dir: &Path, detected: &[ShellConfig]) -> Result<(), String> {
    ensure_dir(dir).await?;
    let mut shells = load_shells(dir).await;
    for d in detected {
        if !shells.iter().any(|s| s.exe == d.exe) {
            shells.push(d.clone());
        }
    }
    sort_shells(&mut shells);
    write_json(dir, SHELLS_FILE, &shells).await
}

// ---- runs ----

pub async fn load_runs(dir: &Path) -> Vec<RunRecord> {
    read_json(dir, HISTORY_FILE).await.unwrap_or_default()
}

pub async fn append_run(dir: &Path, run: RunRecord) -> Result<(), String> {
    ensure_dir(dir).await?;
    let mut runs = load_runs(dir).await;
    runs.insert(0, run);
    if runs.len() > MAX_HISTORY {
        runs.truncate(MAX_HISTORY);
    }
    write_json(dir, HISTORY_FILE, &runs).await
}

pub async fn update_run_status(
    dir: &Path,
    id: &str,
    status: RunStatus,
    exit_code: Option<i32>,
    finished_at: u64,
) -> Result<(), String> {
    ensure_dir(dir).await?;
    let mut runs = load_runs(dir).await;
    let rec = runs
        .iter_mut()
        .find(|r| r.id == id)
        .ok_or("运行记录不存在")?;
    rec.status = status;
    rec.exit_code = exit_code;
    rec.finished_at = Some(finished_at);
    write_json(dir, HISTORY_FILE, &runs).await
}

/// 清除历史记录：`script_id = Some(id)` 时只清该脚本的，`None` 时全清。
/// 被清除记录的 `logs/` 文件一并删除。
pub async fn clear_history(dir: &Path, script_id: Option<&str>) -> Result<(), String> {
    ensure_dir(dir).await?;
    let mut runs = load_runs(dir).await;
    let removed_ids: Vec<String> = match script_id {
        Some(id) => {
            let mut removed = Vec::new();
            runs.retain(|r| {
                if r.script_id.as_deref() == Some(id) {
                    removed.push(r.id.clone());
                    false
                } else {
                    true
                }
            });
            removed
        }
        None => {
            let removed = runs.iter().map(|r| r.id.clone()).collect();
            runs.clear();
            removed
        }
    };
    write_json(dir, HISTORY_FILE, &runs).await?;
    for rid in &removed_ids {
        let _ = fs::remove_file(dir.join("logs").join(format!("{rid}.log"))).await;
    }
    Ok(())
}

pub async fn migrate_running_on_boot(dir: &Path) -> Result<usize, String> {
    ensure_dir(dir).await?;
    let mut runs = load_runs(dir).await;
    let now = now_ms();
    let mut count = 0;
    for r in runs.iter_mut() {
        if r.status == RunStatus::Running {
            r.status = RunStatus::Interrupted;
            r.finished_at = Some(now);
            count += 1;
        }
    }
    if count > 0 {
        write_json(dir, HISTORY_FILE, &runs).await?;
    }
    Ok(count)
}

// ---- logs ----

pub async fn append_log(dir: &Path, run_id: &str, text: &str) -> Result<(), String> {
    let log_dir = dir.join("logs");
    fs::create_dir_all(&log_dir)
        .await
        .map_err(|e| format!("创建日志目录失败: {e}"))?;
    let path = log_dir.join(format!("{run_id}.log"));
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .await
        .map_err(|e| format!("打开日志文件失败: {e}"))?;
    file.write_all(text.as_bytes())
        .await
        .map_err(|e| format!("写入日志失败: {e}"))?;
    file.flush()
        .await
        .map_err(|e| format!("刷新日志失败: {e}"))
}

pub async fn read_log(dir: &Path, run_id: &str) -> Result<String, String> {
    let path = dir.join("logs").join(format!("{run_id}.log"));
    fs::read_to_string(&path)
        .await
        .map_err(|e| format!("读取日志失败: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RunStatus, ShellConfig, ShellKind};
    use crate::{detect, models::RunRecord};

    async fn temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("bsm-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).await.unwrap();
        dir
    }

    fn sample_script(name: &str) -> Script {
        Script {
            id: String::new(),
            name: name.to_string(),
            description: String::new(),
            shell_id: "builtin-cmd".into(),
            exec_type: Default::default(),
            command: "echo hi".into(),
            cwd: None,
            env: Default::default(),
            timeout_sec: 0,
            created_at: 0,
            updated_at: 0,
        }
    }

    fn sample_run(i: u32) -> RunRecord {
        RunRecord {
            id: format!("run-{i}"),
            script_id: None,
            script_name: format!("s{i}"),
            shell_id: "builtin-cmd".into(),
            shell_name: "CMD".into(),
            command: format!("run-{i}"),
            cwd: None,
            status: RunStatus::Success,
            exit_code: Some(0),
            started_at: now_ms(),
            finished_at: Some(now_ms()),
            log_path: format!("logs/run-{i}.log"),
        }
    }

    #[tokio::test]
    async fn save_script_creates_then_updates() {
        let dir = temp_dir().await;
        let mut s = save_script(&dir, sample_script("a")).await.unwrap();
        assert!(!s.id.is_empty());
        let created = s.created_at;
        s.name = "b".into();
        let u = save_script(&dir, s.clone()).await.unwrap();
        assert_eq!(u.id, s.id);
        assert_eq!(u.name, "b");
        assert!(u.updated_at >= created);
        let all = load_scripts(&dir).await;
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "b");
        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn append_run_evicts_oldest_over_500() {
        let dir = temp_dir().await;
        for i in 0..510u32 {
            append_run(&dir, sample_run(i)).await.unwrap();
        }
        let runs = load_runs(&dir).await;
        assert_eq!(runs.len(), 500);
        assert_eq!(runs.first().unwrap().command, "run-509");
        assert_eq!(runs.last().unwrap().command, "run-10");
        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn migrate_running_on_boot_marks_interrupted() {
        let dir = temp_dir().await;
        let mut r1 = sample_run(1);
        r1.status = RunStatus::Running;
        append_run(&dir, r1).await.unwrap();
        let mut r2 = sample_run(2);
        r2.status = RunStatus::Success;
        append_run(&dir, r2).await.unwrap();
        let n = migrate_running_on_boot(&dir).await.unwrap();
        assert_eq!(n, 1);
        let runs = load_runs(&dir).await;
        let rec = runs.iter().find(|r| r.command == "run-1").unwrap();
        assert_eq!(rec.status, RunStatus::Interrupted);
        assert!(rec.finished_at.is_some());
        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn clear_history_filters_by_script_and_deletes_logs() {
        let dir = temp_dir().await;
        let mut r1 = sample_run(1);
        r1.script_id = Some("s1".into());
        append_run(&dir, r1).await.unwrap();
        append_log(&dir, "run-1", "hello").await.unwrap();

        let mut r2 = sample_run(2);
        r2.script_id = Some("s2".into());
        append_run(&dir, r2).await.unwrap();
        append_log(&dir, "run-2", "world").await.unwrap();

        clear_history(&dir, Some("s1")).await.unwrap();
        let runs = load_runs(&dir).await;
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].command, "run-2");
        assert!(read_log(&dir, "run-1").await.is_err(), "run-1 日志应被删除");
        assert!(read_log(&dir, "run-2").await.is_ok(), "run-2 日志应保留");

        clear_history(&dir, None).await.unwrap();
        assert!(load_runs(&dir).await.is_empty());
        assert!(read_log(&dir, "run-2").await.is_err(), "全清后 run-2 日志应被删除");
        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn delete_builtin_shell_rejected() {
        let dir = temp_dir().await;
        let detected = detect::detect_shells();
        let builtin = detected.iter().find(|s| s.builtin).unwrap().clone();
        merge_detected(&dir, &[builtin.clone()]).await.unwrap();
        let res = delete_shell(&dir, &builtin.id).await;
        assert!(res.is_err());
        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn delete_referenced_custom_shell_rejected() {
        let dir = temp_dir().await;
        let shell = save_shell(
            &dir,
            ShellConfig {
                id: String::new(),
                name: "自定义".into(),
                kind: ShellKind::Cmd,
                exe: "/custom/cmd".into(),
                args: vec![],
                builtin: false,
            },
        )
        .await
        .unwrap();
        save_script(
            &dir,
            Script {
                id: String::new(),
                name: "引用者".into(),
                description: String::new(),
                shell_id: shell.id.clone(),
                exec_type: Default::default(),
                command: "echo hi".into(),
                cwd: None,
                env: Default::default(),
                timeout_sec: 0,
                created_at: 0,
                updated_at: 0,
            },
        )
        .await
        .unwrap();
        let res = delete_shell(&dir, &shell.id).await;
        assert!(res.is_err(), "被脚本引用的自定义 Shell 不应可删除");
        // 未被引用时删除应成功
        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn save_shell_infers_kind_from_exe() {
        let dir = temp_dir().await;
        let shell = save_shell(
            &dir,
            ShellConfig {
                id: String::new(),
                name: "Git Bash".into(),
                // 前端不再下发 kind，serde 缺省为 PowerShell；保存时按 exe 推断
                kind: ShellKind::PowerShell,
                exe: "C:/Program Files/Git/bin/bash.exe".into(),
                args: vec![],
                builtin: false,
            },
        )
        .await
        .unwrap();
        assert_eq!(shell.kind, ShellKind::Bash);
        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn merge_detected_preserves_custom_and_adds_builtin() {
        let dir = temp_dir().await;
        let custom = ShellConfig {
            id: String::new(),
            name: "自定义".into(),
            kind: ShellKind::Bash,
            exe: "/custom/bash".into(),
            args: vec![],
            builtin: false,
        };
        save_shell(&dir, custom.clone()).await.unwrap();
        let detected = detect::detect_shells();
        merge_detected(&dir, &detected).await.unwrap();
        let shells = load_shells(&dir).await;
        assert!(shells.iter().any(|s| s.exe == "/custom/bash" && !s.builtin));
        for d in &detected {
            assert!(shells.iter().any(|s| s.exe == d.exe), "缺失 {}", d.exe);
        }
        let mut exes: Vec<&str> = shells.iter().map(|s| s.exe.as_str()).collect();
        let before = exes.len();
        exes.sort_unstable();
        exes.dedup();
        assert_eq!(before, exes.len(), "shell exe 存在重复");
        let _ = fs::remove_dir_all(&dir).await;
    }
}
