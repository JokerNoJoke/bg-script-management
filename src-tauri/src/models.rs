use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 回归：事件经 IPC 序列化后字段必须是 camelCase（runId/pid），
    /// 否则前端 `ev.runId` 取不到，live 卡片会以 undefined 键悬挂。
    #[test]
    fn output_event_serializes_camel_case_fields() {
        let start = serde_json::to_value(OutputEvent::Start { run_id: "abc".into(), pid: 42 }).unwrap();
        assert_eq!(start["type"], "start");
        assert_eq!(start["runId"], "abc");
        assert_eq!(start["pid"], 42);
        assert!(start.get("run_id").is_none());

        let stdout = serde_json::to_value(OutputEvent::Stdout { run_id: "abc".into(), data: "hi".into() }).unwrap();
        assert_eq!(stdout["runId"], "abc");

        let stderr = serde_json::to_value(OutputEvent::Stderr { run_id: "abc".into(), data: "boom".into() }).unwrap();
        assert_eq!(stderr["runId"], "abc");

        let exit = serde_json::to_value(OutputEvent::Exit { run_id: "abc".into(), code: Some(2) }).unwrap();
        assert_eq!(exit["runId"], "abc");
        assert_eq!(exit["code"], 2);
    }

    /// 回归：`ShellKind::PowerShell` 必须序列化为前端契约里的 `powershell`，
    /// 而不是 serde camelCase 推导的 `powerShell`，否则 Shell 管理页种类列与
    /// 编辑表单下拉都取不到对应 label。
    #[test]
    fn shell_kind_serializes_to_frontend_contract() {
        for (kind, expect) in [
            (ShellKind::PowerShell, "powershell"),
            (ShellKind::Cmd, "cmd"),
            (ShellKind::Bash, "bash"),
            (ShellKind::Sh, "sh"),
        ] {
            let v = serde_json::to_value(kind).unwrap();
            assert_eq!(v.as_str().unwrap(), expect);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShellKind {
    // camelCase 会把 PowerShell 推导成 powerShell，与前端契约（types.ts 的
    // "powershell"）不一致，必须显式覆写为小写 p。
    #[serde(rename = "powershell")]
    PowerShell,
    Cmd,
    Bash,
    Sh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum ExecType {
    #[default]
    Command,
    File,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RunStatus {
    Running,
    Success,
    Failed,
    Killed,
    Timeout,
    Interrupted,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Script {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub shell_id: String,
    #[serde(default)]
    pub exec_type: ExecType,
    pub command: String,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub timeout_sec: u64,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellConfig {
    pub id: String,
    pub name: String,
    pub kind: ShellKind,
    pub exe: String,
    #[serde(default)]
    pub args: Vec<String>,
    pub builtin: bool,
}

impl ShellConfig {
    pub fn builtin(kind: ShellKind, id: &str, name: &str, exe: &str, args: Vec<String>) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            kind,
            exe: exe.to_string(),
            args,
            builtin: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunRecord {
    pub id: String,
    pub script_id: Option<String>,
    pub script_name: String,
    pub shell_id: String,
    pub shell_name: String,
    pub command: String,
    pub cwd: Option<String>,
    pub status: RunStatus,
    pub exit_code: Option<i32>,
    pub started_at: u64,
    pub finished_at: Option<u64>,
    pub log_path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunInput {
    pub script_id: Option<String>,
    pub script_name: String,
    pub shell_id: String,
    pub command: String,
    #[serde(default)]
    pub exec_type: ExecType,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub timeout_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum OutputEvent {
    // 枚举级 rename_all 只作用于 variant 的 tag，不会重命名字段；
    // 必须逐 variant 声明，否则 run_id 会以 snake_case 传给前端（runId 缺失）。
    #[serde(rename_all = "camelCase")]
    Start { run_id: String, pid: u32 },
    #[serde(rename_all = "camelCase")]
    Stdout { run_id: String, data: String },
    #[serde(rename_all = "camelCase")]
    Stderr { run_id: String, data: String },
    #[serde(rename_all = "camelCase")]
    Exit { run_id: String, code: Option<i32> },
}
