use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::ipc::Channel;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};

use crate::models::{now_ms, ExecType, OutputEvent, RunInput, RunRecord, RunStatus, ShellConfig, ShellKind};
use crate::state::{AppState, ChildHandle};
use crate::storage;

#[cfg(unix)]
use std::os::unix::process::CommandExt;

/// 按 shell kind 把 RunInput 解析成真实进程命令。命令串/文件路径/工作目录
/// 一律走 args 数组，不做字符串拼接。返回 std 命令便于测试校验，派生时再转 tokio。
pub fn resolve_command(shell: &ShellConfig, input: &RunInput) -> Result<std::process::Command, String> {
    let mut cmd = std::process::Command::new(&shell.exe);
    cmd.args(&shell.args);

    match shell.kind {
        ShellKind::PowerShell => match input.exec_type {
            ExecType::Command => {
                cmd.arg("-Command");
                cmd.arg(&input.command);
            }
            ExecType::File => {
                cmd.arg("-File");
                cmd.arg(&input.command);
            }
        },
        ShellKind::Cmd => {
            if cfg!(target_os = "windows") {
                cmd.arg("/C");
                cmd.arg(&input.command);
            } else {
                return Err("cmd 仅支持 Windows".into());
            }
        }
        ShellKind::Bash | ShellKind::Sh => match input.exec_type {
            ExecType::Command => {
                cmd.arg("-c");
                cmd.arg(&input.command);
            }
            ExecType::File => {
                cmd.arg(&input.command);
            }
        },
    }

    if let Some(cwd) = &input.cwd {
        if !Path::new(cwd).is_dir() {
            return Err(format!("工作目录不存在: {cwd}"));
        }
        cmd.current_dir(cwd);
    }
    cmd.envs(&input.env);
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    #[cfg(unix)]
    cmd.process_group(0);

    Ok(cmd)
}

/// 派生进程并注册。返回的 RunRecord 已写入 history（status=running）。
pub async fn spawn(
    app: &AppState,
    input: RunInput,
    channel: Channel<OutputEvent>,
) -> Result<RunRecord, String> {
    let shells = storage::load_shells(&app.data_dir).await;
    let shell = shells
        .iter()
        .find(|s| s.id == input.shell_id)
        .ok_or("shell 不存在")?
        .clone();

    let mut cmd: tokio::process::Command = resolve_command(&shell, &input)?.into();
    let mut child = cmd.spawn().map_err(|e| format!("启动进程失败: {e}"))?;
    let pid = child.id().unwrap_or(0);
    let stdout = child.stdout.take().ok_or("stdout 不可用")?;
    let stderr = child.stderr.take().ok_or("stderr 不可用")?;

    let run_id = uuid::Uuid::new_v4().to_string();
    let now = now_ms();
    let record = RunRecord {
        id: run_id.clone(),
        script_id: input.script_id.clone(),
        script_name: input.script_name.clone(),
        shell_id: input.shell_id.clone(),
        shell_name: shell.name.clone(),
        command: input.command.clone(),
        cwd: input.cwd.clone(),
        status: RunStatus::Running,
        exit_code: None,
        started_at: now,
        finished_at: None,
        log_path: format!("logs/{run_id}.log"),
    };
    storage::append_run(&app.data_dir, record.clone()).await?;

    let killed = Arc::new(AtomicBool::new(false));
    app.running
        .lock()
        .unwrap()
        .insert(run_id.clone(), ChildHandle { pid, killed: killed.clone() });

    let _ = channel.send(OutputEvent::Start { run_id: run_id.clone(), pid });

    let data_dir = app.data_dir.clone();
    let running = app.running.clone();
    let out_task = tokio::spawn(pipe_reader(
        data_dir.clone(),
        run_id.clone(),
        stdout,
        false,
        channel.clone(),
    ));
    let err_task = tokio::spawn(pipe_reader(
        data_dir.clone(),
        run_id.clone(),
        stderr,
        true,
        channel.clone(),
    ));
    tokio::spawn(monitor_task(
        running,
        killed,
        data_dir,
        run_id,
        input.timeout_sec,
        child,
        out_task,
        err_task,
        channel,
    ));

    Ok(record)
}

/// 终止一个运行中的任务：置 killed 标记并杀整个进程树。
/// 状态回写与 Exit 事件由 monitor 完成。
pub async fn kill(app: &AppState, run_id: &str) -> Result<(), String> {
    let entry = {
        let map = app.running.lock().unwrap();
        map.get(run_id).map(|h| (h.pid, h.killed.clone()))
    };
    let Some((pid, killed)) = entry else {
        return Err("任务不存在或已结束".into());
    };
    killed.store(true, Ordering::SeqCst);
    kill_by_pid(pid).await
}

/// 子进程输出解码：自动识别 UTF-8 / GBK。Windows 中文控制台（java、cmd 等）
/// 默认按系统 ANSI 代码页（简体中文=GBK/CP936）输出，硬按 UTF-8 解会乱码。
/// 同一进程基本只用一种编码：先按 UTF-8 解，遇到非法 UTF-8 即锁定为 GBK。
/// encoding_rs 的有状态解码器负责跨 chunk 拆分的多字节字符。
struct OutputDecoder {
    /// UTF-8 阶段尚未消费的尾部字节（可能是未完成的多字节序列）
    pending: Vec<u8>,
    /// Some = 已锁定 GBK 的有状态解码器
    gbk: Option<encoding_rs::Decoder>,
}

impl OutputDecoder {
    fn new() -> Self {
        Self { pending: Vec::new(), gbk: None }
    }

    /// 追加一段原始字节，返回解码出的文本（可能为空）。
    fn feed(&mut self, chunk: &[u8]) -> String {
        if let Some(dec) = &mut self.gbk {
            return gbk_decode(dec, chunk);
        }
        self.pending.extend_from_slice(chunk);
        match std::str::from_utf8(&self.pending) {
            Ok(s) => {
                let out = s.to_owned();
                self.pending.clear();
                out
            }
            Err(e) => match e.error_len() {
                None => {
                    // 尾部是多字节序列没凑齐，等下一个 chunk
                    let n = e.valid_up_to();
                    let out = String::from_utf8_lossy(&self.pending[..n]).into_owned();
                    self.pending.drain(..n);
                    out
                }
                Some(_) => {
                    // 出现非法 UTF-8 → 锁定 GBK，把已积累字节整体按 GBK 重解
                    let all = std::mem::take(&mut self.pending);
                    let mut dec = encoding_rs::GBK.new_decoder();
                    let out = gbk_decode(&mut dec, &all);
                    self.gbk = Some(dec);
                    out
                }
            },
        }
    }
}

/// 用有状态 GBK 解码器解一段字节。输出 String 必须预留足量容量：
/// `decode_to_string` 以 capacity 为输出上限，容量不够会 OutputFull 静默截断。
fn gbk_decode(dec: &mut encoding_rs::Decoder, bytes: &[u8]) -> String {
    let mut out = String::new();
    let cap = dec
        .max_utf8_buffer_length(bytes.len())
        .unwrap_or(bytes.len());
    out.reserve(cap);
    let _ = dec.decode_to_string(bytes, &mut out, false);
    out
}

/// 逐块读取子进程输出：追加写磁盘日志，同时推给前端。
/// 每个 reader 打开一次日志文件并缓冲写入，EOF 时统一 flush——
/// 旧实现每个 chunk 都 create_dir_all + open + flush，高输出进程会打爆 syscall。
/// stdout/stderr 各持一个 O_APPEND 句柄并发追加，末尾互不覆盖；monitor 先回收
/// reader 再发 Exit，保证磁盘日志在 Exit 事件前完整。写盘失败不阻塞实时推送。
async fn pipe_reader(
    data_dir: PathBuf,
    run_id: String,
    mut reader: impl AsyncRead + Unpin + Send,
    is_stderr: bool,
    channel: Channel<OutputEvent>,
) {
    let mut buf = vec![0u8; 8192];
    let mut decoder = OutputDecoder::new();
    let _ = tokio::fs::create_dir_all(data_dir.join("logs")).await;
    let mut writer = match tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(data_dir.join("logs").join(format!("{run_id}.log")))
        .await
    {
        Ok(file) => Some(tokio::io::BufWriter::with_capacity(16 * 1024, file)),
        Err(_) => None,
    };
    loop {
        let n = match reader.read(&mut buf).await {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        let text = decoder.feed(&buf[..n]);
        if text.is_empty() {
            continue;
        }
        let ok = match writer.as_mut() {
            Some(w) => w.write_all(text.as_bytes()).await.is_ok(),
            None => true,
        };
        if !ok {
            writer = None;
        }
        let ev = if is_stderr {
            OutputEvent::Stderr { run_id: run_id.clone(), data: text }
        } else {
            OutputEvent::Stdout { run_id: run_id.clone(), data: text }
        };
        let _ = channel.send(ev);
    }
    if let Some(mut w) = writer {
        let _ = w.flush().await;
    }
}

enum Waited {
    Ok(Option<i32>),
    TimedOut,
}

/// 等待子进程结束（含超时），回收输出 reader，统一回写状态并推 Exit 事件。
async fn monitor_task(
    running: Arc<Mutex<HashMap<String, ChildHandle>>>,
    killed: Arc<AtomicBool>,
    data_dir: PathBuf,
    run_id: String,
    timeout_sec: u64,
    mut child: tokio::process::Child,
    out_reader: tokio::task::JoinHandle<()>,
    err_reader: tokio::task::JoinHandle<()>,
    channel: Channel<OutputEvent>,
) {
    let waited = if timeout_sec == 0 {
        Waited::Ok(child.wait().await.ok().and_then(|s| s.code()))
    } else {
        match tokio::time::timeout(Duration::from_secs(timeout_sec), child.wait()).await {
            Ok(res) => Waited::Ok(res.ok().and_then(|s| s.code())),
            Err(_) => {
                if let Some(pid) = child.id() {
                    let _ = kill_by_pid(pid).await;
                }
                let _ = child.wait().await;
                Waited::TimedOut
            }
        }
    };

    // 等 reader 收完，保证磁盘日志在 Exit 事件之前是完整的
    let _ = out_reader.await;
    let _ = err_reader.await;

    let (status, code) = if killed.load(Ordering::SeqCst) {
        (RunStatus::Killed, None)
    } else {
        match waited {
            Waited::TimedOut => (RunStatus::Timeout, None),
            Waited::Ok(Some(0)) => (RunStatus::Success, Some(0)),
            Waited::Ok(code) => (RunStatus::Failed, code),
        }
    };

    let _ = storage::update_run_status(&data_dir, &run_id, status, code, now_ms()).await;
    running.lock().unwrap().remove(&run_id);
    let _ = channel.send(OutputEvent::Exit { run_id, code });
}

#[cfg(windows)]
async fn kill_by_pid(pid: u32) -> Result<(), String> {
    let status = tokio::process::Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/T", "/F"])
        .status()
        .await
        .map_err(|e| format!("taskkill 调用失败: {e}"))?;
    match status.code() {
        Some(0) => Ok(()),
        Some(128) => Ok(()), // 进程已不存在
        Some(c) => Err(format!("taskkill 退出码 {c}")),
        None => Err("taskkill 被信号终止".into()),
    }
}

#[cfg(not(windows))]
async fn kill_by_pid(pid: u32) -> Result<(), String> {
    let pgid = -(pid as i32);
    unsafe {
        libc::kill(pgid, libc::SIGTERM);
    }
    tokio::time::sleep(Duration::from_millis(800)).await;
    unsafe {
        libc::kill(pgid, libc::SIGKILL);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ExecType, OutputEvent, ShellConfig, ShellKind};
    use crate::state::AppState;
    use crate::storage;
    use std::sync::Mutex;
    use tauri::ipc::{Channel, InvokeResponseBody};
    use tokio::fs;

    fn sample_shell(kind: ShellKind, exe: &str, args: Vec<&str>, id: &str) -> ShellConfig {
        ShellConfig {
            id: id.into(),
            name: id.into(),
            kind,
            exe: exe.into(),
            args: args.into_iter().map(Into::into).collect(),
            builtin: false,
        }
    }

    fn base_input(shell_id: &str, command: String) -> RunInput {
        RunInput {
            script_id: None,
            script_name: "t".into(),
            shell_id: shell_id.into(),
            command,
            exec_type: ExecType::Command,
            cwd: None,
            env: Default::default(),
            timeout_sec: 0,
        }
    }

    async fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("bsm-runner-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).await.unwrap();
        dir
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

    #[cfg(target_os = "windows")]
    #[test]
    fn resolve_powershell_command() {
        let shell = sample_shell(ShellKind::PowerShell, "powershell.exe", vec!["-NoProfile"], "ps");
        let input = base_input("ps", "echo hi".into());
        let cmd = resolve_command(&shell, &input).unwrap();
        assert_eq!(cmd.get_program().to_string_lossy(), "powershell.exe");
        let args: Vec<String> = cmd.get_args().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args, vec!["-NoProfile", "-Command", "echo hi"]);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn resolve_cmd_file() {
        let shell = sample_shell(ShellKind::Cmd, "cmd.exe", vec![], "cmd");
        let mut input = base_input("cmd", r"C:\tmp\a.bat".into());
        input.exec_type = ExecType::File;
        let cmd = resolve_command(&shell, &input).unwrap();
        let args: Vec<String> = cmd.get_args().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args, vec!["/C", r"C:\tmp\a.bat"]);
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn resolve_cmd_rejected_on_unix() {
        let shell = sample_shell(ShellKind::Cmd, "cmd.exe", vec![], "cmd");
        let input = base_input("cmd", "echo hi".into());
        assert!(resolve_command(&shell, &input).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn resolve_bash_command_unix() {
        let shell = sample_shell(ShellKind::Bash, "/bin/bash", vec!["-l"], "bash");
        let input = base_input("bash", "echo hi".into());
        let cmd = resolve_command(&shell, &input).unwrap();
        let args: Vec<String> = cmd.get_args().map(|a| a.to_string_lossy().into_owned()).collect();
        assert_eq!(args, vec!["-l", "-c", "echo hi"]);
    }

    #[test]
    fn resolve_rejects_missing_cwd() {
        let shell = sample_shell(ShellKind::PowerShell, "powershell.exe", vec![], "ps");
        let mut input = base_input("ps", "echo hi".into());
        input.cwd = Some("Z:/definitely/not/a/dir".into());
        assert!(resolve_command(&shell, &input).is_err());
    }

    async fn platform_shell() -> (ShellConfig, String) {
        #[cfg(target_os = "windows")]
        {
            (sample_shell(ShellKind::Cmd, "cmd.exe", vec![], ""), "cmd".into())
        }
        #[cfg(not(target_os = "windows"))]
        {
            (sample_shell(ShellKind::Sh, "sh", vec![], ""), "sh".into())
        }
    }

    #[tokio::test]
    async fn spawn_emits_start_stdout_exit() {
        let dir = temp_dir().await;
        let (shell, _) = platform_shell().await;
        let saved = storage::save_shell(&dir, shell).await.unwrap();
        let shell_id = saved.id.clone();
        let app = AppState::new(dir.clone());
        let (channel, events) = collect_channel();
        let input = base_input(&shell_id, "echo BSM_OK".into());
        let record = spawn(&app, input, channel).await.unwrap();
        assert_eq!(record.status, RunStatus::Running);
        assert!(!record.id.is_empty());

        let evs = wait_for_exit(&events, 8000).await;
        assert!(evs.iter().any(|e| matches!(e, OutputEvent::Start { .. })));
        let stdout_text: String = evs
            .iter()
            .filter_map(|e| match e {
                OutputEvent::Stdout { data, .. } => Some(data.clone()),
                _ => None,
            })
            .collect();
        assert!(stdout_text.contains("BSM_OK"));
        let exit = evs
            .iter()
            .find_map(|e| match e {
                OutputEvent::Exit { code, .. } => Some(*code),
                _ => None,
            })
            .unwrap();
        assert_eq!(exit, Some(0));

        let runs = storage::load_runs(&dir).await;
        let rec = runs.iter().find(|r| r.id == record.id).unwrap();
        assert_eq!(rec.status, RunStatus::Success);
        let log = storage::read_log(&dir, &record.id).await.unwrap();
        assert!(log.contains("BSM_OK"));
        assert!(app.running.lock().unwrap().is_empty());
        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn spawn_timeout_kills_and_marks_timeout() {
        let dir = temp_dir().await;
        let (shell, _) = platform_shell().await;
        let saved = storage::save_shell(&dir, shell).await.unwrap();
        let shell_id = saved.id.clone();
        let app = AppState::new(dir.clone());
        let (channel, events) = collect_channel();
        let mut input = base_input(&shell_id, "sleep 20".into());
        #[cfg(target_os = "windows")]
        {
            input.command = "ping -n 20 127.0.0.1".into();
        }
        input.timeout_sec = 1;
        let record = spawn(&app, input, channel).await.unwrap();

        let evs = wait_for_exit(&events, 10000).await;
        let exit = evs
            .iter()
            .find_map(|e| match e {
                OutputEvent::Exit { code, .. } => Some(*code),
                _ => None,
            })
            .unwrap();
        assert_eq!(exit, None);
        let runs = storage::load_runs(&dir).await;
        let rec = runs.iter().find(|r| r.id == record.id).unwrap();
        assert_eq!(rec.status, RunStatus::Timeout);
        assert!(app.running.lock().unwrap().is_empty());
        let _ = fs::remove_dir_all(&dir).await;
    }

    #[tokio::test]
    async fn kill_task_marks_killed() {
        let dir = temp_dir().await;
        let (shell, _) = platform_shell().await;
        let saved = storage::save_shell(&dir, shell).await.unwrap();
        let shell_id = saved.id.clone();
        let app = AppState::new(dir.clone());
        let (channel, events) = collect_channel();
        let mut input = base_input(&shell_id, "sleep 60".into());
        #[cfg(target_os = "windows")]
        {
            input.command = "ping -n 60 127.0.0.1".into();
        }
        let record = spawn(&app, input, channel).await.unwrap();

        tokio::time::sleep(Duration::from_millis(500)).await;
        kill(&app, &record.id).await.unwrap();

        let evs = wait_for_exit(&events, 10000).await;
        #[cfg(unix)]
        let pid = evs
            .iter()
            .find_map(|e| match e {
                OutputEvent::Start { pid, .. } => Some(*pid),
                _ => None,
            })
            .unwrap();
        let exit = evs
            .iter()
            .find_map(|e| match e {
                OutputEvent::Exit { code, .. } => Some(*code),
                _ => None,
            })
            .unwrap();
        assert_eq!(exit, None);
        let runs = storage::load_runs(&dir).await;
        let rec = runs.iter().find(|r| r.id == record.id).unwrap();
        assert_eq!(rec.status, RunStatus::Killed);
        assert!(app.running.lock().unwrap().is_empty());

        #[cfg(unix)]
        {
            let mut gone = false;
            for _ in 0..50 {
                if unsafe { libc::kill(pid as i32, 0) } != 0 {
                    gone = true;
                    break;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            assert!(gone, "进程仍然存在 pid={pid}");
        }
        let _ = fs::remove_dir_all(&dir).await;
    }

    #[test]
    fn output_decoder_passes_utf8_and_ascii() {
        let mut d = OutputDecoder::new();
        assert_eq!(d.feed(b"hello "), "hello ");
        assert_eq!(d.feed(&[0xE4, 0xBD]), ""); // UTF-8 多字节拆在两个 chunk
        assert_eq!(d.feed(&[0xA0, 0x21]), "你!");
    }

    #[test]
    fn output_decoder_falls_back_to_gbk() {
        let mut d = OutputDecoder::new();
        assert_eq!(d.feed(b"java: "), "java: ");
        // "中文" 的 GBK 字节：D6D0 CEC4
        assert_eq!(d.feed(&[0xD6, 0xD0, 0xCE, 0xC4]), "中文");
    }

    #[test]
    fn output_decoder_handles_gbk_split_across_chunks() {
        let mut d = OutputDecoder::new();
        // "测" 的 GBK 字节 B2E2 拆在两个 chunk 之间
        assert_eq!(d.feed(&[0x61, 0xB2]), "a");
        assert_eq!(d.feed(&[0xE2, 0xCE, 0xC4]), "测文");
    }
}
